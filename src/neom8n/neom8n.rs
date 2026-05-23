use embedded_hal_nb::serial::{Read, Write};
use stm32f4xx_hal::serial::{Config, Error, Tx, Rx, Serial};
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::gpio::{PA2, PA3, Input};
use nb;
use stm32f4xx_hal::serial::config::{DmaConfig, Parity, StopBits, WordLength};
use stm32f4xx_hal::time::Bps;

pub const MAX_NMEA_0183: usize = 82;
pub const CONFIG: Config = Config {
    baudrate: Bps(9600),
    wordlength: WordLength::DataBits8,
    parity: Parity::ParityNone,
    stopbits: StopBits::STOP1,
    dma: DmaConfig::None
};

pub fn gps_setup(
    usart2: USART2,
    tx: PA2<Input>,
    rx: PA3<Input>,
    clocks: &Clocks,
) -> Neom8n<Rx<USART2>,Tx<USART2>>{

    let uart = Serial::<USART2, u8>::new(
        usart2,
        (tx.into_alternate::<7>(), rx.into_alternate::<7>()),
        CONFIG,
        &clocks
    );

    let (tx, rx) = uart.unwrap().split();

    Neom8n::new(rx, tx)
}

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

    pub fn from(
        lat: f64,
        long: f64,
        heading: f32,
        velocity: f32,
    ) -> Self {
        GpsData { lat, long, heading, velocity }
    }
}

#[derive(Debug)]
pub enum GpsError<E> {
    Uart(E),
}

// RX handled externally by hardware interrupt
pub struct Neom8n<RX, TX> {
    rx: RX,
    tx: TX,
    data: GpsData,
    rx_buffer: [u8; MAX_NMEA_0183],
    msg_idx: usize,
    msg_started: bool
}


impl <RX, TX> Neom8n<RX, TX>
where
    RX: Read<u8>,
    TX: Write<u8>,
{
    pub fn new(rx: RX, tx: TX) -> Self {

        // Setup
        Neom8n {
            rx,
            tx,
            data: GpsData::new(),
            rx_buffer: [0; MAX_NMEA_0183],
            msg_idx: 0,
            msg_started: false
        }

    }

    /// This function is called in ISR context to build a GPS message
    /// Byte wise; returns true if a message is built in the buffer, false if the
    /// message is not ready
    pub fn build_message(&mut self) -> bool {

        let byte = nb::block!(self.rx.read()).unwrap();

        self.rx_buffer[self.msg_idx] = byte;

        // For Message syncing
        if self.msg_started {
            if byte == b'\n' {
                self.msg_started = false;
                self.msg_idx = 0;
                true
            } else {
                self.msg_idx += 1;
                false
            }
        } else {
            if byte == b'$' {
                self.msg_started = true;
                self.msg_idx = 1;
            }
            false
        }
    }

    pub fn parse_message(&mut self) -> Result<(), GpsError<Error>> {
        todo!();
        Ok(())
    }
    
    pub fn get_data(&mut self) -> GpsData {
        self.data
    }


}


