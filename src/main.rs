#![no_std]
#![no_main]

mod icm42688p;

extern crate alloc;

use panic_halt as _;

use rtic::app;
use rtic_monotonics::systick::prelude::*;
use embedded_alloc::LlffHeap as Heap;

// Subject to change
const HEAP_SIZE: usize = 4096;
#[global_allocator]
static HEAP: Heap = Heap::empty();

systick_monotonic!(Mono, 1_000_000);

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use super::*;
    use super::icm42688p::icm42688p::{
        Icm42688p, IMUData, Gyro, Accel
    };
    use stm32f4xx_hal::prelude::*;
    use stm32f4xx_hal::gpio::{
        PC13, Output, PushPull,     // LED
        PB12
    };
    use stm32f4xx_hal::pac::*;
    use stm32f4xx_hal::spi::{Spi, Mode, Polarity, Phase};
    use embedded_hal_bus::spi::ExclusiveDevice;

    #[shared]
    struct Shared {
        imu_data: IMUData
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        imu:  Icm42688p<ExclusiveDevice<Spi<SPI2>, PB12<Output<PushPull>>, Mono>, Mono>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let dp = cx.device;

        // Clocks
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(96.MHz()).freeze();

        // Init the heap
        unsafe {
            embedded_alloc::init!(HEAP, HEAP_SIZE);
        }

        // delay clock startup
        Mono::start(cx.core.SYST, clocks.sysclk().to_Hz());

        let gpioc = dp.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output();

        // IMU SPI
        let gpiob = dp.GPIOB.split();
        let cs1 = gpiob.pb12.into_push_pull_output();
        let sck1 = gpiob.pb13.into_alternate();
        let miso1 = gpiob.pb14.into_alternate();
        let mosi1 = gpiob.pb15.into_alternate();

        let spi1 = dp.SPI2.spi(
            (sck1, miso1, mosi1),
            Mode { polarity: Polarity::IdleHigh, phase: Phase::CaptureOnFirstTransition },
            10_u32.MHz(),
            &clocks
        );

        // Exclusive Device is an spi peripheral
        let imu_spi = ExclusiveDevice::new(spi1, cs1, Mono).unwrap();

        let imu = Icm42688p::new(imu_spi, Mono);

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

        blink::spawn().unwrap();
        update::spawn().unwrap();

        (Shared {
            imu_data
        }, Local {
            led, imu
        })
    }

    // Try CLI here
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {

        }
    }

    #[task(local=[led], priority = 1)]
    async fn blink(_cx: blink::Context) {
        let led = _cx.local.led;
        loop {
            led.toggle();
            Mono::delay(1000.millis()).await; // Wait 500 milliseconds
        }
    }

    #[task(local = [imu], priority = 2)]
    async fn update(mut _cx: update::Context) {
        let imu = _cx.local.imu;
        imu.startup().await.unwrap();
        loop {
            imu.update().await.unwrap();
            Mono::delay(250.micros()).await; // 4KHz res
        }
    }
}
