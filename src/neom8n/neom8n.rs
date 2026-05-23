use embedded_hal::

pub struct GpsData {

    // LLA position
    pub lat: f64, // Degrees
    pub long: f64, // Degrees

    // Heading
    pub heading: f64, // Degrees
    pub velocity: f64, // m/s
}

pub enum GpsError<E> {
    Uart(E)
}

pub struct Neom8N<UART> {
    pub data: GpsData,
    uart : UART,
}

impl<UART> Neom8N<UART> {
    pub fn new(uart: UART) -> Neom8N<UART> -> Result<Self <>
    where
}
