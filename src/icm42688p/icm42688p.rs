
/// LONG TERM
// TODO : Functionality -> Interrupt setup (INT_CONFIG, Identify Pin Number)
// Want INT1 mode to be 1 (latched mode), open drain (default), and active low (default)
// Data ready flag exists as bit 3 of the int status register
// As long as Accel and Gyro have same sample rate, when this flag goes off just burst read the whole shit
// TODO: Learn about notch filtering and Anti-Alias for better data quality

/// NOW!!
// TODO: Start with polling implementation, just create a function that burst read regs
// TODO: comments functions
// and gyro registers

// hal interface
use embedded_hal::spi::SpiDevice;
use embedded_hal_async::delay::DelayNs;

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
use super::config::*;

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

pub enum ImuError<E> {
    Spi(E),
    WhoAmI
}

impl<E> From<E> for ImuError<E> {
    fn from(e: E) -> Self {
        ImuError::Spi(e)
    }
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
    pub async fn new(spi: SPI, delay: D) -> Result<Self, ImuError<SPI::Error>>
    where
        SPI: SpiDevice,
        D:  DelayNs
    {
        // Return reference to imu
        let mut chip = Icm42688p {

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

        chip.startup().await?;

        Ok(chip)
    }
}

// Exposed functions
impl <SPI: SpiDevice, D: DelayNs> Icm42688p<SPI, D> {

    // TODO: Burst read
    pub async fn get_data(&mut self) {

    }

    // TODO: Enable interrupts
    async fn enable_int(&mut self) {

    }

    // TODO: Notch filter setup
    async fn notch_filter_setup(&mut self){

    }

    // TODO: Anti Alias Filter
    async fn aaf_filter_setup(&mut self) {

    }

    async fn spi_write_reg(&mut self, addr: u8, data: u8) -> Result<(), ImuError<SPI::Error>> {
        let buf: [u8; 2] = [addr & (SPI_WRITE_CMD << 7), data];
        self.spi.write(&buf)?;
        Ok(())
    }

    async fn spi_read_reg(&mut self, addr: u8) -> Result<u8, SPI::Error> {
        let mut buf: [u8; 2] = [addr & (SPI_READ_CMD << 7), 0];
        self.spi.transfer_in_place(&mut buf)?;
        Ok(buf[1]) // return the buffered value
    }

    async fn startup(&mut self) -> Result<(), ImuError<SPI::Error>> {

        // Power on delay
        self.delay.delay_ms(100).await;

        // Verify the Who am I
        let who_am_i = self.spi_read_reg(Bank0::WhoAmI as u8).await?;
        if who_am_i != WHO_AM_I {
            return Err(ImuError::WhoAmI);
        }

        // Reset for clarity
        self.soft_reset().await?;

        self.delay.delay_ms(100).await;

        self.set_sensor_config().await?;

        // Delay for startup time
        self.delay.delay_ms(100).await;

        Ok(())
    }

    async fn start_sensors(&mut self) -> Result<(), ImuError<SPI::Error>> {

        self.spi_write_reg(
            Bank0::PwrMGMT0 as u8,
            (GYRO_LN_CMD << 2) | ACCEL_LN_CMD
        ).await?;

        Ok(())
    }

    async fn soft_reset(&mut self) -> Result<(), ImuError<SPI::Error>> {

        self.spi_write_reg(
            Bank0::DeviceConfig as u8,
            RESET_CMD
        ).await?;

        Ok(())
    }

    async fn set_sensor_config(&mut self) -> Result<(), ImuError<SPI::Error>> {

        // setup commands
        let commands: [[u8; 2]; 2] = [
            [Bank0::GyroConfig0 as u8, ((GYRO_FSR as u8) << 5) | GYRO_ODR as u8 ],  // Gyro Settings
            [Bank0::AccelConfig0 as u8, ((ACCEL_FSR as u8) << 5) | ACCEL_ODR as u8] // Accel Settings
        ];

        for command in commands {
            self.spi_write_reg(
                command[0],
                command[1]
            ).await?;
        }

        Ok(())
    }


}