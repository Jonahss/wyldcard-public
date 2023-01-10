#[allow(dead_code)]
#[allow(unused)]
#[allow(unused_variables)]
#[allow(unused_imports)]
use std::error::Error;
use std::ffi::{ OsString };
use std::fmt;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool };
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::collections::HashMap;
use std::time::{ Duration, Instant };
use png;

use signal_hook::flag;
use signal_hook::consts::TERM_SIGNALS;
use rppal::gpio::{ Gpio, Trigger };
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::i2c::I2c;
use rppal::uart::{ Uart, Parity };

use embedded_hal::digital::v2::{ InputPin, OutputPin };
use embedded_hal::blocking::spi::Transfer;

use shared_bus::{BusManagerSimple, SpiProxy, I2cProxy, NullMutex};

use m95320::prelude::*;
use m95320::m95320::Flash;

use pcf857x::{ Pcf8574, SlaveAddr, P3 };

mod epaper_display;
use epaper_display::init_epd;

use rand::prelude::IteratorRandom;

use crate::epaper_display::GDEW029T5DController;

// Gpio uses BCM pin numbering. BCM GPIO 12 is tied to physical pin 32.
const GPIO_DATA_OR_COMMAND: u8 = 25;

// const GPIO_BUSY_0: u8 = 28;
// const GPIO_BUSY_1: u8 = 32;
// const GPIO_BUSY_2: u8 = 36;
// const GPIO_BUSY_3: u8 = 40;

// const GPIO_RESET_0: u8 = 27;
// const GPIO_RESET_1: u8 = 31;
// const GPIO_RESET_2: u8 = 35;
// const GPIO_RESET_3: u8 = 39;

// const GPIO_CHIP_SELECT_0: u8 = 29;
// const GPIO_CHIP_SELECT_1: u8 = 33;
// const GPIO_CHIP_SELECT_2: u8 = 37;
// const GPIO_CHIP_SELECT_3: u8 = 41;

// const GPIO_MEMORY_CHIP_SELECT_0: u8 = 30;
// const GPIO_MEMORY_CHIP_SELECT_1: u8 = 34;
// const GPIO_MEMORY_CHIP_SELECT_2: u8 = 38;
// const GPIO_MEMORY_CHIP_SELECT_3: u8 = 42;

const GPIO_SWITCH_A0: u8 = 23; // gpio #4 throws "in use" errors :(
const GPIO_SWITCH_B0: u8 = 5;
const GPIO_SWITCH_C0: u8 = 6;

const GPIO_SWITCH_A1: u8 = 21; // swapped for first version of plinth board
const GPIO_SWITCH_B1: u8 = 13;
const GPIO_SWITCH_C1: u8 = 12; // swapped for first version of plinth board

const GPIO_SWITCH_A2: u8 = 22;
const GPIO_SWITCH_B2: u8 = 16;
const GPIO_SWITCH_C2: u8 = 17;

const GPIO_SWITCH_A3: u8 = 18;
const GPIO_SWITCH_B3: u8 = 19;
const GPIO_SWITCH_C3: u8 = 20;

const IMAGE_DIR: &str = "/home/pi/Pictures/wyldcard/";
const CARD_COLLECTION: &str = "collectionB";

fn main() -> Result<(), Box<dyn Error>> {
  let terminate_now = Arc::new(AtomicBool::new(false));
  for sig in TERM_SIGNALS {
      flag::register_conditional_shutdown(*sig, 1, Arc::clone(&terminate_now))?;
      flag::register(*sig, Arc::clone(&terminate_now))?;
  }

  let gpio = Gpio::new().unwrap();

  // setup switches
  let mut switch_a0 = gpio.get(GPIO_SWITCH_A0).unwrap().into_input_pullup();
  let mut switch_b0 = gpio.get(GPIO_SWITCH_B0).unwrap().into_input_pullup();
  let mut switch_c0 = gpio.get(GPIO_SWITCH_C0).unwrap().into_input_pullup();
  let mut switch_a1 = gpio.get(GPIO_SWITCH_A1).unwrap().into_input_pullup();
  let mut switch_b1 = gpio.get(GPIO_SWITCH_B1).unwrap().into_input_pullup();
  let mut switch_c1 = gpio.get(GPIO_SWITCH_C1).unwrap().into_input_pullup();
  let mut switch_a2 = gpio.get(GPIO_SWITCH_A2).unwrap().into_input_pullup();
  let mut switch_b2 = gpio.get(GPIO_SWITCH_B2).unwrap().into_input_pullup();
  let mut switch_c2 = gpio.get(GPIO_SWITCH_C2).unwrap().into_input_pullup();
  let mut switch_a3 = gpio.get(GPIO_SWITCH_A3).unwrap().into_input_pullup();
  let mut switch_b3 = gpio.get(GPIO_SWITCH_B3).unwrap().into_input_pullup();
  let mut switch_c3 = gpio.get(GPIO_SWITCH_C3).unwrap().into_input_pullup();
  
  let switch_names = HashMap::from([
    (GPIO_SWITCH_A0, "Switch A for Card 0"),
    (GPIO_SWITCH_B0, "Switch B for Card 0"),
    (GPIO_SWITCH_C0, "Switch C for Card 0"),
    (GPIO_SWITCH_A1, "Switch A for Card 1"),
    (GPIO_SWITCH_B1, "Switch B for Card 1"),
    (GPIO_SWITCH_C1, "Switch C for Card 1"),
    (GPIO_SWITCH_A2, "Switch A for Card 2"),
    (GPIO_SWITCH_B2, "Switch B for Card 2"),
    (GPIO_SWITCH_C2, "Switch C for Card 2"),
    (GPIO_SWITCH_A3, "Switch A for Card 3"),
    (GPIO_SWITCH_B3, "Switch B for Card 3"),
    (GPIO_SWITCH_C3, "Switch C for Card 3"),
  ]);

  switch_a0.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch a0");
  switch_b0.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch b0");
  switch_c0.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch c0");
  switch_a1.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch a1");
  switch_b1.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch b1");
  switch_c1.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch c1");
  switch_a2.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch a2");
  switch_b2.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch b2");
  switch_c2.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch c2");
  switch_a3.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch a3");
  switch_b3.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch b3");
  switch_c3.set_interrupt(Trigger::RisingEdge).expect("problem setting interrupt on switch c3");

  let switch_refs = [
    &switch_a0,
    &switch_b0,
    &switch_c0,
    &switch_a1,
    &switch_b1,
    &switch_c1,
    &switch_a2,
    &switch_b2,
    &switch_c2,
    &switch_a3,
    &switch_b3,
    &switch_c3,
  ];

  //let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();
  //let spi_bus = shared_bus::BusManagerSimple::new(spi);
  let data_or_command: Rc<RefCell<dyn embedded_hal::digital::v2::OutputPin<Error = rppal::gpio::Error>>> = Rc::new(RefCell::new(gpio.get(GPIO_DATA_OR_COMMAND).unwrap().into_output()));
 
  let i2c = I2c::new().unwrap();
  let i2c_bus = shared_bus::BusManagerSimple::new(i2c);
  
  let expander_0 = Pcf8574::new(i2c_bus.acquire_i2c(), SlaveAddr::Alternative(false, false, false));
  let expander_1 = Pcf8574::new(i2c_bus.acquire_i2c(), SlaveAddr::Alternative(false, false, true));

  let mut virtual_gpios_0 = expander_0.split();
  let mut virtual_gpios_1 = expander_1.split();

  let mut display_0 = init_epd(&mut virtual_gpios_0.p0, &virtual_gpios_0.p1, Rc::clone(&data_or_command), &mut virtual_gpios_0.p2);
  let mut display_1 = init_epd(&mut virtual_gpios_0.p4, &virtual_gpios_0.p5, Rc::clone(&data_or_command), &mut virtual_gpios_0.p6);
  let mut display_2 = init_epd(&mut virtual_gpios_1.p0, &virtual_gpios_1.p1, Rc::clone(&data_or_command), &mut virtual_gpios_1.p2);
  let mut display_3 = init_epd(&mut virtual_gpios_1.p4, &virtual_gpios_1.p5, Rc::clone(&data_or_command), &mut virtual_gpios_1.p6);


  let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();
  let spi_bus = BusManagerSimple::new(spi);
  
  let mut flash_0 = Flash::init(spi_bus.acquire_spi(), virtual_gpios_0.p3).unwrap();
  let mut flash_1 = Flash::init(spi_bus.acquire_spi(), virtual_gpios_0.p7).unwrap();
  let mut flash_2 = Flash::init(spi_bus.acquire_spi(), virtual_gpios_1.p3).unwrap();
  let mut flash_3 = Flash::init(spi_bus.acquire_spi(), virtual_gpios_1.p7).unwrap();

  let mut uart = Uart::new(9600, Parity::Even, 8, 1)?;
  uart.set_write_mode(true)?;
  uart.set_read_mode(0, Duration::from_millis(100))?;
  


  fn increment_counter<T: Read<u16,S, CS> + BlockDevice<u16, S, CS>, S: Transfer<u8>, CS: OutputPin> (flash: &mut T) -> Result<(), m95320::Error<S, CS>> {
    let mut buf: [u8;32] = [0x0; 32];
    flash.read(0, &mut buf)?;
    buf[0] = buf[0].wrapping_add(1);
    println!("count is now {}", buf[0]);
    flash.write_bytes(0, &mut buf)?;
    Ok(())
  }

  fn display_peacock(display: &mut GDEW029T5DController) {
    let image = load_image("images/converted/Peacock.png").expect("problem loading image file");
    if let Err(e) = display.start_epd() {
      println!("{}", e); return;
    }
    display.display_image(image);
    display.sleep();
  }

  fn display_random_image(display: &mut GDEW029T5DController) {
    let path = random_image().into_string().expect("could not read path");
    let image = load_image(&path).expect("problem loading image file {path}");
    if let Err(e) = display.start_epd() {
      println!("{}", e); return;
    }
    display.display_image(image);
    display.sleep();
  }

  fn display_lady_red(display: &mut GDEW029T5DController) {
    let image = load_image("images/converted/ladyRed_0.png").expect("problem loading image file");
    if let Err(e) = display.start_epd() {
      println!("{}", e); return;
    }
    display.display_image(image);
    display.sleep();
  }

  let mut connected = false;
  let mut last_ping_received = Instant::now();
  let mut last_ping_sent = Instant::now();
  let ping_interval = Duration::from_millis(100);
 
  loop {
    if let Some((pin, _)) = gpio.poll_interrupts(&switch_refs, true, None).unwrap() {
      println!("{} pressed", switch_names.get(&pin.pin()).expect("could not find name for switch"));
      match pin.pin() {
        GPIO_SWITCH_A0 => {
          display_random_image(&mut display_0);
          increment_counter(&mut flash_0); 
        },
        GPIO_SWITCH_B0 => {
          display_random_image(&mut display_0);
          increment_counter(&mut flash_0); 
        },
        GPIO_SWITCH_C0 => {
          display_random_image(&mut display_0);
          increment_counter(&mut flash_0); 
        },
        GPIO_SWITCH_A1 => {
          display_random_image(&mut display_1);
          increment_counter(&mut flash_1); 
        },
        GPIO_SWITCH_B1 => {
          display_random_image(&mut display_1);
          increment_counter(&mut flash_1); 
        },
        GPIO_SWITCH_C1 => {
          display_random_image(&mut display_1);
          increment_counter(&mut flash_1); 
        },GPIO_SWITCH_A2 => {
          display_random_image(&mut display_2);
          increment_counter(&mut flash_2); 
        },
        GPIO_SWITCH_B2 => {
          display_random_image(&mut display_2);
          increment_counter(&mut flash_2); 
        },
        GPIO_SWITCH_C2 => {
          display_random_image(&mut display_2);
          increment_counter(&mut flash_2); 
        },
        GPIO_SWITCH_A3 => {
          display_random_image(&mut display_3);
          increment_counter(&mut flash_3); 
        },
        GPIO_SWITCH_B3 => {
          display_random_image(&mut display_3);
          increment_counter(&mut flash_3); 
        },
        GPIO_SWITCH_C3 => {
          display_random_image(&mut display_3);
          increment_counter(&mut flash_3); 
        },
        _ => println!("doing nothing"),
      }
    }


    // if !connected {
    //   uart.write(b"ping")?;
    //   let mut pong = [0x00; 4];
    //   match uart.read(&mut pong) {
    //     Ok(bytes_read) => {
    //       if bytes_read > 0 {
    //         if &pong == b"ping" {
    //           println!("got ping");
    //           last_ping_received = Instant::now();
    //           connected = true;
    //           uart.set_read_mode(0, Duration::new(0, 0))?;
    //         } else {
    //           println!("got data but we're not synced: {:?}", pong);
    //         }
    //       } else {
    //         println!("not connected yet");
    //       }
    //     },
    //     Err(x) => println!("not connected yet, error {:?}", x),
    //   } 
    // } else {
    //   if Instant::now() - last_ping_received > 3 * ping_interval {
    //     println!("disconnected!");
    //     connected = false;
    //     uart.set_read_mode(0, ping_interval)?;
    //   } else if Instant::now() - last_ping_sent > ping_interval {
    //     uart.write(b"ping")?;
    //     last_ping_sent = Instant::now();
    //   }

    //   let mut ping = [0x00; 4];
    //   let bytes_read = uart.read(&mut ping)?;
    //   if bytes_read > 0 {
    //     println!("got a ping: {:?}", ping);
    //     if &ping != b"ping" {
    //       println!("WASNT A FULL PING");
    //     }
    //     last_ping_received = Instant::now();
    //   }
    // }

  }

  Ok(())
}

#[derive(PartialEq, Eq, Debug)]
struct ImageFormat {
  width: u32,
  height: u32,
  bit_depth: png::BitDepth,
  color_type: png::ColorType,
}

struct ImageLoadError {
  expected: ImageFormat,
  received: ImageFormat,
}

impl fmt::Display for ImageLoadError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "input image format is not acceptable. expect: {:?}, received: {:?}", self.expected, self.received)
  }
}
impl fmt::Debug for ImageLoadError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "input image format is not acceptable. expect: {:?}, received: {:?}", self.expected, self.received)
  }
}

fn load_image(image_path: &str) -> Result<Vec<u8>, ImageLoadError> {

  let decoder = png::Decoder::new(File::open(image_path).unwrap());
  let mut reader = decoder.read_info().unwrap();

  println!("loading image");
  let image_info = reader.info();
  
  let expected_image_format = ImageFormat {
    width: 128,
    height: 296,
    bit_depth: png::BitDepth::Two,
    color_type: png::ColorType::Grayscale,
  };

  let received_image_format = ImageFormat {
    width: image_info.width,
    height: image_info.height,
    bit_depth: image_info.bit_depth,
    color_type: image_info.color_type,
  };

  if expected_image_format != received_image_format {
    return Err(ImageLoadError {
      received: received_image_format,
      expected: expected_image_format,
    })
  }

  let mut buf = vec![0; reader.output_buffer_size()];
  reader.next_frame(&mut buf).expect("Error decoding image");

  Ok(buf)
}

fn random_image() -> OsString {
  let root = Path::new("/");
  let directory_path = root.join(IMAGE_DIR).join(CARD_COLLECTION);
  let paths = fs::read_dir(directory_path.to_str().unwrap()).unwrap();
  paths.choose(&mut rand::thread_rng()).unwrap().unwrap().path().into_os_string()
}