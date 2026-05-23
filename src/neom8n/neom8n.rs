use embedded_io::{Write, Read};
use stm32f4xx_hal::serial::Error;
use embedded_hal_async::delay::DelayNs;
use stm32f4xx_hal::hal_02::blocking::delay::DelayUs;
use stm32f4xx_hal::serial::Serial;

pub const MAX_NMEA_0183: usize = 82;

#[derive(Copy, Clone)]
pub struct GpsData {

    // LLA position
    pub lat: f64, // Degrees
    pub long: f64, // Degrees

    // Heading
    pub heading: f32, // Degrees
    pub velocity: f32, // m/s
}

impl GpsData {
    pub fn new() -> Self {
        GpsData {
            lat: f64::NAN,
            long: f64::NAN,
            heading: f32::NAN,
            velocity: f32::NAN
        }
    }
}

pub enum GpsError<E> {
    Uart(E),
}

// RX handled externally by hardware interrupt
pub struct Neom8n {
    pub data: GpsData,
}


impl Neom8n {
    pub fn new() -> Self {

        // Setup
        Neom8n {

            data: GpsData {

                lat: f64::NAN,
                long: f64::NAN,
                velocity: f32::NAN,
                heading: f32::NAN
            }
        }

    }
}

impl Neom8n {


    // Read an incoming message out of the UART buffer
    // pub fn read_message_buf(&mut self) -> Result<[u8; MAX_NMEA_0183], GpsError<RX::Error>> {
    //
    //     let mut message: [u8; MAX_NMEA_0183] = [0; MAX_NMEA_0183];
    //     let mut index = 0;
    //     loop {
    //         if index == MAX_NMEA_0183 { break } // Overflow
    //
    //         let mut curr: [u8; 1] = [0];
    //         let res = self.rx.read(&mut curr);
    //         match res {
    //             Ok(_) => (),
    //             Err(e) => return Err(GpsError::Uart(e))
    //         }
    //
    //         message[index] = curr[0];
    //         if curr[0] == b'\n' { break } // Valid message
    //
    //         index += 1;
    //     }
    //
    //     Ok((message))
    // }

    pub fn parse_message(&mut self, message: [u8; MAX_NMEA_0183]) -> Result<(), GpsError<Error>> {
        todo!();
        Ok(())
    }
    
    pub fn get_data(&mut self) -> GpsData {
        self.data
    }


}


