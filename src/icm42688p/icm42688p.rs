
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
use stm32f4xx_hal::prelude::{_stm32f4xx_hal_spi_SpiExt, _stm32f4xx_hal_timer_TimerExt};
use stm32f4xx_hal::rcc::{Clocks, Rcc};
use stm32f4xx_hal::spi::{Mode, Phase, Polarity, Spi};
use stm32f4xx_hal::gpio::Input;
use ufmt::{uDisplay, Formatter, uwrite};
use super::super::util::util::{DisplayFloat};

// spi Setup
pub fn imu_setup(
    spi: SPI2,
    cs: PB12<Input>,
    sck: PB13<Input>,
    miso: PB14<Input>,
    mosi: PB15<Input>,
    rcc: &mut Rcc,
    tim2: TIM2
) -> (Icm42688p<ExclusiveDevice<Spi<SPI2>, PB12<Output<PushPull>>, DelayUs<TIM2>>>, IMUData) {

    let cs = cs.into_push_pull_output();
    let sck = sck.into_alternate::<5>();
    let miso = miso.into_alternate::<5>();
    let mosi = mosi.into_alternate::<5>();

    // Spi peripheral configuration
    let spi = spi.spi(
        (Some(sck), Some(miso), Some(mosi)),
        Mode { polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition },
        12.MHz(),
        rcc
    );

    let delay = tim2.delay_us(rcc);

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
             | X: {} Y: {} Z: {}        \n\
             |                          \n\
             | Gyro                     \n\
             | X: {} Y: {} Z: {}        \n\
             +--------------------------\n",
            DisplayFloat(self.accel.accel_x), DisplayFloat(self.accel.accel_y),
            DisplayFloat(self.accel.accel_z), DisplayFloat(self.gyro.gyro_x),
            DisplayFloat(self.gyro.gyro_y), DisplayFloat(self.gyro.gyro_z)
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
        let starting_addr: [u8; 1]  = [DATA_READ_START_REG | 0x80];

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
            accel_x: (((buf[0] as u16) << 8 | buf[1] as u16) as i16 as f32) * ACCEL_SENS_FACTOR,
            accel_y: (((buf[2] as  u16) << 8 | buf[3] as  u16) as i16 as f32) * ACCEL_SENS_FACTOR,
            accel_z: (((buf[4] as  u16) << 8 | buf[5] as  u16) as i16 as f32) * ACCEL_SENS_FACTOR,
        };

        self.data.gyro = Gyro {
            gyro_x: (((buf[6] as  u16) << 8 | buf[7] as u16) as i16 as f32) * GYRO_SENS_FACTOR,
            gyro_y: (((buf[8] as  u16) << 8 | buf[9] as  u16) as i16 as f32) * GYRO_SENS_FACTOR,
            gyro_z: (((buf[10] as  u16) << 8 | buf[11] as  u16) as i16 as f32) * GYRO_SENS_FACTOR,
        };

        if INTERRUPTS_ENABLED {
            // Clean the flags register
            let _flags = self.spi_read_reg(
                Bank0::IntStatus as u8
            ).await?;
        }


        // For debug printing
        macro_rules! split_float {
            ($val:expr) => {{
                let sign = if $val < 0.0 { "-" } else { "" };
                let abs_val = $val.abs();
                let int_part = abs_val as i32;
                let frac_part = ((abs_val - int_part as f32) * 1000.0) as u32;
                (sign, int_part, frac_part)
            }};
        }

        // Extract the safe parts
        let (ax_s, ax_i, ax_f) = split_float!(self.data.accel.accel_x);
        let (ay_s, ay_i, ay_f) = split_float!(self.data.accel.accel_y);
        let (az_s, az_i, az_f) = split_float!(self.data.accel.accel_z);
        let (gx_s, gx_i, gx_f) = split_float!(self.data.gyro.gyro_x);
        let (gy_s, gy_i, gy_f) = split_float!(self.data.gyro.gyro_y);
        let (gz_s, gz_i, gz_f) = split_float!(self.data.gyro.gyro_z);

        // Log with the explicit sign slot '{=str}' directly in front of the integer block
        defmt::println!(
            "Accel X: {=str}{=i32}.{=u32:03} Y: {=str}{=i32}.{=u32:03} Z: {=str}{=i32}.{=u32:03} Gyro X: {=str}{=i32}.{=u32:03} Y: {=str}{=i32}.{=u32:03} Z: {=str}{=i32}.{=u32:03}",
            ax_s, ax_i, ax_f,
            ay_s, ay_i, ay_f,
            az_s, az_i, az_f,
            gx_s, gx_i, gx_f,
            gy_s, gy_i, gy_f,
            gz_s, gz_i, gz_f
        );

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
        let mut buf: [u8; 2] = [addr, data];
        self.spi.transfer_in_place(&mut buf)?;
        Ok(())
    }

    async fn spi_read_reg(&mut self, addr: u8) -> Result<u8, SPI::Error> {
        let mut buf: [u8; 2] = [addr | (0b1 << 7), 0];
        self.spi.transfer_in_place(&mut buf)?; // writes and then overwrites the buffer
        Ok(buf[1]) // return the buffered value
    }

    pub async fn startup(&mut self, delay: &mut impl DelayNs) -> Result<(), ImuError<SPI::Error>> {

        // Power on delay
        delay.delay_ms(100).await;

        // Verify the Who am I
        let _who_am_i = self.spi_read_reg(Bank0::WhoAmI as u8).await?;

        delay.delay_ms(100).await;

        self.start_sensors().await?;

        delay.delay_ms(100).await;

        self.set_sensor_config().await?;

        // Delay for startup time
        delay.delay_ms(100).await;

        Ok(())
    }

    async fn start_sensors(&mut self) -> Result<(), ImuError<SPI::Error>> {

        self.spi_write_reg(
            Bank0::PwrMGMT0 as u8,
            (ACTIVE_CMD << 4) |(GYRO_PWR_CMD << 2) | ACCEL_PWR_CMD
        ).await?;

        Ok(())
    }

    async fn soft_reset(&mut self) -> Result<(), ImuError<SPI::Error>> {

        self.spi_write_reg(
            Bank0::DeviceConfig as u8,
            0b1
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