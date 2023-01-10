use std::io::{Error, ErrorKind, self};
use std::thread;
use std::time::{Duration, Instant};
use std::rc::Rc;
use std::cell::RefCell;
use rand::Rng;

use embedded_hal::digital::v2::{ InputPin, OutputPin };

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};


// const SPI_CLOCK: u8 = 11;
// const SPI_MOSI: u8 = 10;
// const SPI_MISO: u8 = 9; // unused, a rare read operation may go over the MOSI connection


// resolution: 128 x 296
pub struct GDEW029T5DController<'a> {
    reset: &'a mut dyn OutputPin<Error = pcf857x::Error<rppal::i2c::Error>>,
    busy: &'a dyn InputPin<Error = pcf857x::Error<rppal::i2c::Error>>,
    data_or_command: Rc<RefCell<dyn OutputPin<Error = rppal::gpio::Error>>>,
    chip_select: &'a mut dyn OutputPin<Error = pcf857x::Error<rppal::i2c::Error>>,
    spi: Spi,
}

struct WaveformLut {
    vcom: [u8; 44],
    ww: [u8; 42],
    bw: [u8; 42],
    wb: [u8; 42],
    bb: [u8; 42],
}

impl<'a> GDEW029T5DController<'a> {
    fn reset_ic(&mut self) {
        println!("setting reset pin low, three times");
        for _ in 0..3 { 
            self.reset.set_low();
            thread::sleep(Duration::from_millis(10));
            self.reset.set_high();
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_idle(&mut self) -> Duration {
        println!("waiting for idle signal");
        let start = Instant::now();
        while self.busy.is_low().expect("read busy pin") {
            thread::sleep(Duration::from_millis(10));
        }
        let wait_duration = start.elapsed();
        println!("epd was busy for {:?}", wait_duration);
        wait_duration
    }
    
    fn write_command(&mut self, command: &[u8]) {
        self.chip_select.set_low();

        self.data_or_command.borrow_mut().set_low();
        self.spi.write(command).expect("error with spi write");

        self.chip_select.set_high();
    }
    
    fn write_data(&mut self, data: &[u8]) {
        self.chip_select.set_low();

        self.data_or_command.borrow_mut().set_high();
        self.spi.write(data).expect("error with spi write");

        self.chip_select.set_high();
    }

    // sends a command, followed by data. This uses one less toggle of the Chip Select gpio
    // which increases our performance (since we're behind a gpio expander)
    fn write_command_with_data(&mut self, command: &[u8], data: &[u8]) {
        self.chip_select.set_low();

        self.data_or_command.borrow_mut().set_low();
        self.spi.write(command).expect("error with spi write");

        self.data_or_command.borrow_mut().set_high();
        self.spi.write(data).expect("error with spi write");

        self.chip_select.set_high();
    }

    pub fn start_epd(&mut self) -> Result<(), Error> {
        self.chip_select.set_high();
        
        self.reset_ic();

        // stuff added for custom lut:

        self.write_command_with_data(&[0x06], &vec![0x17, 0x17, 0x17]); // booster soft start 

        self.write_command_with_data(&[0x01], &vec![0x03, 0x00, 0x2b, 0x2b, 0x13]); // power setting

        // end extra lut stuff (not sure if needed? or can use all the time?)
    
        println!("sending Power On command");
        self.write_command(&[0x04]);
        let power_on_time = self.wait_for_idle();
        if power_on_time < Duration::from_millis(50) {
            return Err(Error::new(ErrorKind::NotConnected, "Power On command returned in less than 50ms, card is probably not present"));
        }
    
        println!("setting up epd");
        self.write_command_with_data(&[0x00], &[0b1011_1111]); // we might want the first bit to be `1`, for our resolution, but the example has it as zero
        // nevermind, it's overridden by the next command. meaning the next command could be superflous?
        // third bit flipped for custom lut!!

        // self.write_command(&[0x61]); // resolution setting not needed, since we specified in panel setup
        // self.write_data(&[0x80]);
        // self.write_data(&[0x01]);
        // self.write_data(&[0x28]);

        // also added for custom lut
        self.write_command_with_data(&[0x82], &[0x12]); // vcom_DC setting
    
        self.write_command_with_data(&[0x50], &[0x97]); // vcom and data settings

        Ok(())
    }

    #[allow(dead_code)]
    fn display_single_color_image(&mut self, black: bool) {
        self.wait_for_idle();
        println!("displaying default image");

        let pixel: &[u8];

        match black {
            true => { pixel = &[0x00]},
            false => { pixel = &[0xff]},
        }

        self.write_command(&[0x10]);
        for _ in 0..4736 {
            self.write_data(pixel);
        }
        self.write_command(&[0x13]);
        for _ in 0..4736 {
            self.write_data(pixel);
        }
    
        self.write_command(&[0x12]);    
        thread::sleep(Duration::from_millis(10));
        self.wait_for_idle();
    }

    #[allow(dead_code)]
    pub fn display_alternating_pixel_grid(&mut self) {
        self.wait_for_idle();
        println!("displaying alternating pixel grid");


        self.write_command(&[0x10]);
        for _ in 0..296/2 {
            for _ in 0..128/8 {
                self.write_data(&[0b01010101]);
            }
            for _ in 0..128/8 {
                self.write_data(&[0b10101010]);
            }
        }
        self.write_command(&[0x13]);
        for _ in 0..296/2 {
            for _ in 0..128/8 {
                self.write_data(&[0b01010101]);
            }
            for _ in 0..128/8 {
                self.write_data(&[0b10101010]);
            }
        }
    
        self.write_command(&[0x12]);    
        thread::sleep(Duration::from_millis(10));
        self.wait_for_idle();
    }

    #[allow(dead_code)]
    pub fn display_random_static(&mut self) {
        self.wait_for_idle();
        println!("displaying alternating pixel grid");

        let mut rng = rand::thread_rng();

        self.write_command(&[0x10]);
        for _ in 0..296 {
            for _ in 0..128/8 {
                self.write_data(&[rng.gen()]);
            }
        }
        self.write_command(&[0x13]);
        for _ in 0..296 {
            for _ in 0..128/8 {
                self.write_data(&[rng.gen()]);
            }
        }
    
        self.write_command(&[0x12]);    
        thread::sleep(Duration::from_millis(10));
        self.wait_for_idle();
    }

    #[allow(dead_code)]
    pub fn display_four_color_image(&mut self) {
        self.wait_for_idle();
        println!("displaying 4 color grayscale image");

        self.load_lut(GDEW029T5DController::get_4_grayscale_lut());

        self.write_command(&[0x10]);
        for _ in 0..(4736/4) {
            self.write_data(&[0xff]);
        }
        for _ in 0..(4736/4) {
            self.write_data(&[0xff]);
        }
        for _ in 0..(4736/4) {
            self.write_data(&[0x00]);
        }
        for _ in 0..(4736/4) {
            self.write_data(&[0x00]);
        }

        self.write_command(&[0x13]);
        for _ in 0..(4736/4) {
            self.write_data(&[0xff]);
        }
        for _ in 0..(4736/4) {
            self.write_data(&[0x00]);
        }
        for _ in 0..(4736/4) {
            self.write_data(&[0xff]);
        }
        for _ in 0..(4736/4) {
            self.write_data(&[0x00]);
        }
    
        self.write_command(&[0x12]);    
        thread::sleep(Duration::from_millis(10));
        self.wait_for_idle();
    }
    
    #[allow(dead_code)]
    pub fn display_black_image(&mut self) {
        self.display_single_color_image(true);
    }

    #[allow(dead_code)]
    pub fn display_white_image(&mut self) {
        self.display_single_color_image(false);
    }

    pub fn display_image(&mut self, buf: Vec<u8>) {
        self.wait_for_idle();
        println!("displaying image from input buffer");

        let mut old_data: [u8; 4736] = [0; 4736];
        let mut new_data: [u8; 4736] = [0; 4736];

        for (i, _) in buf.iter().enumerate().step_by(2) {
            let two_bytes = &buf[i..i+2];
            let mut sixteen_bits: u16 = ((two_bytes[0] as u16) << 8) | two_bytes[1] as u16;
            sixteen_bits = sixteen_bits.reverse_bits();

            let mut odd_bits: u8 = 0;
            let mut even_bits: u8 = 0;

            odd_bits = odd_bits | (sixteen_bits as u8 & 0b0000_0001);
            sixteen_bits = sixteen_bits >> 1;
            even_bits = even_bits | (sixteen_bits as u8 & 0b0000_0001);
            sixteen_bits = sixteen_bits >> 1;
            for _i in 0..7 {
                odd_bits = odd_bits << 1;
                odd_bits = odd_bits | (sixteen_bits as u8 & 0b0000_0001);
                sixteen_bits = sixteen_bits >> 1;
                even_bits = even_bits << 1;
                even_bits = even_bits | (sixteen_bits as u8 & 0b0000_0001);
                sixteen_bits = sixteen_bits >> 1;
            }

            old_data[i/2] = odd_bits;
            new_data[i/2] = even_bits; 
        }
        
        self.load_lut(GDEW029T5DController::get_4_grayscale_lut());

        self.write_command_with_data(&[0x10], &old_data);

        self.write_command_with_data(&[0x13], &new_data);
    
        self.write_command(&[0x12]);
        thread::sleep(Duration::from_millis(10));
        self.wait_for_idle();
    }

    fn get_4_grayscale_lut() -> WaveformLut {
        WaveformLut{
            vcom: [
                0x00	,0x0A	,0x00	,0x00	,0x00	,0x01,
                0x60	,0x14	,0x14	,0x00	,0x00	,0x01,
                0x00	,0x14	,0x00	,0x00	,0x00	,0x01,
                0x00	,0x13	,0x0A	,0x01	,0x00	,0x01,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00    ,0x00,
            ],
            ww: [
                0x40	,0x0A	,0x00	,0x00	,0x00	,0x01,
                0x90	,0x14	,0x14	,0x00	,0x00	,0x01,
                0x10	,0x14	,0x0A	,0x00	,0x00	,0x01,
                0xA0	,0x13	,0x01	,0x00	,0x00	,0x01,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
            ],
            bw: [
                0x40	,0x0A	,0x00	,0x00	,0x00	,0x01,
                0x90	,0x14	,0x14	,0x00	,0x00	,0x01,
                0x00	,0x14	,0x0A	,0x00	,0x00	,0x01,
                0x99	,0x0C	,0x01	,0x03	,0x04	,0x01,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
            ],
            wb: [
                0x40	,0x0A	,0x00	,0x00	,0x00	,0x01,
                0x90	,0x14	,0x14	,0x00	,0x00	,0x01,
                0x00	,0x14	,0x0A	,0x00	,0x00	,0x01,
                0x99	,0x0B	,0x04	,0x04	,0x01	,0x01,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,                
            ],
            bb: [
                0x80	,0x0A	,0x00	,0x00	,0x00	,0x01,
                0x90	,0x14	,0x14	,0x00	,0x00	,0x01,
                0x20	,0x14	,0x0A	,0x00	,0x00	,0x01,
                0x50	,0x13	,0x01	,0x00	,0x00	,0x01,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,
                0x00	,0x00	,0x00	,0x00	,0x00	,0x00,                
            ]
        }
    }

    fn load_lut(&mut self, lut: WaveformLut) {
        self.write_command_with_data(&[0x20], &lut.vcom);

        self.write_command_with_data(&[0x21], &lut.ww);

        self.write_command_with_data(&[0x22], &lut.bw);

        self.write_command_with_data(&[0x23], &lut.wb);

        self.write_command_with_data(&[0x24], &lut.bb);
    }

    pub fn sleep(&mut self) {
        self.wait_for_idle();
        println!("putting display to sleep");
        self.write_command_with_data(&[0x50], &[0xf7]);
    
        self.write_command(&[0x02]);
        self.wait_for_idle();
        self.write_command_with_data(&[0x07], &[0xA5]);
    }
}

pub fn init_epd<'a>(
    reset: &'a mut dyn OutputPin<Error = pcf857x::Error<rppal::i2c::Error>>,
    busy: &'a dyn InputPin<Error = pcf857x::Error<rppal::i2c::Error>>,
    data_or_command: Rc<RefCell<dyn OutputPin<Error = rppal::gpio::Error>>>,
    mut chip_select: &'a mut dyn OutputPin<Error = pcf857x::Error<rppal::i2c::Error>>
) -> GDEW029T5DController<'a> {
    println!("setting up pins/devices");

    chip_select.set_high();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0).unwrap();

    GDEW029T5DController {
        reset,
        busy,
        data_or_command,
        chip_select,
        spi,
    }
}