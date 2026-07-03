use embedded_hal_nb::serial::Read;
use nb;

pub const SBUS_MESSAGE_LENGTH: usize  = 25;

const SYNC_BYTE: u8 = 0x0F;
const FOOTER_BYTE: u8 = 0x00;

const FAILSAFE_FLAG: u8 = 0x08;
const FRAME_LOST_FLAG: u8 = 0x04;

const MIN: u16 = 172; // -100% power
const MAX: u16 = 1811; // 100% power



// Converts a SBUS value to a min or maximum percentage
fn convert_to_percent(
    val: u16
) -> f32 {
    ((val - MIN) as f32 / (MAX - MIN) as f32) * 2.0 - 1.0
}

// Telemetry driver
pub struct X4r {
    data: X4rData,
}

#[derive(Debug, Clone, Copy)]
pub enum X4rError{
    NoErr,
    BufferErr,
    FlagsErr
}

#[derive(Debug, Clone, Copy)]
pub struct X4rData {
    status: X4rError,
    throttle: f32,  // CH1
    roll:     f32,  // CH2
    pitch:    f32,  // CH3
    yaw:      f32,  // CH4
}

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

    /// Implementations
    fn build_message(&mut self) {
        todo!("Implement with DMA");

    }


    // Given a full SBUS message
    fn parse(&mut self) {
        let buf: [u8; SBUS_MESSAGE_LENGTH] = [0; SBUS_MESSAGE_LENGTH];

        // Verify Header and footer
        if buf[0] != SYNC_BYTE && buf[23] != FOOTER_BYTE {
            self.data.status = X4rError::BufferErr;
            return
        }

        let _flags = buf[22]; // Flags byte

        if _flags & FAILSAFE_FLAG > 0 || _flags & FRAME_LOST_FLAG > 0 {
            self.data.status = X4rError::FlagsErr;
            return
        }

        // -- Valid Data --

        // 11 bit dataframes
        let ch1 = (buf[1] as u16)  | (((buf[2] as u16) & 0x07) << 8);
        let ch2 = (((buf[2] as u16) & 0xF8) >> 3)  | (((buf[3] as u16) & 0x3F) << 5);
        let ch3 = (((buf[3] as u16) & 0xC0) >> 6) | ((buf[4] as u16) << 2) | (((buf[5] as u16) & 0x01) << 10);
        let ch4=  (((buf[5] as u16) & 0xFE) >> 1) | (((buf[6] as u16) & 0xF) << 1);
        // Ignore the other channels for now

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

    }

    pub fn get_data(&self) -> X4rData { self.data }
}



