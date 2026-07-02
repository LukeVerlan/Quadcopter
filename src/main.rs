#![no_std]
#![no_main]

pub mod cli;
pub mod util;
pub mod x4r;

extern crate alloc;

use panic_probe as _;
use defmt_rtt as _;      // transport backend

use rtic::app;
use rtic_monotonics::systick::prelude::*;
use embedded_alloc::LlffHeap as Heap;

// Subject to change
const HEAP_SIZE: usize = 4096;

#[global_allocator]
static HEAP: Heap = Heap::empty();

systick_monotonic!(Mono, 10_000);

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use super::*;
    use super::cli::cli::{ Writer };
    use stm32f4xx_hal::prelude::*;

    use stm32f4xx_hal::gpio::{ PC13, Output, PushPull };
    use stm32f4xx_hal::serial::config::{DmaConfig, Parity, StopBits, WordLength};
    use stm32f4xx_hal::otg_fs::{USB, UsbBus};
    use usbd_serial::SerialPort;
    use stm32f4xx_hal::time::Bps;
    use embedded_cli::cli::{CliBuilder};
    use embedded_io::Write;
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use stm32f4xx_hal::serial::{Config, Tx, Rx, Serial};
    use usb_device::bus::UsbBusAllocator;
    use stm32f4xx_hal::pac::*;
    use crate::x4r::x4r::{
        X4rData, X4r,
        X4R_DMA_BUFFER1, X4R_DMA_BUFFER2
    };
    use super::cli::cli::QuadCli;

    use stm32f4xx_hal::dma::config::DmaConfig as SerialDmaConfig;


    static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;
    static mut SER: Option<SerialPort<'static, UsbBus<USB>>> = None;
    static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // 4096 bytes of memory for the serial port

    #[shared]
    struct Shared {
        x4r_data: X4rData
    }

    #[local]
    struct Local {

        // Onboard led
        led: PC13<Output<PushPull>>,

        // Cli
        cli: QuadCli,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,

        // Telemetry
        x4r: X4r<Rx<USART1>>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {

        let dp = cx.device;

        // Clocks
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(96.MHz()).freeze();

        // Init the heap
        unsafe { embedded_alloc::init!(HEAP, HEAP_SIZE); }

        // delay clock startup
        Mono::start(cx.core.SYST, clocks.sysclk().to_Hz());

        defmt::println!("BOOT");

        // GPIO pins
        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        // CLI SETUP
        // ---------------------------------------------------------------------

        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
            &clocks
        );

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

        // -------------------------------------------

        // Telemetry setup
        // ------------------------------------------

        let x4r_config = Config {
            baudrate: Bps(100_000),
            wordlength: WordLength::DataBits8,
            parity: Parity::ParityEven,
            dma: DmaConfig::None, // Change to use DMA
            stopbits: StopBits::STOP2
        };

        // Telemetry Startup
        let telem_usart = Serial::<USART1, u8>::new(
            dp.USART1, (gpioa.pa9.into_alternate(), gpioa.pa10.into_alternate()), x4r_config, &clocks
        );

        let (_tx, rx) = telem_usart.unwrap().split();

        let x4r = X4r::new(rx);
        let x4r_data = x4r.get_data();

        // ---------------------------------------------

        blink::spawn().unwrap();

        (Shared {
            x4r_data
        }, Local {
            led, cli, usb_dev, x4r
        })
    }

    // Try CLI here
    #[idle(local = [cli, usb_dev])]
    fn idle(mut _cx: idle::Context) -> ! {

        let cli = _cx.local.cli;
        let usb_dev = _cx.local.usb_dev;
        let ser = unsafe { SER.as_mut().unwrap() };

        ser.write_all(b"Quadcopter Online\n").unwrap();

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

    }

    #[task(local=[led], priority = 1)]
    async fn blink(_cx: blink::Context) {
        let led = _cx.local.led;
        loop {
            led.toggle();
            Mono::delay(1_000.millis()).await; // Wait 500 milliseconds
        }
    }

    #[task(binds=USART1, local=[x4r])]
    fn rx_telemetry_message(cx: rx_telemetry_message::Context) {
        let x4r = cx.local.x4r;

    }

    

}
