use std::io::{ Error };
use std::thread;
use std::time::{Duration, Instant};

use embedded_hal::digital::v2::{ InputPin, OutputPin };
use embedded_hal::blocking::spi::Transfer;

const IDLE_TIMEOUT: Duration = Duration::new(6, 0);

// resolution: 128 x 296
pub struct GDEY029T94Controller<SPI, RESET, BUSY, DC, CS> {
    reset: RESET,
    busy: BUSY,
    data_or_command: DC,
    chip_select: CS,
    spi: SPI,
    color_resolution: ColorResolution,
}

struct WaveformLut {
    lut: [u8; 153],
}

enum ColorResolution {
    BlackAndWhiteMonochrome,
    FourColorGrayscale,
}

impl<SPI, RESET, BUSY, DC, CS, E> GDEY029T94Controller<SPI, RESET, BUSY, DC, CS>
    where SPI: Transfer<u8, Error = E>,
          RESET: OutputPin,
          BUSY: InputPin,
          BUSY::Error: std::fmt::Debug,
          DC: OutputPin,
          CS: OutputPin,
          E: std::fmt::Debug {
            
    fn reset_ic(&mut self) {
        println!("toggling hardware reset pin");
        self.reset.set_low();
        thread::sleep(Duration::from_millis(10));
        self.reset.set_high();
        thread::sleep(Duration::from_millis(10));
    }

    fn wait_for_idle(&mut self) -> Duration {
        println!("waiting for idle signal");
        const IDLE_CHECK_FREQUENCY: Duration = Duration::from_millis(10);
        let start = Instant::now();
        while self.busy.is_high().expect("read busy pin") && start.elapsed() < IDLE_TIMEOUT {
            thread::sleep(IDLE_CHECK_FREQUENCY);
        }
        let wait_duration = start.elapsed();
        println!("epd was busy for {:?}", wait_duration);
        wait_duration
    }
    
    fn write_command(&mut self, command: &[u8]) {
        let command: &mut [u8] = &mut command.to_owned();
        self.chip_select.set_low();

        self.data_or_command.set_low();
        self.spi.transfer(command).expect("error with spi write");

        self.chip_select.set_high();
    }
    
    fn write_data(&mut self, data: &[u8]) {
        let data = &mut data.to_owned();
        self.chip_select.set_low();

        self.data_or_command.set_high();
        self.spi.transfer(data).expect("error with spi write");

        self.chip_select.set_high();
    }

    // sends a command, followed by data. This uses one less toggle of the Chip Select gpio
    // which increases our performance (since we're behind a gpio expander)
    fn write_command_with_data(&mut self, command: &[u8], data: &[u8]) {
        let command = &mut command.to_owned();
        let data = &mut data.to_owned();
        self.chip_select.set_low();

        self.data_or_command.set_low();
        self.spi.transfer(command).expect("error with spi write");

        self.data_or_command.set_high();
        self.spi.transfer(data).expect("error with spi write");

        self.chip_select.set_high();
    }

    pub fn start_epd(&mut self) -> Result<(), Error> {
        match self.color_resolution {
            ColorResolution::BlackAndWhiteMonochrome => self.init_epd_monochrome(),
            ColorResolution::FourColorGrayscale => self.init_epd_4_color_grayscale(),
        }
    }

    fn init_epd_monochrome(&mut self) -> Result<(), Error> {
        // println!("i wonder if we start busy? we do!");
        // self.wait_for_idle();

        self.reset_ic();
        self.wait_for_idle();

        self.write_command(&[0x12]);  //software reset
        self.wait_for_idle();

        self.write_command(&[0x01]); //Driver output control
        self.write_data(&[0x27]); //TODO these can be combined into one data write?
        self.write_data(&[0x01]);
        self.write_data(&[0x00]); // number of MUX lines, how they're arranged...

        self.write_command_with_data(&[0x11], &[0x01]); // data entry mode (decrement ram pixel counter, zigzag)

        self.write_command(&[0x44]); // set RAM X address start/end position
        self.write_data(&[0x00]);
        self.write_data(&[0x0F]);    // #0x0F-->(15+1)*8=128

        self.write_command(&[0x45]); // set RAM Y address start/end position
        self.write_data(&[0x27]);    // 0x0127-->(295+1)=296
        self.write_data(&[0x01]);
        self.write_data(&[0x00]);    // 0x0127-->(295+1)=296
        self.write_data(&[0x00]);

        self.write_command_with_data(&[0x3C], &[0x05]);  // border waveform, follow LUT0

        self.write_command(&[0x21]); // settings for updating the display, "Update Display Control". normal black and white and red, source from S8 to S167
        self.write_data(&[0x00]);
        self.write_data(&[0x80]);

        self.write_command_with_data(&[0x18], &[0x80]); // read temperature from built-in temperature sensor

        self.write_command(&[0x4E]); // set RAM x address count to 0
        self.write_data(&[0x00]);

        self.write_command(&[0x4F]); // set RAM y address count to 0x127
        self.write_data(&[0x27]);
        self.write_data(&[0x01]);

        self.wait_for_idle();

        Ok(())
    }

    fn init_epd_4_color_grayscale(&mut self) -> Result<(), Error> {
        // println!("i wonder if we start busy? we do!");
        // self.wait_for_idle();

        self.reset_ic();
        self.wait_for_idle();

        self.write_command(&[0x12]);  //software reset
        self.wait_for_idle();

        // these commands are undocumented and may or may not do anything
        self.write_command(&[0x74]); // set analog block control
        self.write_data(&[0x54]);
        self.write_command(&[0x7E]); // set digital block control
        self.write_data(&[0x3B]);

        self.write_command(&[0x01]); //Driver output control
        self.write_data(&[0x27]); //TODO these can be combined into one data write?
        self.write_data(&[0x01]);
        self.write_data(&[0x00]); // number of MUX lines, how they're arranged...

        self.write_command_with_data(&[0x11], &[0x03]); // data entry mode (decrement ram pixel counter, zigzag)

        self.write_command(&[0x44]); // set RAM X address start/end position
        self.write_data(&[0x00]);
        self.write_data(&[0x0F]);    // #0x0F-->(15+1)*8=128

        self.write_command(&[0x45]); // set RAM Y address start/end position
        self.write_data(&[0x00]);    // 0x0127-->(295+1)=296
        self.write_data(&[0x00]);
        self.write_data(&[0x27]);    // 0x0127-->(295+1)=296
        self.write_data(&[0x01]);

        self.write_command_with_data(&[0x3C], &[0x00]);  // border waveform

        self.write_command_with_data(&[0x2C], &[0x30]);  // VCOM Voltage

        self.write_command_with_data(&[0x3F], &[0x22]);  // EOPQ, Option for LUT end: 'normal'

        self.write_command_with_data(&[0x03], &[0x15]);  // VGH, 19V

        self.write_command(&[0x04]);  // source driving voltage control
        self.write_data(&[0x41]);     // VSH1: 15V
        self.write_data(&[0xA8]);     // VSH2: 5V
        self.write_data(&[0x32]);     // VSL : -15V

        self.write_command(&[0x32]);  // write LUT
        self.write_data(&get_4_grayscale_lut().lut);

        self.write_command(&[0x21]); // settings for updating the display, "Update Display Control"
        self.write_data(&[0x88]);    // invert black and white, invert red
        self.write_data(&[0x80]);    // source from S8 to S167

        self.write_command_with_data(&[0x18], &[0x80]); // read temperature from built-in temperature sensor

        self.write_command(&[0x4E]); // set RAM x address count to 0
        self.write_data(&[0x00]);

        self.write_command(&[0x4F]); // set RAM y address count to 0x127
        self.write_data(&[0x27]);
        self.write_data(&[0x01]);

        self.wait_for_idle();

        Ok(())
    }

    fn update_display(&mut self) {
        match self.color_resolution {
            ColorResolution::BlackAndWhiteMonochrome => self.update_display_monochrome(),
            ColorResolution::FourColorGrayscale => self.update_display_4_color_grayscale(),
        }
    }

    // run the update sequence, displaying contents of EPD RAM
    fn update_display_monochrome(&mut self) {
        println!("updating display, using contents of RAM");

        self.write_command_with_data(&[0x22], &[0xF7]); // update control: enable clock signal, enable analog, load temperature value, display with DISPLAY Mode 1, disable analog, disable OSC
        self.write_command(&[0x20]); // Master Activation, activate display update sequence

        self.wait_for_idle();
    }

    // run the update sequence, displaying contents of EPD RAM
    fn update_display_4_color_grayscale(&mut self) {
        println!("updating display, using contents of RAM");

        self.write_command_with_data(&[0x22], &[0xC7]); // update control: enable clock signal, enable analog, load temperature value, display with DISPLAY Mode 1, disable analog, disable OSC
        self.write_command(&[0x20]); // Master Activation, activate display update sequence

        self.wait_for_idle();
    }

    #[allow(dead_code)]
    fn display_single_color_image(&mut self, black: bool) {
        println!("displaying single color image");
        self.wait_for_idle();
        

        let pixel: &[u8];

        match black {
            true => { pixel = &[0x00]},
            false => { pixel = &[0xff]},
        }

        self.write_command(&[0x24]);  // write to Black and White RAM. 0 black, 1 white
        for _ in 0..4736 {
            self.write_data(pixel);
        }

        self.update_display();
    }

    #[allow(dead_code)]
    pub fn display_alternating_pixel_grid(&mut self) {
        self.wait_for_idle();
        println!("displaying alternating pixel grid");

        self.write_command(&[0x24]);  // write to Black and White RAM. 0 black, 1 white
        for _ in 0..296/2 {
            for _ in 0..128/8 {
                self.write_data(&[0b01010101]);
            }
            for _ in 0..128/8 {
                self.write_data(&[0b10101010]);
            }
        }
        
        self.update_display();
    }

    #[allow(dead_code)]
    pub fn display_random_static(&mut self) {
        // self.wait_for_idle();
        // println!("displaying alternating pixel grid");

        // let mut rng = rand::thread_rng();

        // self.write_command(&[0x10]);
        // for _ in 0..296 {
        //     for _ in 0..128/8 {
        //         self.write_data(&[rng.gen()]);
        //     }
        // }
        // self.write_command(&[0x13]);
        // for _ in 0..296 {
        //     for _ in 0..128/8 {
        //         self.write_data(&[rng.gen()]);
        //     }
        // }
    
        // self.write_command(&[0x12]);    
        // thread::sleep(Duration::from_millis(10));
        // self.wait_for_idle();
    }

    #[allow(dead_code)]
    pub fn display_four_color_image(&mut self) {
        self.wait_for_idle();
        println!("displaying 4 color grayscale image");

        self.write_command(&[0x24]);  // write to Black and White RAM
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

        self.write_command(&[0x26]);  // write to Red RAM
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
    
        self.update_display();
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

            old_data[i/2] = even_bits;
            new_data[i/2] = odd_bits; 
        }

        self.write_command_with_data(&[0x24], &old_data);

        self.write_command_with_data(&[0x26], &new_data);
    
        self.update_display();
    }

    pub fn sleep(&mut self) {
        self.wait_for_idle();
        println!("putting display to sleep");

        self.write_command_with_data(&[0x10], &[0x01]);

        thread::sleep(Duration::from_millis(10));
    }

    pub fn new(reset: RESET, busy: BUSY, data_or_command: DC, mut chip_select: CS, spi: SPI) -> Result<Self, E> {
        
        let color_resolution = ColorResolution::FourColorGrayscale;

        chip_select.set_high();

        let display = GDEY029T94Controller {
            reset,
            busy,
            data_or_command,
            chip_select,
            spi,
            color_resolution,
        };

        Ok(display)
    }
}

fn get_4_grayscale_lut() -> WaveformLut {
    WaveformLut{
        lut: [
            0x50,0x90,0xA0,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x40,0x90,0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x80,0x90,0x40,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0xA0,0x90,0x50,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x01,0x09,0x00,0x00,0x00,0x00,0x00,
            0x02,0x02,0x13,0x00,0x00,0x00,0x00,
            0x01,0x09,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,
            0x22,0x22,0x22,0x22,0x22,0x22,0x00,0x00,0x00,
        ]
    }
}