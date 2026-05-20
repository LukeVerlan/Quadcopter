
// hal interface
use embedded_hal;
use embedded_hal_async;
// Bank Numbers
use super::reg::{
    BankNum,
    Bank0,
    Bank1,
    Bank2,
    Bank3,
    Bank4,
};

// Sensor Setup
use super::config::{GYRO_ODR, GYRO_FSR, ACCEL_ODR, ACCEL_FSR, RESET_CMD, ACCEL_LN_CMD, GYRO_LN_CMD};

// Accel Data -- G forces
struct Accel {
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
}

// Gyro Data -- Meters / Second^2
struct Gyro {
    gyro_x: f32,
    gyro_y: f32,
    gyro_z: f32,
}

// Imu definition
pub struct Icm42688p<SPI, D> {
    spi: SPI,
    delay: D,
    accel: Accel,
    gyro: Gyro
}

// Constructor
impl <SPI, D> Icm42688p<SPI, D> {
    pub fn new(spi: SPI, delay: D) -> Self
    where
        SPI: embedded_hal::spi::SpiDevice,
        D:  embedded_hal_async::delay::DelayNs
    {
        // Return reference to imu
        let chip = Icm42688p {

            spi,
            delay,

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

        };

        chip
    }
}

// Exposed functions
impl <SPI: embedded_hal::spi::SpiDevice, D: embedded_hal_async::delay::DelayNs> Icm42688p<SPI, D> {
    async fn startup(&mut self) -> Result<(), SPI::Error> {

        // Power on delay
        self.delay.delay_ms(100).await;

        // Reset for clarity
        self.soft_reset().await?;
        self.delay.delay_ms(100).await;

        self.set_sensor_config().await?;

        Ok(())
    }

    async fn start_sensors(&mut self) -> Result<(), SPI::Error> {
        let buf: [u8; 2] = [Bank0::PwrMGMT0 as u8, (GYRO_LN_CMD << 2) | ACCEL_LN_CMD ];
        self.spi.write(&buf)?;
        Ok(())
    }

    async fn soft_reset(&mut self) -> Result<(), SPI::Error> {
        let buf: [u8; 2] = [Bank0::DeviceConfig as u8, RESET_CMD];
        self.spi.write(&buf)?;
        Ok(())
    }

    async fn set_sensor_config(&mut self) -> Result<(), SPI::Error> {

        // setup commands
        let commands: [[u8; 2]; 2] = [
            [Bank0::GyroConfig0 as u8, ((GYRO_FSR as u8) << 5) | GYRO_ODR as u8 ], // Gyro Settings
            [Bank0::AccelConfig0 as u8, ((ACCEL_FSR as u8) << 5) | ACCEL_ODR as u8] // Accel Settings
        ];

        for command in commands {
            self.spi.write(&command)?;
        }

        Ok(())
    }
}