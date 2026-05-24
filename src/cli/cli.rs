use core::sync::atomic::{ Ordering};
use embedded_cli::{Command, CommandGroup};
use stm32f4xx_hal::otg_fs::{USB, UsbBus};
use usbd_serial::SerialPort;
use core::convert::Infallible;
use embedded_cli::cli::Cli;
use rtic::export::atomic::AtomicBool;

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

static GPS_PRINTING: AtomicBool = AtomicBool::new(false);

#[derive(Command)]
#[command(help_title = "Gps Functions")]
enum GpsCli {
    // Print current GPS data
    StartPrint,
    StopPrint,
}

#[derive(Command)]
pub enum Base<'a> {
    
    Hello {
        name: Option<&'a str>,
    },
    
    Exit,
}

#[derive(CommandGroup)]
enum Group<'a> {
    Base(Base<'a>),
    Gps(GpsCli),
}

fn process_gps_cmds(cmd: GpsCli) -> Result<(), Infallible> {
    match cmd {
        StartPrint => unsafe {GPS_PRINTING.store(true, Ordering::Relaxed) ,
    }
}

// cli/mod.rs
pub fn process(cli: &mut Cli<Writer, Infallible, &'static mut [u8], &'static mut [u8]>, byte: u8) {
    let _ = cli.process_byte::<Base, _>(
        byte,
        &mut Group::processor(|cli, command| {
            match command {
                Group::Base(cmd) => todo!("Process Base cmd"),
                Group::Gps(cmd) => todo!("Process Gps cmd")
            }
            Ok(())
        }),
    );
}


