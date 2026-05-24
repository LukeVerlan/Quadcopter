
/// LONG TERM
// TODO : Functionality -> Interrupt setup (INT_CONFIG, Identify Pin Number)
// As long as Accel and Gyro have same sample rate, when this flag goes off just burst read the whole shit
// TODO: Learn about notch filtering and Anti-Alias for better data quality

/// NOW!!
// TODO: comment functions

// hal interface
use embedded_hal::spi::{Operation, SpiDevice};
use embedded_hal_async::delay::DelayNs;
use embedded_hal_bus::spi::ExclusiveDevice;
use rtic_monotonics::fugit::RateExtU32;
use stm32f4xx_hal::timer::delay::DelayUs;
use stm32f4xx_hal::pac::{SPI2, TIM2};
// Bank Numbers
use super::reg::{Bank0, DATA_READ_LEN, DATA_READ_START_REG};

// Sensor Setup
use super::config::*;
use stm32f4xx_hal::gpio::{Output, PushPull, PB12, PB13, PB14, PB15};
use stm32f4xx_hal::prelude::_stm32f4xx_hal_spi_SpiExt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Mode, Phase, Polarity, Spi};
use stm32f4xx_hal::gpio::Input;
use ufmt::{uDisplay, Formatter, uwrite};
use super::super::util::util::{DisplayF32};



// spi Setup
pub fn imu_setup(
    spi: SPI2,
    cs: PB12<Input>,
    sck: PB13<Input>,
    miso: PB14<Input>,
    mosi: PB15<Input>,
    clocks: &Clocks,
    delay: DelayUs<TIM2>
) -> (Icm42688p<ExclusiveDevice<Spi<SPI2>, PB12<Output<PushPull>>, DelayUs<TIM2>>>, IMUData) {

    let cs = cs.into_push_pull_output();
    let sck = sck.into_alternate::<5>();
    let miso = miso.into_alternate::<5>();
    let mosi = mosi.into_alternate::<5>();
    
    // Spi peripheral configuration
    let spi = spi.spi(
        (sck, miso, mosi),
        Mode { polarity: Polarity::IdleHigh, phase: Phase::CaptureOnFirstTransition },
        10_u32.MHz(),
        clocks
    );

    let imu_spi = ExclusiveDevice::new(spi, cs, delay).unwrap();

    let imu_data = IMUData {

        accel: Accel {
            accel_x: f32::NAN,
            accel_y: f32::NAN,
            accel_z: f32::NAN,
        },

        gyro: Gyro {
            gyro_x: f32::NAN,
            gyro_y: f32::NAN,
            gyro_z: f32::NAN,
        }
    };

    // Create imu object
    (Icm42688p::new(imu_spi), imu_data)
}

#[derive(Copy, Clone)]
pub struct IMUData {
    pub accel: Accel,
    pub gyro: Gyro,
}

impl uDisplay for IMUData {
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error> {
        uwrite!(f,
            "+--------------------------\n\
             | Accel                    \n\
             | X: {} Y:{} Z:{}          \n\
             |                          \n\
             | Gyro                     \n\
             | X: {} Y: {} Z: {}        \n\
             +-----------------------------\n",
            DisplayF32(self.accel.accel_x), DisplayF32(self.accel.accel_y),
            DisplayF32(self.accel.accel_z), DisplayF32(self.gyro.gyro_x),
            DisplayF32(self.gyro.gyro_y), DisplayF32(self.gyro.gyro_z)
        )
    }
}

// Accel Data -- G forces
#[derive(Copy, Clone)]
pub struct Accel {
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,
}

// Gyro Data -- degrees / second
#[derive(Copy, Clone)]
pub struct Gyro {
    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,
}

#[derive(Debug)]
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
pub struct Icm42688p<SPI> {
    spi: SPI,
    pub data: IMUData
}

// Constructor
impl <SPI> Icm42688p<SPI> {
    pub fn new(spi: SPI) -> Self
    where
        SPI: SpiDevice + 'static,
    {
        // Return reference to imu
        Icm42688p {

            spi,

            data: IMUData {
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
}

// Exposed functions
impl <SPI: SpiDevice + 'static> Icm42688p<SPI> {

    pub fn get_data(&mut self) -> IMUData {
        self.data
    }

    pub async fn update(&mut self) -> Result<(), ImuError<SPI::Error>> {

        // Grab the data
        let starting_addr: [u8; 1]  = [DATA_READ_START_REG];

        // Burst read the data
        let mut buf: [u8; DATA_READ_LEN] = [0; DATA_READ_LEN];
        self.spi.transaction(&mut [
            Operation::Write(&starting_addr),
            Operation::Read(&mut buf)
        ])?;

        // Comes over as Accel xyz (MSB, LSB) for 6 bytes
        // Then Gyro xyz (MSB, LSB) for 6 bytes

        // Update the struct values
        self.data.accel = Accel {
            accel_x: (((buf[0] as i16) << 8 | buf[1] as i16) as f32) * ACCEL_SENS_FACTOR,
            accel_y: (((buf[2] as i16) << 8 | buf[3] as i16) as f32) * ACCEL_SENS_FACTOR,
            accel_z: (((buf[4] as i16) << 8 | buf[5] as i16) as f32) * ACCEL_SENS_FACTOR,
        };

        self.data.gyro = Gyro {
            gyro_x: (((buf[6] as i16) << 8 | buf[7] as i16) as f32) * GYRO_SENS_FACTOR,
            gyro_y: (((buf[8] as i16) << 8 | buf[9] as i16) as f32) * GYRO_SENS_FACTOR,
            gyro_z: (((buf[10] as i16) << 8 | buf[11] as i16) as f32) * GYRO_SENS_FACTOR,
        };

        if INTERRUPTS_ENABLED {
            // Clean the flags register
            let flags = self.spi_read_reg(
                Bank0::IntStatus as u8
            ).await?;
        }

        Ok(())
    }

    // Enables interrupts on the INT1 pin
    async fn enable_int(&mut self) -> Result<(), ImuError<SPI::Error>> {
        self.spi_write_reg(
            Bank0::IntConfig as u8,
            (0b1 << 2) as u8
        ).await?;

        Ok(())
    }

    // TODO: Notch filter setup
    async fn notch_filter_setup(&mut self){
        // Implement notch filter setup
    }

    // TODO: Anti Alias Filter
    async fn aaf_filter_setup(&mut self) {
        // Implement anti-alias filtering setup
    }

    async fn spi_write_reg(&mut self, addr: u8, data: u8) -> Result<(), ImuError<SPI::Error>> {
        let buf: [u8; 2] = [addr & (0b0 << 7), data];
        self.spi.write(&buf)?;
        Ok(())
    }

    async fn spi_read_reg(&mut self, addr: u8) -> Result<u8, SPI::Error> {
        let mut buf: [u8; 1] = [addr & (0b1 << 7)];
        self.spi.transfer_in_place(&mut buf)?; // writes and then overwrites the buffer
        Ok(buf[0]) // return the buffered value
    }

    pub async fn startup(&mut self, delay: &mut impl DelayNs) -> Result<(), ImuError<SPI::Error>> {

        // Power on delay
        delay.delay_ms(100).await;

        // Verify the Who am I
        let who_am_i = self.spi_read_reg(Bank0::WhoAmI as u8).await?;
        if who_am_i != WHO_AM_I {
            return Err(ImuError::WhoAmI);
        }

        // Reset for clarity
        self.soft_reset().await?;

        delay.delay_ms(100).await;

        self.set_sensor_config().await?;

        // Delay for startup time
        delay.delay_ms(100).await;

        Ok(())
    }

    async fn start_sensors(&mut self) -> Result<(), ImuError<SPI::Error>> {

        self.spi_write_reg(
            Bank0::PwrMGMT0 as u8,
            (GYRO_PWR_CMD << 2) | ACCEL_PWR_CMD
        ).await?;

        Ok(())
    }

    async fn soft_reset(&mut self) -> Result<(), ImuError<SPI::Error>> {

        self.spi_write_reg(
            Bank0::DeviceConfig as u8,
            0b1
        ).await?;

        // Clear the native interrupt flag by reading
        self.spi_read_reg(
            Bank0::IntStatus as u8
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