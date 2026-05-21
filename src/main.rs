#![no_std]
#![no_main]

mod icm42688p;
mod cli;

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

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use super::*;
    use super::icm42688p::icm42688p::{
        Icm42688p, IMUData, Gyro, Accel
    };
    use super::cli::cli::Writer;
    use stm32f4xx_hal::prelude::*;

    use embedded_hal_bus::spi::ExclusiveDevice;
    use stm32f4xx_hal::gpio::{
        PC13, Output, PushPull,     // LED
        PB12
    };
    use stm32f4xx_hal::pac::*;
    use stm32f4xx_hal::spi::{Spi, Mode, Polarity, Phase};

    use stm32f4xx_hal::otg_fs::{USB, UsbBus};
    use usbd_serial::SerialPort;
    use embedded_cli::cli::{Cli, CliBuilder};
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use usb_device::bus::UsbBusAllocator;
    use core::convert::Infallible;
    use ufmt::uwrite;
    use ufmt::uwriteln;

    #[shared]
    struct Shared {
        imu_data: IMUData
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        imu:  Icm42688p<ExclusiveDevice<Spi<SPI2>, PB12<Output<PushPull>>, Mono>, Mono>,
        cli: Cli<Writer, Infallible, &'static mut [u8], &'static mut [u8]>,
        usb_dev: UsbDevice<'static, UsbBus<USB>>
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

        // CTRL CODE HERE

        static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;

        // Gpio pins
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        // IMU Setup

        // Pin Definitions
        let cs1 = gpiob.pb12.into_push_pull_output();
        let sck1 = gpiob.pb13.into_alternate();
        let miso1 = gpiob.pb14.into_alternate();
        let mosi1 = gpiob.pb15.into_alternate();

        // Spi peripheral configuration
        let spi1 = dp.SPI2.spi(
            (sck1, miso1, mosi1),
            Mode { polarity: Polarity::IdleHigh, phase: Phase::CaptureOnFirstTransition },
            10_u32.MHz(),
            &clocks
        );

        // Create embedded_hal spi device, exclusive device is for a dedicated SPI channel
        let imu_spi = ExclusiveDevice::new(spi1, cs1, Mono).unwrap();

        // Create imu object
        let imu = Icm42688p::new(imu_spi, Mono);

        // Shared data struct
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

        // ---------------------------------------

        // CLI SETUP

        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
            &clocks
        );

        static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // 4096 bytes of memory for the serial port

        let (ser, usb_dev) = unsafe {
            USB_BUS = Some(UsbBus::new(usb, &mut EP_MEMORY));
            let usb_bus_ref = USB_BUS.as_ref().unwrap();
            let ser = SerialPort::new(usb_bus_ref);
            let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x16c0, 0x27dd))
                .device_class(usbd_serial::USB_CLASS_CDC)
                .build();
            (ser, usb_dev)
        };

        let (command_buffer, history_buffer) = unsafe {
            static mut COMMAND_BUFFER: [u8; 40] = [0; 40];
            static mut HISTORY_BUFFER: [u8; 41] = [0; 41];

            #[allow(static_mut_refs)]
            (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
        };

        let writer = Writer { ser };

        let mut cli = CliBuilder::default()
            .writer(writer)
            .command_buffer(command_buffer)
            .history_buffer(history_buffer)
            .build()
            .expect("Cli failed to init");

        // -------------------------------------------

        blink::spawn().unwrap();
        imu_update::spawn().unwrap();

        (Shared {
            imu_data
        }, Local {
            led, imu, cli, usb_dev
        })
    }

    // Try CLI here
    #[idle(local = [cli, usb_dev], shared = [imu_data])]
    fn idle(mut _cx: idle::Context) -> ! {
        let cli = _cx.local.cli;
        let imu_data = _cx.shared.imu_data;

        let _ = cli.write(|writer| {

            uwrite!(
                writer,
                "{}",
                "Quadcopter Online"
            )?;

            Ok(())

        });

        // Write base cli
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

    #[task(local = [imu], shared = [imu_data], priority = 2)]
    async fn imu_update(mut _cx: imu_update::Context) {
        let imu = _cx.local.imu;
        imu.startup().await.unwrap();
        loop {
            imu.update().await.unwrap();
            Mono::delay(250.micros()).await; // 4KHz res
        }
    }

}
