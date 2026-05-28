pub use embedded_cli::{Command, CommandGroup};
use stm32f4xx_hal::otg_fs::{USB, UsbBus};
use usbd_serial::SerialPort;
use core::convert::Infallible;
use embedded_cli::cli::{Cli, CliHandle};
use ufmt::{uwrite, uwriteln};

// Serial port writer
pub struct Writer {
    pub ser: *mut SerialPort<'static, UsbBus<USB>>
}

impl ufmt::uWrite for Writer {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        unsafe { (*self.ser).write(s.as_bytes()).ok() };
        Ok(())
    }
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

struct PrintingState {

}

pub struct QuadCli {
    cli: Cli<Writer, Infallible, &'static mut [u8], &'static mut [u8]>,
    printing_state: PrintingState,
    last_print: Option<u32>
}

#[derive(Command)]
pub enum Base<'a> {

    /// Hello World
    Hello {
        name: Option<&'a str>,
    },

    /// Exit cli
    Exit,
}

#[derive(CommandGroup)]
enum Group<'a> {
    Base(Base<'a>),
}

impl QuadCli {

    pub fn new(cli: Cli<Writer, Infallible, &'static mut [u8], &'static mut [u8]>) -> Self {
        QuadCli {
            cli,
            printing_state: PrintingState {
            },
            last_print: Some(0),
        }
    }

    pub fn print_state(
        &mut self,
        ser: *mut SerialPort<'static, UsbBus<USB>>,
        now: u32
    ) {
        let mut w = Writer { ser };

        // Time to 1hz
        if let Some(last) = self.last_print {
            if now - last < 10000 { return; }
        }

        self.last_print = Some(now);

    }

    /** Base defined CLI commands */
    fn base_cmds(cmd: Base, cli: &mut CliHandle<Writer, Infallible>) {
        match cmd {
            Base::Hello { name } => { uwrite!(cli.writer(), "Hello, {}", name.unwrap_or("World")).ok(); },
            Base::Exit => { uwrite!(cli.writer(), "Exit").ok(); },
        }
    }

    pub fn process(&mut self, byte: u8) {
        let printing_state = &mut self.printing_state;
        let _ = self.cli.process_byte::<Group, _>(
            byte,
            &mut Group::processor(|cli, command| {
                match command {
                    Group::Base(cmd) => Self::base_cmds(cmd, cli),
                }
                Ok(())
            }),
        );
    }
}
