#[allow(dead_code)]
#[allow(unused)]
#[allow(unused_variables)]
#[allow(unused_imports)]
use std::error::Error;

use std::collections::HashMap;
use std::time::{ Duration };


use rppal::gpio::{ Gpio, Trigger, OutputPin, InputPin, Level };
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::i2c::I2c;
use rppal::uart::{ Uart, Parity };

use m95320::prelude::*;
use m95320::m95320::Flash;

use port_expander::{ Pca9555, Pcf8574 };

use crate::epaper_display::GDEW029T5DController;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
enum GpioExpander {
  EXPANDER0,
  EXPANDER1,
}

#[derive(Copy, Clone, Debug)]
struct VirtualPinAddress(GpioExpander, usize);

#[derive(Copy, Clone, Debug)]
struct Well {
  reset_pin: VirtualPinAddress,
  busy_pin: VirtualPinAddress,
  data_or_command_pin: u8,
  epd_chip_select_pin: VirtualPinAddress,
  memory_chip_select_pin: VirtualPinAddress,
}

pub trait Plinth {
  fn display_image(&self, well: usize, image: Vec<u8>);
  fn read_memory(&self, well: usize, buffer: &mut [u8]) -> Result<(), String>;
  fn write_memory(&self, well: usize, buffer: &mut [u8]) -> Result<(), String>;
  fn set_switch_callback(&mut self, well: usize, switch: char, callback: impl FnMut(Level) + Send + 'static) -> Result<(), String>;
}

pub struct DevKitV1 {
  gpio_expander_addresses: HashMap<GpioExpander, (bool, bool, bool)>,
  wyldcard_wells: [Well; 4],
  uart: Uart,
  switch_names: HashMap<u8, String>,
  switches: HashMap<(usize, char), InputPin>,
}

impl Plinth for DevKitV1 {
  fn display_image(&self, well: usize, image: Vec<u8>) {
    let pin_assignments = self.wyldcard_wells[well];

    let gpio = Gpio::new().unwrap();
    let data_or_command = gpio.get(pin_assignments.data_or_command_pin).unwrap().into_output();

    let i2c = I2c::new().unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    // we're assuming all pins for a card are using the same expander
    let expander_address = self.gpio_expander_addresses[&pin_assignments.busy_pin.0];
    let mut expander = Pca9555::new(i2c, expander_address.0, expander_address.1, expander_address.2);
    let virtual_gpios = expander.split();

    let mut pins = HashMap::from([
      (0, virtual_gpios.io0_0),
      (1, virtual_gpios.io0_1),
      (2, virtual_gpios.io0_2),
      (3, virtual_gpios.io0_3),
      (4, virtual_gpios.io0_4),
      (5, virtual_gpios.io0_5),
      (6, virtual_gpios.io0_6),
      (7, virtual_gpios.io0_7),
      (8, virtual_gpios.io1_0),
      (9, virtual_gpios.io1_1),
      (10, virtual_gpios.io1_2),
      (11, virtual_gpios.io1_3),
      (12, virtual_gpios.io1_4),
      (13, virtual_gpios.io1_5),
      (14, virtual_gpios.io1_6),
      (15, virtual_gpios.io1_7),
    ]);

    let reset = pins.remove(&pin_assignments.reset_pin.1).expect("missing pin").into_output().expect("reset pin");
    let busy = pins.remove(&pin_assignments.busy_pin.1).expect("missing pin").into_input().expect("busy pin");
    let epd_chip_select = pins.remove(&pin_assignments.epd_chip_select_pin.1).expect("missing pin").into_output().expect("cs pin");

    let mut display = GDEW029T5DController::new(
                      reset,
                      busy,
                      data_or_command,
                      epd_chip_select,
                      spi).expect("create epd");

    if let Err(e) = display.start_epd() {
      println!("{}", e); return;
    }
    display.display_image(image);
    display.sleep();
  }

  fn read_memory(&self, well: usize, buffer: &mut [u8]) -> Result<(), String> {
    let pin_assignments = self.wyldcard_wells[well];

    let i2c = I2c::new().unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    // we're assuming all pins for a card are using the same expander
    let expander_address = self.gpio_expander_addresses[&pin_assignments.busy_pin.0];
    let mut expander = Pca9555::new(i2c, expander_address.0, expander_address.1, expander_address.2);
    let virtual_gpios = expander.split();

    let mut pins = HashMap::from([
      (0, virtual_gpios.io0_0),
      (1, virtual_gpios.io0_1),
      (2, virtual_gpios.io0_2),
      (3, virtual_gpios.io0_3),
      (4, virtual_gpios.io0_4),
      (5, virtual_gpios.io0_5),
      (6, virtual_gpios.io0_6),
      (7, virtual_gpios.io0_7),
      (8, virtual_gpios.io1_0),
      (9, virtual_gpios.io1_1),
      (10, virtual_gpios.io1_2),
      (11, virtual_gpios.io1_3),
      (12, virtual_gpios.io1_4),
      (13, virtual_gpios.io1_5),
      (14, virtual_gpios.io1_6),
      (15, virtual_gpios.io1_7),
    ]);

    let memory_chip_select = pins.remove(&pin_assignments.memory_chip_select_pin.1).expect("missing pin").into_output().expect("mem chip select");

    let mut flash = Flash::init(
                    spi,
                    memory_chip_select,
                  ).expect("memory");

    let result = flash.read(0, buffer);

    match result {
      Err(e) => return Err(e.to_string()),
      Ok(_) => return Ok(()),
    }
  }

  fn write_memory(&self, well: usize, buffer: &mut [u8]) -> Result<(), String> {
    let pin_assignments = self.wyldcard_wells[well];

    let i2c = I2c::new().unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    // we're assuming all pins for a card are using the same expander
    let expander_address = self.gpio_expander_addresses[&pin_assignments.busy_pin.0];
    let mut expander = Pca9555::new(i2c, expander_address.0, expander_address.1, expander_address.2);
    let virtual_gpios = expander.split();

    let mut pins = HashMap::from([
      (0, virtual_gpios.io0_0),
      (1, virtual_gpios.io0_1),
      (2, virtual_gpios.io0_2),
      (3, virtual_gpios.io0_3),
      (4, virtual_gpios.io0_4),
      (5, virtual_gpios.io0_5),
      (6, virtual_gpios.io0_6),
      (7, virtual_gpios.io0_7),
      (8, virtual_gpios.io1_0),
      (9, virtual_gpios.io1_1),
      (10, virtual_gpios.io1_2),
      (11, virtual_gpios.io1_3),
      (12, virtual_gpios.io1_4),
      (13, virtual_gpios.io1_5),
      (14, virtual_gpios.io1_6),
      (15, virtual_gpios.io1_7),
    ]);

    let memory_chip_select = pins.remove(&pin_assignments.memory_chip_select_pin.1).expect("missing pin").into_output().expect("mem chip select");

    let mut flash = Flash::init(
                    spi,
                    memory_chip_select,
                  ).expect("memory");

    let result = flash.write_bytes(0, buffer);

    match result {
      Err(e) => return Err(e.to_string()),
      Ok(_) => return Ok(()),
    }
  }

  // switches are identified by well number and then switch 'a', 'b', or 'c'
  fn set_switch_callback(&mut self, well: usize, switch: char, callback: impl FnMut(Level) + Send + 'static) -> Result<(), String> {
    if !['a','b','c'].contains(&switch) {
      return Err(String::from("Invalid switch"));
    }

    let switch = self.switches.get_mut(&(well, switch)).unwrap();

    switch.set_async_interrupt(Trigger::RisingEdge, callback).expect("set switch");

    Ok(())
  }
}

impl DevKitV1 {
  pub fn new() -> DevKitV1 {

    let gpio_expander_addresses = HashMap::from([
      (GpioExpander::EXPANDER0, (false, false, false)),
    ]);

    let wyldcard_wells = [
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 0),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 1),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 2),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 3),
      },
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 4),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 5),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 6),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 7),
      },
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 8),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 9),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 10),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 11),
      },
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 12),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 13),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 14),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 15),
      }
    ];

    let mut uart = Uart::new(9600, Parity::Even, 8, 1).expect("uart");
    uart.set_write_mode(true).expect("uart");
    uart.set_read_mode(0, Duration::from_millis(100)).expect("uart");


    let gpio = Gpio::new().unwrap();

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

    // setup switches
    let switch_a0 = gpio.get(GPIO_SWITCH_A0).unwrap().into_input_pullup();
    let switch_b0 = gpio.get(GPIO_SWITCH_B0).unwrap().into_input_pullup();
    let switch_c0 = gpio.get(GPIO_SWITCH_C0).unwrap().into_input_pullup();
    let switch_a1 = gpio.get(GPIO_SWITCH_A1).unwrap().into_input_pullup();
    let switch_b1 = gpio.get(GPIO_SWITCH_B1).unwrap().into_input_pullup();
    let switch_c1 = gpio.get(GPIO_SWITCH_C1).unwrap().into_input_pullup();
    let switch_a2 = gpio.get(GPIO_SWITCH_A2).unwrap().into_input_pullup();
    let switch_b2 = gpio.get(GPIO_SWITCH_B2).unwrap().into_input_pullup();
    let switch_c2 = gpio.get(GPIO_SWITCH_C2).unwrap().into_input_pullup();
    let switch_a3 = gpio.get(GPIO_SWITCH_A3).unwrap().into_input_pullup();
    let switch_b3 = gpio.get(GPIO_SWITCH_B3).unwrap().into_input_pullup();
    let switch_c3 = gpio.get(GPIO_SWITCH_C3).unwrap().into_input_pullup();
    
    let switch_names = HashMap::from([
      (GPIO_SWITCH_A0, String::from("Switch A for Card 0")),
      (GPIO_SWITCH_B0, String::from("Switch B for Card 0")),
      (GPIO_SWITCH_C0, String::from("Switch C for Card 0")),
      (GPIO_SWITCH_A1, String::from("Switch A for Card 1")),
      (GPIO_SWITCH_B1, String::from("Switch B for Card 1")),
      (GPIO_SWITCH_C1, String::from("Switch C for Card 1")),
      (GPIO_SWITCH_A2, String::from("Switch A for Card 2")),
      (GPIO_SWITCH_B2, String::from("Switch B for Card 2")),
      (GPIO_SWITCH_C2, String::from("Switch C for Card 2")),
      (GPIO_SWITCH_A3, String::from("Switch A for Card 3")),
      (GPIO_SWITCH_B3, String::from("Switch B for Card 3")),
      (GPIO_SWITCH_C3, String::from("Switch C for Card 3")),
    ]);

    DevKitV1 {
      gpio_expander_addresses,
      wyldcard_wells,
      uart,
      switch_names,
      switches: HashMap::from([
        ((0, 'a'), switch_a0),
        ((0, 'b'), switch_b0),
        ((0, 'c'), switch_c0),
        ((1, 'a'), switch_a1),
        ((1, 'b'), switch_b1),
        ((1, 'c'), switch_c1),
        ((2, 'a'), switch_a2),
        ((2, 'b'), switch_b2),
        ((2, 'c'), switch_c2),
        ((3, 'a'), switch_a3),
        ((3, 'b'), switch_b3),
        ((3, 'c'), switch_c3),
      ]),
    }
  }
}

pub struct Prototype {
  gpio_expander_addresses: HashMap<GpioExpander, (bool, bool, bool)>,
  wyldcard_wells: [Well; 4],
  uart: Uart,
  switch_names: HashMap<u8, String>,
  switches: HashMap<(usize, char), InputPin>,
}

impl Plinth for Prototype {
  fn display_image(&self, well: usize, image: Vec<u8>) {
    println!("image vec size is {}", image.len());
    let pin_assignments = self.wyldcard_wells[well];

    let gpio = Gpio::new().unwrap();
    let data_or_command = gpio.get(pin_assignments.data_or_command_pin).unwrap().into_output();

    let i2c = I2c::new().unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    // we're assuming all pins for a card are using the same expander
    let expander_address = self.gpio_expander_addresses[&pin_assignments.busy_pin.0];
    let mut expander = Pcf8574::new(i2c, expander_address.0, expander_address.1, expander_address.2);
    let virtual_gpios = expander.split();

    let mut pins = HashMap::from([
      (0, virtual_gpios.p0),
      (1, virtual_gpios.p1),
      (2, virtual_gpios.p2),
      (3, virtual_gpios.p3),
      (4, virtual_gpios.p4),
      (5, virtual_gpios.p5),
      (6, virtual_gpios.p6),
      (7, virtual_gpios.p7),
    ]);

    let reset = pins.remove(&pin_assignments.reset_pin.1).expect("reset pin");
    let busy = pins.remove(&pin_assignments.busy_pin.1).expect("busy pin");
    let epd_chip_select = pins.remove(&pin_assignments.epd_chip_select_pin.1).expect("cs pin");

    let mut display = GDEW029T5DController::new(
                      reset,
                      busy,
                      data_or_command,
                      epd_chip_select,
                      spi).expect("create epd");

    if let Err(e) = display.start_epd() {
      println!("{}", e); return;
    }
    display.display_image(image);
    display.sleep();
  }

  fn read_memory(&self, well: usize, buffer: &mut [u8]) -> Result<(), String> {
    let pin_assignments = self.wyldcard_wells[well];

    let i2c = I2c::new().unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    // we're assuming all pins for a card are using the same expander
    let expander_address = self.gpio_expander_addresses[&pin_assignments.busy_pin.0];
    let mut expander = Pcf8574::new(i2c, expander_address.0, expander_address.1, expander_address.2);
    let virtual_gpios = expander.split();

    let mut pins = HashMap::from([
      (0, virtual_gpios.p0),
      (1, virtual_gpios.p1),
      (2, virtual_gpios.p2),
      (3, virtual_gpios.p3),
      (4, virtual_gpios.p4),
      (5, virtual_gpios.p5),
      (6, virtual_gpios.p6),
      (7, virtual_gpios.p7),
    ]);

    let memory_chip_select = pins.remove(&pin_assignments.memory_chip_select_pin.1).expect("mem chip select");

    let mut flash = Flash::init(
                    spi,
                    memory_chip_select,
                  ).expect("memory");

    let result = flash.read(0, buffer);

    match result {
      Err(e) => return Err(e.to_string()),
      Ok(_) => return Ok(()),
    }
  }

  fn write_memory(&self, well: usize, buffer: &mut [u8]) -> Result<(), String> {
    let pin_assignments = self.wyldcard_wells[well];

    let i2c = I2c::new().unwrap();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    // we're assuming all pins for a card are using the same expander
    let expander_address = self.gpio_expander_addresses[&pin_assignments.busy_pin.0];
    let mut expander = Pcf8574::new(i2c, expander_address.0, expander_address.1, expander_address.2);
    let virtual_gpios = expander.split();

    let mut pins = HashMap::from([
      (0, virtual_gpios.p0),
      (1, virtual_gpios.p1),
      (2, virtual_gpios.p2),
      (3, virtual_gpios.p3),
      (4, virtual_gpios.p4),
      (5, virtual_gpios.p5),
      (6, virtual_gpios.p6),
      (7, virtual_gpios.p7),
    ]);

    let memory_chip_select = pins.remove(&pin_assignments.memory_chip_select_pin.1).expect("mem chip select");

    let mut flash = Flash::init(
                    spi,
                    memory_chip_select,
                  ).expect("memory");

    let result = flash.write_bytes(0, buffer);

    match result {
      Err(e) => return Err(e.to_string()),
      Ok(_) => return Ok(()),
    }
  }

  // switches are identified by well number and then switch 'a', 'b', or 'c'
  fn set_switch_callback(&mut self, well: usize, switch: char, callback: impl FnMut(Level) + Send + 'static) -> Result<(), String> {
    if !['a','b','c'].contains(&switch) {
      return Err(String::from("Invalid switch"));
    }

    let switch = self.switches.get_mut(&(well, switch)).unwrap();

    switch.set_async_interrupt(Trigger::RisingEdge, callback).expect("set switch");

    Ok(())
  }
}

impl Prototype {
  pub fn new() -> Prototype {

    let gpio_expander_addresses = HashMap::from([
      (GpioExpander::EXPANDER0, (false, false, false)),
      (GpioExpander::EXPANDER1, (true, false, false)),
    ]);

    let wyldcard_wells = [
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 0),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 1),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 2),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 3),
      },
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 4),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 5),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 6),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER0, 7),
      },
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 0),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 1),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 2),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 3),
      },
      Well {
        reset_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 4),
        busy_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 5),
        data_or_command_pin: 25,
        epd_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 6),
        memory_chip_select_pin: VirtualPinAddress(GpioExpander::EXPANDER1, 7),
      }
    ];

    let mut uart = Uart::new(9600, Parity::Even, 8, 1).expect("uart");
    uart.set_write_mode(true).expect("uart");
    uart.set_read_mode(0, Duration::from_millis(100)).expect("uart");


    let gpio = Gpio::new().unwrap();

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

    // setup switches
    let switch_a0 = gpio.get(GPIO_SWITCH_A0).unwrap().into_input_pullup();
    let switch_b0 = gpio.get(GPIO_SWITCH_B0).unwrap().into_input_pullup();
    let switch_c0 = gpio.get(GPIO_SWITCH_C0).unwrap().into_input_pullup();
    let switch_a1 = gpio.get(GPIO_SWITCH_A1).unwrap().into_input_pullup();
    let switch_b1 = gpio.get(GPIO_SWITCH_B1).unwrap().into_input_pullup();
    let switch_c1 = gpio.get(GPIO_SWITCH_C1).unwrap().into_input_pullup();
    let switch_a2 = gpio.get(GPIO_SWITCH_A2).unwrap().into_input_pullup();
    let switch_b2 = gpio.get(GPIO_SWITCH_B2).unwrap().into_input_pullup();
    let switch_c2 = gpio.get(GPIO_SWITCH_C2).unwrap().into_input_pullup();
    let switch_a3 = gpio.get(GPIO_SWITCH_A3).unwrap().into_input_pullup();
    let switch_b3 = gpio.get(GPIO_SWITCH_B3).unwrap().into_input_pullup();
    let switch_c3 = gpio.get(GPIO_SWITCH_C3).unwrap().into_input_pullup();
    
    let switch_names = HashMap::from([
      (GPIO_SWITCH_A0, String::from("Switch A for Card 0")),
      (GPIO_SWITCH_B0, String::from("Switch B for Card 0")),
      (GPIO_SWITCH_C0, String::from("Switch C for Card 0")),
      (GPIO_SWITCH_A1, String::from("Switch A for Card 1")),
      (GPIO_SWITCH_B1, String::from("Switch B for Card 1")),
      (GPIO_SWITCH_C1, String::from("Switch C for Card 1")),
      (GPIO_SWITCH_A2, String::from("Switch A for Card 2")),
      (GPIO_SWITCH_B2, String::from("Switch B for Card 2")),
      (GPIO_SWITCH_C2, String::from("Switch C for Card 2")),
      (GPIO_SWITCH_A3, String::from("Switch A for Card 3")),
      (GPIO_SWITCH_B3, String::from("Switch B for Card 3")),
      (GPIO_SWITCH_C3, String::from("Switch C for Card 3")),
    ]);

    Prototype {
      gpio_expander_addresses,
      wyldcard_wells,
      uart,
      switch_names,
      switches: HashMap::from([
        ((0, 'a'), switch_a0),
        ((0, 'b'), switch_b0),
        ((0, 'c'), switch_c0),
        ((1, 'a'), switch_a1),
        ((1, 'b'), switch_b1),
        ((1, 'c'), switch_c1),
        ((2, 'a'), switch_a2),
        ((2, 'b'), switch_b2),
        ((2, 'c'), switch_c2),
        ((3, 'a'), switch_a3),
        ((3, 'b'), switch_b3),
        ((3, 'c'), switch_c3),
      ]),
    }
  }
}