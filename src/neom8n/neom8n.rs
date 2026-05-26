use embedded_hal_nb::serial::{Read, Write};
use stm32f4xx_hal::serial::{Config, Tx, Rx, Serial};
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::gpio::{PA2, PA3, Input};
use nb;
use stm32f4xx_hal::serial::config::{DmaConfig, Parity, StopBits, WordLength};
use stm32f4xx_hal::time::Bps;
use ufmt::{uDisplay, Formatter, uwrite};
use stm32f4xx_hal::prelude::*;
use super::super::util::util::{DisplayF32, DisplayF64};

const MAX_NMEA_0183: usize = 82;

const KNOTS_TO_MS: f32 = 1.94384;

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

    let (tx, mut rx) = uart.unwrap().split();

    rx.listen(); // Enable interrupts

    Neom8n::new(rx, tx)
}

#[derive(Copy, Clone)]
pub struct UtcTime {
    pub hours: u8,
    pub mins: u8,
    pub secs: u8,
}

impl UtcTime {
    fn new() -> Self {
        UtcTime {
            hours: 0,
            mins: 0,
            secs: 0
        }

    }
}

#[derive(Copy, Clone)]
pub struct GpsData {

    // LLA position
    pub lat: f64, // Degrees
    pub long: f64, // Degrees

    // Heading
    pub heading: f32, // Degrees
    pub velocity: f32, // m/s

    pub utc: UtcTime,
}

impl GpsData {
    pub fn new() -> Self {
        GpsData {
            lat: f64::NAN,
            long: f64::NAN,
            heading: f32::NAN,
            velocity: f32::NAN,
            utc: UtcTime::new(),
        }
    }

}

impl uDisplay for GpsData {
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error> {
        uwrite!(
            f,
            "+-------------------------------- \n\
             | Hours: {} Mins {} Sec {}     \n\
             | Lat:      {} Long:     {}    \n\
             | Heading:  {} Velocity: {}    \n\
             +------------------------------+\n",
            self.utc.hours, self.utc.mins, self.utc.secs,
            DisplayF64(self.lat), DisplayF64(self.long),
            DisplayF32(self.heading), DisplayF32(self.velocity),
        )
    }
}


#[derive(Debug)]
pub enum GpsError<E> {
    Uart(E),
    BrokenMessage,
    InvalidFix
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

        let byte = match nb::block!(self.rx.read()) {
            Ok(b) => b,
            Err(_) => return false,
        };

        self.rx_buffer[self.msg_idx] = byte;

        // For Message syncing
        if self.msg_started {

            // Garbled Data
            if self.msg_idx >= MAX_NMEA_0183 {
                self.msg_started = false;
                self.msg_idx = 0;

            // End of Message
            } else if byte == b'\n' {
                self.msg_started = false;
                self.msg_idx = 0;
                return true;

            // Valid Message
            } else {
                self.msg_idx += 1;
            }

        } else {

            // Start of message sync byte
            if byte == b'$' {
                self.msg_started = true;
                self.msg_idx = 1;
                self.rx_buffer = [0; MAX_NMEA_0183];
            }

        }

        false
    }

    pub fn parse_message(&mut self) {

        let buf: [u8; MAX_NMEA_0183] = self.rx_buffer; // Copy for interrupt safety

        // Only need RMC messages, can add others later
        if &buf[3..6] != b"RMC" { return }

        self.parse_rmc(&buf);
    }

    fn parse_f32(field: &[u8]) -> f32 {
        if field.is_empty() {
            return f32::NAN;
        }
        core::str::from_utf8(field)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(f32::NAN)
    }

    fn parse_f64(field: &[u8]) -> f64 {
        if field.is_empty() {
            return f64::NAN;
        }
        core::str::from_utf8(field)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(f64::NAN)
    }

    // UTC
    fn parse_utc(utc_slice: &[u8]) -> UtcTime {
        let temp = core::str::from_utf8(utc_slice).unwrap_or("000000");
        UtcTime {
            hours: temp[0..2].parse().unwrap_or(0),
            mins:  temp[2..4].parse().unwrap_or(0),
            secs:  temp[4..6].parse().unwrap_or(0),
        }
    }

    fn lla_frac_mins_to_deg(mins: f64, deg: f64) -> f64 { deg + (mins)/60.0 }
    // LLA
    fn parse_lla(lat: &[u8], long: &[u8]) -> (f64, f64) {
        let lat_d = Self::parse_f64(&lat[0..2]);
        let lat_m = Self::parse_f64(&lat[2..9]);

        let long_d = Self::parse_f64(&long[0..3]);
        let long_m = Self::parse_f64(&long[3..10]);

        ( Self::lla_frac_mins_to_deg(lat_d, lat_m), Self::lla_frac_mins_to_deg(long_d, long_m) )
    }

    // Speed over ground
    fn parse_sog_cog(speed: &[u8], course: &[u8]) -> (f32, f32) {
        ( Self::parse_f32(speed) / KNOTS_TO_MS, Self::parse_f32(course) )
    }

    /// RMC FMT : $ID,UTC,STATUS,LAT,N/S,LONG,E/W,SPEED OVER GROUND, COURSE OVER GROUND,DATE,
    ///           MAGNETIC VARIATION,EAST/WEST,MODE,CHECKSUM,TERMINATOR
    fn parse_rmc(&mut self, msg: &[u8; MAX_NMEA_0183]) {

        let mut sections = msg.split(|&b| b == b',');
        sections.next(); // Skip header

        // Break into needed pieces
        let utc = sections.next().unwrap();
        let valid_bit = sections.next().unwrap();
        let lat = sections.next().unwrap();
        let ns = sections.next().unwrap();
        let long = sections.next().unwrap();
        let ew = sections.next().unwrap();
        let speed = sections.next().unwrap();  // Knots
        let course = sections.next().unwrap(); // Degrees
        let _date    = sections.next().unwrap();      // skip
        let _mag_var = sections.next().unwrap();      // skip
        let mag_dir  = sections.next().unwrap();

        // Valid bit is one char long
        if valid_bit.first() != Some(&b'A') { return }

        // Parse Fields
        let (lat, long) = Self::parse_lla(lat, long);
        let utc = Self::parse_utc(utc);
        let (velocity, heading) = Self::parse_sog_cog(speed, course);

        // Update the data
        self.data = GpsData {
            lat,
            long,
            velocity,
            heading,
            utc
        };
    }
    
    pub fn get_data(&mut self) -> GpsData { self.data }
}


