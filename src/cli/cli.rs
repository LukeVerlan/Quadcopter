use embedded_cli::Command;
use stm32f4xx_hal::otg_fs::{USB, UsbBus};
use usbd_serial::SerialPort;

// Serial port writer
pub struct Writer {
    pub ser: SerialPort<'static, UsbBus<USB>>
}

// Error implementation
impl embedded_io::ErrorType for Writer {
   type Error = core::convert::Infallible;
}

impl embedded_io::Write for Writer {

    // Write some bytes over serial
    fn write(&mut self, buffer: &[u8]) -> Result<usize, Self::Error> {
        let len = self.ser.write(buffer).unwrap_or(0);
        Ok(len)
    }

    // Flush the serial port
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.ser.flush().ok();
        Ok(())
    }
}

#[derive(Command)]
enum Base<'a> {
    
    Hello {
        name: Option<&'a str>,
    },
    
    Exit,
}


