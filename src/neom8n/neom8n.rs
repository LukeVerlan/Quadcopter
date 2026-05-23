use embedded_io::{Write, Read};
use stm32f4xx_hal::serial::Error;
use embedded_hal_async::delay::DelayNs;


#[derive(Copy, Clone)]
pub struct GpsData {

    // LLA position
    lat: f64, // Degrees
    long: f64, // Degrees

    // Heading
    heading: f32, // Degrees
    velocity: f32, // m/s
}

pub enum GpsError<E> {
    Uart(E)
}

pub struct Neom8n<RX, TX, D> {
    pub data: GpsData,
    rx : RX,
    tx : TX,
    delay: D
}

impl<RX, TX, D> Neom8n<RX, TX, D> {
    pub fn new(rx: RX, tx: TX, delay: D) -> Self
    where
        RX: Read,
        TX: Write,
        D: DelayNs
    {

        // Setup
        Neom8n {

            rx,
            tx,
            delay,

            data: GpsData {

                lat: f64::NAN,
                long: f64::NAN,
                velocity: f32::NAN,
                heading: f32::NAN
            }
        }

    }
}

impl <RX, TX, D> Neom8n<RX, TX, D> {

    pub fn read_message(&mut self) {

        while 
    }
}


