#![no_std]
#![no_main]

pub mod cli;
pub mod util;


extern crate alloc;

use defmt_rtt as _;
use panic_probe as _;

use rtic::app;
use embedded_alloc::LlffHeap as Heap;
use rtic_monotonics::fugit::{TimerDurationU32, TimerDurationU64};

// Subject to change
const HEAP_SIZE: usize = 4096;
#[global_allocator]
static HEAP: Heap = Heap::empty();

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use rtic_monotonics::Monotonic;
    use rtic_monotonics::stm32_tim5_monotonic;
    use super::*;

    use super::cli::cli::{QuadCli, Writer};
    use stm32f4xx_hal::prelude::*;



    // CLI
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
    use rtic_monotonics::fugit::ExtU64;
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use stm32f4xx_hal::serial::{Rx, Tx, Serial, Event};
    use usb_device::bus::UsbBusAllocator;

    use stm32f4xx_hal::timer::DelayUs;
    use stm32f4xx_hal::dma::{DmaFlag, PeripheralToMemory, Stream2, StreamsTuple, Transfer};

    use stm32f4xx_hal::rcc::Config;
    use stm32f4xx_hal::timer::{CounterUs};

    stm32_tim5_monotonic!(Mono, 1_000_000);

    // CLI
    static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;
    static mut SER: Option<SerialPort<'static, UsbBus<USB>>> = None;
    static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // 4096 bytes of memory for the serial port

    #[shared]
    struct Shared {

    }

    #[local]
    struct Local {

        // Cli
        cli: QuadCli,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Clocks
        let dp = cx.device;

        let mut rcc = dp.RCC.freeze(
            Config::hse(25.MHz())
                .sysclk(96.MHz())
        );

        // Init the heap
        unsafe { embedded_alloc::init!(HEAP, HEAP_SIZE); }

        // delay clock startup
        Mono::start(rcc.clocks.timclk1().raw());

        // CTRL CODE HERE

        // Gpio pins
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);


        // CLI SETUP
        // ---------------------------------------------------------------------

        // CLI
        // ---------------------------------------------

        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
            &rcc.clocks
        );

        // USB setup
        let (usb_dev, writer) = unsafe {
            USB_BUS = Some(UsbBus::new(usb, &mut EP_MEMORY));
            let usb_bus_ref = USB_BUS.as_ref().unwrap();
            let ser = SerialPort::new(usb_bus_ref);

            let usb_dev = UsbDeviceBuilder::new(
                usb_bus_ref,
                UsbVidPid(0x16c0, 0x27dd))
                .device_class(usbd_serial::USB_CLASS_CDC)
                .build();

            SER = Some(ser);
            let writer = Writer { ser: &raw mut *SER.as_mut().unwrap() };
            (usb_dev, writer)
        };

        // CLI buffers
        let (command_buffer, history_buffer) = unsafe {
            static mut COMMAND_BUFFER: [u8; 40] = [0; 40];
            static mut HISTORY_BUFFER: [u8; 41] = [0; 41];

            #[allow(static_mut_refs)]
            (COMMAND_BUFFER.as_mut(), HISTORY_BUFFER.as_mut())
        };

        // Build the CLI object
        let cli = CliBuilder::default()
            .writer(writer)
            .command_buffer(command_buffer)
            .history_buffer(history_buffer)
            .build()
            .expect("Cli failed to init");

        // Pass the cli object to our cli
        let cli = QuadCli::new(cli);

        //  -------------------------------------------

        // GPS SETUP

        (Shared {
        }, Local {
            cli,
            usb_dev,
        })
    } // INIT

    // Try CLI here
    #[idle(local = [cli, usb_dev])]
    fn idle(mut _cx: idle::Context) -> ! {
        let cli = _cx.local.cli;
        let usb_dev = _cx.local.usb_dev;
        let ser = unsafe { SER.as_mut().unwrap() };

        ser.write(b"Quadcopter Online\n").unwrap();

        // Write base cli
        loop {

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
    } // CLI
} // App
