use core::usize::MAX;
use embedded_hal_nb::serial::{Read, Write};
use stm32f4xx_hal::serial::{Config, Error, Tx, Rx, Serial};
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::gpio::{PA2, PA3, Input};
use nb;
use stm32f4xx_hal::serial::config::{DmaConfig, Parity, StopBits, WordLength};
use stm32f4xx_hal::time::Bps;

const MAX_NMEA_0183: usize = 82;
const UTC_FIELD_SIZE: usize = 9;

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

    let (tx, rx) = uart.unwrap().split();

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

        let byte = nb::block!(self.rx.read()).unwrap();

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
            }

        }

        false
    }

    pub fn parse_message(&mut self) -> Result<(), GpsError<Error>> {

        let buf: [u8; MAX_NMEA_0183] = self.rx_buffer; // Copy for interrupt safety

        if buf[0] != b'$' { return Err(GpsError::BrokenMessage); }

        // Only need RMC messages, can add others later
        if &buf[3..6] != b"RMC" { return Ok(()) }

        self.parse_rmc(&buf)?;

        Ok(())
    }

    // UTC
    fn parse_utc(utc_slice: &[u8]) -> UtcTime {
        let temp = core::str::from_utf8(utc_slice).unwrap();
        UtcTime {
            hours: temp.parse().unwrap(),
            mins: temp.parse().unwrap(),
            secs: temp.parse().unwrap(),
        }
    }

    // LLA
    fn parse_lla(lat: &[u8], long: &[u8]) -> (f64, f64) {
        let lat_temp = core::str::from_utf8(lat).unwrap();
        let long_temp = core::str::from_utf8(long).unwrap();
        ( lat_temp.parse::<f64>().unwrap(), long_temp.parse::<f64>().unwrap() )
    }

    // Speed over ground
    fn parse_sog_cog(speed: &[u8], course: &[u8]) -> (f32, f32) {
        let s_t = core::str::from_utf8(speed).unwrap();
        let c_t = core::str::from_utf8(course).unwrap();
        ( s_t.parse::<f32>().unwrap() / KNOTS_TO_MS, c_t.parse::<f32>().unwrap() )
    }

    /// RMC FMT : $ID,UTC,STATUS,LAT,N/S,LONG,E/W,SPEED OVER GROUND, COURSE OVER GRND,DATE,
    ///           MAGNETIC VARIATION,EAST/WEST,MODE,CHECKSUM,TERMINATOR
    fn parse_rmc(&mut self, msg: &[u8; MAX_NMEA_0183]) -> Result<(), GpsError<Error>> {

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
        if valid_bit.first() != Some(&b'A') { return Err(GpsError::InvalidFix); }

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

        Ok(())
    }
    
    pub fn get_data(&mut self) -> GpsData {
        self.data
    }


}


