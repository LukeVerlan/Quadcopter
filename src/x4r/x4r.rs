use embedded_hal_nb::serial::Read;
use nb;

const SYNC_BYTE: u8 = 0x0F;

// Telemetry driver
pub struct X4r<SBUS> {
    data: X4rData,
    last_msg_timestamp: f32,
    sbus: SBUS,
}

#[derive(Debug, Clone, Copy)]
pub struct X4rData {
    // Data fields from over telemetry
}

impl <SBUS> X4r<SBUS> where
    SBUS: Read<u8>
{

    /// Constructor
    pub fn new(sbus: SBUS) -> Self {
        X4r {
            data: X4rData {},
            last_msg_timestamp: 0.0,
            sbus,
        }
    }

    /// Implementations
    fn build_message(&mut self) {

        let byte = match nb::block!(self.sbus.read()) {
            Ok(v) => v,
            Err(_) => return,
        };

    }

    fn parse(&mut self) {
        todo!();
    }

    pub fn get_data(&self) -> X4rData { self.data }
}



