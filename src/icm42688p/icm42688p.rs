
// hal interface
extern crate embedded_hal;

// Bank Numbers
use super::reg::{
    BankNum,
    Bank0,
    Bank1,
    Bank2,
    Bank3,
    Bank4,
};

// Accel Data -- G forces
struct Accel {
    accel_x: f64,
    accel_y: f64,
    accel_z: f64,
}

// Gyro Data -- Meters / Second^2
struct Gyro {
    gyro_x: f64,
    gyro_y: f64,
    gyro_z: f64,
}

// Imu definition
pub struct Icm42688p<SPI> {
    spi: SPI,
    accel: Accel,
    gyro: Gyro,
}

// Constructor
impl <SPI> Icm42688p<SPI> {
    pub fn new<E>(spi: SPI) -> Self
    where
        SPI: embedded_hal::spi::SpiDevice,
    {
        // Return reference to imu
        Icm42688p {

            spi,

            accel: Accel {
                accel_x: 0.0,
                accel_y: 0.0,
                accel_z: 0.0
            },

            gyro: Gyro {
                gyro_x: 0.0,
                gyro_y: 0.0,
                gyro_z: 0.0
            }

        }
    }
}

// Exposed functions
impl <SPI: embedded_hal::spi::SpiDevice> Icm42688p<SPI> {

    fn set_config(&mut self) -> Result<(), SPI::Error> {

        // self.spi.write()
        Ok(())
    }
}