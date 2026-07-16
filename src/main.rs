#![no_std]
#![no_main]

pub mod cli;
pub mod icm42688p;
pub mod util;

extern crate alloc;

use panic_probe as _;
use defmt_rtt as _;      // transport backend

use rtic::app;
use embedded_alloc::LlffHeap as Heap;
use rtic_monotonics::fugit::TimerDurationU64;

// Subject to change
const HEAP_SIZE: usize = 4096;
const IMU_TIME_US: TimerDurationU64<1_000_000> = TimerDurationU64::from_ticks(250); // 250 micros

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {

    use rtic_monotonics::Monotonic;
    use rtic_monotonics::stm32_tim5_monotonic;

    use super::*;
    use super::icm42688p::icm42688p::{
        Icm42688p, IMUData, imu_setup
    };

    use super::cli::cli::{ Writer };
    use stm32f4xx_hal::prelude::*;

    use embedded_hal_bus::spi::ExclusiveDevice;
    use stm32f4xx_hal::gpio::{
        PC13, Output, PushPull,     // LED
        PB12
    };
    use stm32f4xx_hal::pac::*;
    use stm32f4xx_hal::spi::{Spi};

    use stm32f4xx_hal::otg_fs::{USB, UsbBus};
    use usbd_serial::SerialPort;
    use embedded_cli::cli::{CliBuilder};
    use embedded_io::Write;
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use usb_device::bus::UsbBusAllocator;
    use super::cli::cli::QuadCli;
    use stm32f4xx_hal::timer::DelayUs;
    use stm32f4xx_hal::rcc::Config;

    stm32_tim5_monotonic!(Mono, 1_000_000);

    static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;
    static mut SER: Option<SerialPort<'static, UsbBus<USB>>> = None;
    static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // 4096 bytes of memory for the serial port

    #[shared]
    struct Shared {
        imu_data: IMUData
    }

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>,
        cli: QuadCli,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,

        imu:  Icm42688p<ExclusiveDevice<Spi<SPI2>, PB12<Output<PushPull>>, DelayUs<TIM2>>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        
        let dp = cx.device;

        let mut rcc = dp.RCC.freeze(
            Config::hse(25.MHz())
                .sysclk(96.MHz())
            // add .require_pll48clk() if you need USB, etc.
        );

        // Init the heap
        unsafe {
            embedded_alloc::init!(HEAP, HEAP_SIZE);
        }

        // delay clock startup
        Mono::start(rcc.clocks.timclk1().raw());

        defmt::println!("BOOT");

        // CTRL CODE HERE

        // Gpio pins
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        // IMU
        let (imu, imu_data) = imu_setup(
            dp.SPI2,       // SPI instance
            gpiob.pb12,    // CS
            gpiob.pb13,    // SCK
            gpiob.pb14,    // MISO
            gpiob.pb15,    // MOSI
            &mut rcc,       // Clock reference
            dp.TIM2,
        );

        // CLI SETUP

        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
            &rcc.clocks 
        );

        let (usb_dev, writer) = unsafe {
            USB_BUS = Some(UsbBus::new(usb, &mut EP_MEMORY));
            let usb_bus_ref = USB_BUS.as_ref().unwrap();
            let ser = SerialPort::new(usb_bus_ref);
            let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x16c0, 0x27dd))
                .device_class(usbd_serial::USB_CLASS_CDC)
                .build();
            SER = Some(ser);
            let writer = Writer { ser: &raw mut *SER.as_mut().unwrap() };
            (usb_dev, writer)
        };

        let (command_buffer, history_buffer) = unsafe {
            static mut COMMAND_BUFFER: [u8; 40] = [0; 40];
            static mut HISTORY_BUFFER: [u8; 41] = [0; 41];

            #[allow(static_mut_refs)]
            (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
        };

        let cli = CliBuilder::default()
            .writer(writer)
            .command_buffer(command_buffer)
            .history_buffer(history_buffer)
            .build()
            .expect("Cli failed to init");

        let cli = QuadCli::new(
            cli
        );

        // // -------------------------------------------

        imu_update::spawn().unwrap();

        (Shared {
            imu_data
        }, Local {
            led, cli, usb_dev, imu
        })
    }

    // Try CLI here
    #[idle(local = [cli, usb_dev], shared=[imu_data])]
    fn idle(mut _cx: idle::Context) -> ! {
        let mut cli = _cx.local.cli;
        let usb_dev = _cx.local.usb_dev;
        let ser = unsafe { SER.as_mut().unwrap() };
        let mut imu = _cx.shared.imu_data;

        ser.write_all(b"Quadcopter Online\n").unwrap();

        // Write base cli
        loop {

            let imu_data = imu.lock(|imu| { imu.clone() });

            if usb_dev.poll(&mut [ser]) {

                // 64 serial characters
                let mut buf: [u8; 64] = [0; 64];
                if let Ok(count) = ser.read(&mut buf) {
                    for byte in &buf[0..count] {
                        cli.process(*byte);
                    }

                }
            }
        }
    }

    // IMU is the highest priority for flight stability
    #[task(local = [imu], shared = [imu_data], priority = 4)]
    async fn imu_update(mut _cx: imu_update::Context) {
        let imu = _cx.local.imu;
        let mut imu_data = _cx.shared.imu_data;

        imu.startup(&mut Mono).await.unwrap();

        loop {
            let next = Mono::now() + IMU_TIME_US;

            imu.update().await.unwrap();

            imu_data.lock(|imu_data| {
                *imu_data = imu.get_data();
            });

            Mono::delay_until(next).await;
        }
    } // IMU
}
