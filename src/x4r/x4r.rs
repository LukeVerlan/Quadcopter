use defmt::{Formatter, Format, write};
use super::super::util::util::DisplayFloat;
use stm32f4xx_hal::dma::config::DmaConfig as SerialDmaConfig;
use stm32f4xx_hal::serial::{Config as SerialConfig};
use stm32f4xx_hal::serial::config::{DmaConfig, IrdaMode, Parity, StopBits, WordLength};
use stm32f4xx_hal::time::Bps;

pub const SBUS_MESSAGE_LENGTH: usize  = 25;

pub const FLAGS_IDX: usize = 24;

const SYNC_BYTE: u8 = 0x0F;
const FOOTER_BYTE: u8 = 0x00;

const FAILSAFE_FLAG: u8 = 0x08;
const FRAME_LOST_FLAG: u8 = 0x04;

const SBUS_MIN: u16 = 172; // -100% power
const SBUS_MAX: u16 = 1811; // 100% power

/// Configs
pub const X4R_SBUS_CONFIG: SerialConfig = SerialConfig {
    baudrate: Bps(100_000),
    wordlength: WordLength::DataBits8,
    parity: Parity::ParityEven,
    dma: DmaConfig::Rx,
    stopbits: StopBits::STOP2,
    irda: IrdaMode::None,
};

pub fn get_x4r_dma_config() -> SerialDmaConfig {
    SerialDmaConfig::default()
        .memory_increment(true)
        .transfer_complete_interrupt(true)
        .double_buffer(false)
}



/** Converts an channel value to a percent value
 *  @param val: SBUS mapped value
 *  @return Percent value between -1 and 1
 */
fn convert_to_percent(val: u16) -> f32 {
    if val < SBUS_MIN || val > SBUS_MAX { return f32::NAN; }
    (((val - SBUS_MIN) as f32 / (SBUS_MAX - SBUS_MIN) as f32) * 2.0) - 1.0
}

/** Telemetry System */
pub struct X4r { data: X4rData, }

/** Error system for Telemetry
    NoErr       -> Clean transmission
    FramingErr  -> Missplaced sync / footer
    FlagsErr    -> No Signal
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X4rError{
    NoErr,
    FramingErr,
    FailSafeErr
}

/** ToString functionality for defmt printing of the Error type */
impl Format for X4rError {
    fn format(&self, fmt: Formatter) {
        match self {
            Self::NoErr => write!(fmt, "No Error"),
            Self::FramingErr => write!(fmt, "Framing Error"),
            Self::FailSafeErr => write!(fmt, "FATAL: No Signal"),
        }
    }
}

/** Data storage for the Telemetry System */
#[derive(Debug, Clone, Copy)]
pub struct X4rData {
    status: X4rError,
    throttle: f32,  // CH1
    roll:     f32,  // CH2
    pitch:    f32,  // CH3
    yaw:      f32,  // CH4
}

/** ToString functionality for defmt printing of the data */
impl Format for X4rData {
    fn format(&self, fmt: Formatter) {
        write!(fmt, "Error: {=?} \t Throttle {} \t Roll {} \t Pitch {} \t Yaw {} \n",
               self.status, DisplayFloat(self.throttle), DisplayFloat(self.roll),
               DisplayFloat(self.pitch), DisplayFloat(self.yaw))
    }
}

/** Implementations for the telemetry system */
impl X4r {

    /// Constructor
    pub fn new() -> Self {

        X4r {
            data: X4rData {
                status: X4rError::NoErr,
                throttle: 0.0,
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
            },
        }

    }

    /** Parses a given buffer as an SBUS message
     *  Errors:
     *      FramingErr -> PacketCame over corrupted
     *      FlagsErr   -> The reciever sent over error flags
     *  Params:
     *      Takes in a buffer to parse an SBUS buffer
     */
    pub fn parse(&mut self, buf: &[u8; SBUS_MESSAGE_LENGTH]) -> Result<(), X4rError> {

        // Verify Header and footer
        if buf[0] != SYNC_BYTE || buf[24] != FOOTER_BYTE {

            self.data = X4rData {
                status: X4rError::FramingErr,
                throttle: 0.0,
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
            };

            defmt::println!("Framing Error");
            return Err(X4rError::FramingErr);
        }

        let _flags = buf[FLAGS_IDX]; // Flags byte

        if _flags & FAILSAFE_FLAG > 0 || _flags & FRAME_LOST_FLAG > 0 {

            self.data = X4rData {
                status: X4rError::FailSafeErr,
                throttle: 0.0,
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
            };

            defmt::println!("FailSafe Error");

            return Err(X4rError::FailSafeErr);
        }

        // 11 bit dataframes
        let ch1 =
                (buf[1] as u16)  |
                (((buf[2] as u16) & 0x07) << 8);
        let ch2 =
                (((buf[2] as u16) & 0xF8) >> 3)  |
                (((buf[3] as u16) & 0x3F) << 5);
        let ch3 =
                (((buf[3] as u16) & 0xC0) >> 6) |
                ((buf[4] as u16) << 2) |
                (((buf[5] as u16) & 0x01) << 10);
        let ch4 =
                ((buf[5] as u16) >> 1) |
                (((buf[6] as u16) & 0x0F) << 7);

        let throttle = convert_to_percent(ch1);
        let roll = convert_to_percent(ch2);
        let pitch = convert_to_percent(ch3);
        let yaw = convert_to_percent(ch4);

        self.data = X4rData{
            status: X4rError::NoErr,
            throttle,
            roll,
            pitch,
            yaw
        };

        defmt::println!("Valid: \t {}", self.data);

        Ok(())

    }

    /** Returns the current stored telem data */
    pub fn get_data(&self) -> X4rData { self.data }
}