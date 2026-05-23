pub use embedded_cli::Command;
use stm32f4xx_hal::otg_fs::{USB, UsbBus};
use usbd_serial::SerialPort;
use core::convert::Infallible;
use embedded_cli::cli::Cli;
use ufmt::uwrite;

// Serial port writer
pub struct Writer {
    pub ser: *mut SerialPort<'static, UsbBus<USB>>
}

// Error implementation
impl embedded_io::ErrorType for Writer {
   type Error = Infallible;
}

impl embedded_io::Write for Writer {

    // Write some bytes over serial
    fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        unsafe { (*self.ser).write(buffer).ok(); };
        Ok(buffer.len())
    }

    // Flush the serial port
    fn flush(&mut self) -> Result<(), Self::Error> {
        unsafe { (*self.ser).flush().ok(); };
        Ok(())
    }
}

#[derive(Command)]
pub enum Base<'a> {
    
    Hello {
        name: Option<&'a str>,
    },
    
    Exit,
}

// cli/mod.rs
pub fn process(cli: &mut Cli<Writer, Infallible, &'static mut [u8], &'static mut [u8]>, byte: u8) {
    let _ = cli.process_byte::<Base, _>(
        byte,
        &mut Base::processor(|cli, command| {
            match command {
                Base::Hello { name } => {
                    uwrite!(cli.writer(), "Hello {}", name.unwrap_or("World"))?;
                }
                Base::Exit => {
                    cli.writer().write_str("Idk how to do allat").ok();
                }
            }
            Ok(())
        }),
    );
}


