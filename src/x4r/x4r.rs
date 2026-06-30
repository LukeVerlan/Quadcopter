use embedded_hal_nb::serial::Read;
use nb;

const SYNC_BYTE: u8 = 0x0F;
const FOOTER_BYTE: u8 = 0x00;
const MIN: u16 = 172; // -100% power
const MAX: u16 = 1811; // 100% power


#[allow(static_mut_refs)]
// 25 Byte buffer
static mut DMA_BUFFER: [u8; 25] = [0; 25];

// Telemetry driver
pub struct X4r<SBUS> {
    data: X4rData,
    last_msg_timestamp: f32,
    sbus: SBUS,
}

pub enum X4rError{
    Parsing,
}

#[derive(Debug, Clone, Copy)]
pub struct X4rData {
    throttle: f32,  // CH1
    roll:     f32,  // CH2
    pitch:    f32,  // CH3
    yaw:      f32,  // CH4
}

impl <SBUS> X4r<SBUS> where
    SBUS: Read<u8>
{

    /// Constructor
    pub fn new(sbus: SBUS) -> Self {

        X4r {

            data: X4rData {
                throttle: 0.0,
                roll: 0.0,
                pitch: 0.0,
                yaw: 0.0,
            },

            last_msg_timestamp: 0.0,
            sbus,
        }

    }

    /// Implementations
    fn build_message(&mut self) {
        todo!("Implement with DMA");

    }


    // Given a full SBUS message
    fn parse(&mut self) -> Result<(), X4rError> {
        // Grab the buffer
        let buf = unsafe { DMA_BUFFER };

        // Verify Header and footer
        if buf[0] != SYNC_BYTE && buf[23] != FOOTER_BYTE {
            return Err(X4rError::Parsing)
        };

        let _flags = buf[22];

        // 11 bit dataframes
        let ch1 = (buf[1] as u16)  | (((buf[2] as u16) & 0x07) << 8);
        let ch2 = (((buf[2] as u16) & 0xF8) >> 3)  | (((buf[3] as u16) & 0x3F) << 5);
        let ch3 = (((buf[3] as u16) & 0xC0) >> 6) | ((buf[4] as u16) << 2) | (((buf[5] as u16) & 0x01) << 10);
        let ch4=  (((buf[5] as u16) & 0xFE) >> 1) | (((buf[6] as u16) & 0x07) << 1);
        let ch4 =
        Ok(())
    }

    pub fn get_data(&self) -> X4rData { self.data }
}



