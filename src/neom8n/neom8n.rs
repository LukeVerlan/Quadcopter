use embedded_hal_nb::serial::{Read, Write};
use stm32f4xx_hal::serial::{Config, Tx, Rx, Serial};
use stm32f4xx_hal::pac::USART2;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::gpio::{PA2, PA3, Input};
use nb;
use stm32f4xx_hal::serial::config::{DmaConfig, Parity, StopBits, WordLength};
use stm32f4xx_hal::time::Bps;
use ufmt::{uDisplay, Formatter, uwrite, uWrite};
use stm32f4xx_hal::prelude::*;
use super::super::util::util::{DisplayF32, DisplayF64};

// TODO: Comment functions on this bum code

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
pub struct FixQuality {
    fix: u8,
    satellites: u8
}

impl FixQuality {
    fn new() -> Self {
        FixQuality {
            fix: 0,
            satellites: 0
        }
    }

}

impl uDisplay for FixQuality {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        uwrite!(f, "Fix: {} \t Satellites: {}", self.fix, self.satellites)
    }
}

#[derive(Copy, Clone)]
pub struct UtcTime {
    pub hours: u8,
    pub mins: u8,
    pub secs: f32,
}

impl UtcTime {
    fn new() -> Self {
        UtcTime {
            hours: 0,
            mins: 0,
            secs: 0.0
        }

    }
}

impl uDisplay for UtcTime {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        uwrite!(f, "UTC {}:{}:{}", self.hours, self.mins, DisplayF32(self.secs))
    }
}

#[derive(Copy, Clone)]
pub struct GpsData {

    // LLA position
    pub lat: f32, // Degrees
    pub long: f32, // Degrees

    // Heading
    pub heading: f32, // Degrees
    pub velocity: f32, // m/s

    // Altitude
    pub alt: f32,  // m

    pub utc: UtcTime,
    pub fix_quality: FixQuality,

    // DEBUG
    pub rmc_rx_buf_copy: [u8; MAX_NMEA_0183],
    pub gga_rx_buf_copy: [u8; MAX_NMEA_0183]
}

impl GpsData {
    pub fn new() -> Self {
        GpsData {
            lat: f32::NAN,
            long: f32::NAN,
            heading: f32::NAN,
            velocity: f32::NAN,
            alt: f32::NAN,
            utc: UtcTime::new(),
            fix_quality: FixQuality::new(),
            rmc_rx_buf_copy: [0; MAX_NMEA_0183],
            gga_rx_buf_copy: [0; MAX_NMEA_0183]

        }
    }

}

impl uDisplay for GpsData {
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error> {
        uwrite!(
            f,
            "+-------------------------------+ \n\
             | {}    \n\
             | {} \n\
             | \n\
             | Lat:      {} Long:     {}    \n\
             | Heading:  {} Velocity: {}    \n\
             | Alt: {} \n\
             | RMC_BUF: {} \n\
             | GGA_BUF: {} \n\
             +------------------------------+\n",
            self.utc, self.fix_quality,
            DisplayF32(self.lat), DisplayF32(self.long),
            DisplayF32(self.heading), DisplayF32(self.velocity),
            DisplayF32(self.alt), core::str::from_utf8(&self.rmc_rx_buf_copy).unwrap(), core::str::from_utf8(&self.gga_rx_buf_copy).unwrap(),
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


        // For Message syncing
        if self.msg_started {

            // Garbled Data
            if self.msg_idx >= MAX_NMEA_0183 {
                self.msg_started = false;
                self.msg_idx = 0;

            // End of Message
            } else if byte == b'\n' {
                self.msg_started = false;
                self.rx_buffer[self.msg_idx] = byte;
                self.msg_idx = 0;
                return true;

            // Valid Message
            } else {
                self.rx_buffer[self.msg_idx] = byte;
                self.msg_idx += 1;
            }

        } else {

            // Start of message sync byte
            if byte == b'$' {
                self.msg_idx = 0;
                self.msg_started = true;
                self.rx_buffer = [0; MAX_NMEA_0183];
                self.rx_buffer[self.msg_idx] = byte;
                self.msg_idx = 1;
            }

        }

        false
    }

    pub fn parse_message(&mut self) {

        // Copy for interrupt safety
        let buf: [u8; MAX_NMEA_0183] = self.rx_buffer;

        // Parse messages correctly
        match &buf[3..6] {
            b"RMC" => self.parse_rmc(&buf),
            b"GGA" => self.parse_gga(&buf),
            _ => return
        }

    }

    fn parse_f32(field: &[u8]) -> f32 {
        str::from_utf8(field).unwrap_or("NaN").parse::<f32>().unwrap_or(f32::NAN)
    }

    fn parse_u8(val: &[u8]) -> u8 {
        str::from_utf8(val).unwrap_or("0").parse::<u8>().unwrap_or(0)
    }

    // UTC
    fn parse_utc(utc_slice: &[u8]) -> UtcTime {
        let temp = core::str::from_utf8(utc_slice).unwrap_or("000000");
        UtcTime {
            hours: temp[0..2].parse().unwrap_or(0),
            mins:  temp[2..4].parse().unwrap_or(0),
            secs:  Self::parse_f32(&utc_slice[4..]),
        }
    }

    fn lla_frac_mins_to_deg(deg: f32, mins: f32) -> f32 { deg + mins / 60.0 }
    // LLA
    fn parse_lla(lat: &[u8], long: &[u8], ns: &[u8], ew: &[u8]) -> (f32, f32) {

        let lat_d = Self::parse_f32(&lat[..2]);
        let lat_m = Self::parse_f32(&lat[2..]);

        let long_d = Self::parse_f32(&long[..3]);
        let long_m = Self::parse_f32(&long[3..]);

        let mut lat = Self::lla_frac_mins_to_deg(lat_d, lat_m);
        let mut long = Self::lla_frac_mins_to_deg(long_d, long_m);

        // Correct for location
        if ns == b"S" { lat *= -1.0 }
        if ew == b"W" { long *= -1.0 }

        ( lat, long )
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
        let _mag_dir  = sections.next().unwrap();

        // Valid bit is one char long
        if valid_bit.first() != Some(&b'A') { return }

        // Parse Fields
        let (lat, long) = Self::parse_lla(lat, long, ns, ew);
        let utc = Self::parse_utc(utc);
        let (velocity, heading) = Self::parse_sog_cog(speed, course);

        // Update the data
        self.data.utc = utc;
        self.data.lat = lat;
        self.data.long = long;
        self.data.heading = heading;
        self.data.velocity = velocity;

        self.data.rmc_rx_buf_copy = *msg;
    }

    fn parse_gga(&mut self, msg: &[u8; MAX_NMEA_0183]) {
        let mut sections = msg.split(|&b| b == b',');
        sections.next(); // Skip header

        // Break into needed pieces
        let utc = sections.next().unwrap();
        let lat = sections.next().unwrap();
        let ns = sections.next().unwrap();
        let long = sections.next().unwrap();
        let ew = sections.next().unwrap();
        let fix = sections.next().unwrap();
        let satellites = sections.next().unwrap();
        let _hdop = sections.next().unwrap();
        let msl_alt   = sections.next().unwrap();       // skip
        let _units = sections.next().unwrap();                // skip
        let _geoid_separation  = sections.next().unwrap();    // skip
        let _units = sections.next().unwrap();
        let _age = sections.next().unwrap();

        let utc = Self::parse_utc(utc);
        let (lat, long) = Self::parse_lla(lat, long, ns, ew);
        let fix = Self::parse_u8(fix);
        let alt = Self::parse_f32(msl_alt);


        let satellites = Self::parse_u8(satellites);

        let fix_qual = FixQuality { fix, satellites };

        // Update the values
        self.data.utc = utc;
        self.data.lat = lat;
        self.data.long = long;
        self.data.alt = alt;
        self.data.fix_quality = fix_qual;

        self.data.gga_rx_buf_copy = *msg;
    }
    
    pub fn get_data(&mut self) -> GpsData { self.data }
}
