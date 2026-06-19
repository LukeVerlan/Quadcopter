use embedded_hal_nb::serial::Read;

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
    fn new(sbus: SBUS) -> Self {
        X4r {
            data: X4rData {},
            last_msg_timestamp: 0.0,
            sbus,
        }
    }

    /// Implementations
    fn receive(&mut self) {
        todo!();
    }

    fn parse(&mut self) {
        todo!();
    }

    fn get_data(&mut self) -> X4rData { self.data }
}



