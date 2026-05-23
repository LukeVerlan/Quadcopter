#![no_std]
#![no_main]

pub mod cli;
pub mod neom8n;

extern crate alloc;

use defmt_rtt as _;
use panic_probe as _;

use rtic::app;
use rtic_monotonics::systick::prelude::*;
use embedded_alloc::LlffHeap as Heap;

// Subject to change
const HEAP_SIZE: usize = 4096;

#[global_allocator]
static HEAP: Heap = Heap::empty();

systick_monotonic!(Mono, 10_000);

defmt::timestamp!("{=u32}", { 0u32 });

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use super::*;
    use super::cli::cli::{
        process, Writer
    };
    use stm32f4xx_hal::prelude::*;
    use stm32f4xx_hal::gpio::{PC13, Output, PushPull};

    // CLI
    use stm32f4xx_hal::otg_fs::{USB, UsbBus};
    use usbd_serial::SerialPort;
    use embedded_cli::cli::{Cli, CliBuilder};
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use usb_device::bus::UsbBusAllocator;
    use core::convert::Infallible;
    use stm32f4xx_hal::serial::{Rx, Tx};
    use ufmt::uwrite;

    // GPS
    use super::neom8n::neom8n::{Neom8n, GpsData, gps_setup};

    use stm32f4xx_hal::pac::USART2;

    // CLI
    static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;
    static mut SER: Option<SerialPort<'static, UsbBus<USB>>> = None;
    static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // 4096 bytes of memory for the serial port


    #[shared]
    struct Shared {

        // GPS
        gps_data: GpsData,
        gps: Neom8n<Rx<USART2>, Tx<USART2>>

    }

    #[local]
    struct Local {

        // LED
        led: PC13<Output<PushPull>>,

        // CLI
        cli: Cli<Writer, Infallible, &'static mut [u8], &'static mut [u8]>,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,
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

        // Gpio pins
        let gpioa = dp.GPIOA.split();
        let gpioc = dp.GPIOC.split();

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        // CLI SETUP

        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
            &clocks
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

        // // -------------------------------------------

        // GPS SETUP
        
        let mut gps = gps_setup(
            dp.USART2,
            gpioa.pa2,
            gpioa.pa3,
            &clocks
        );

        let gps_data = gps.get_data();

        blink::spawn().unwrap();

        (Shared {
            gps_data, gps
        }, Local {
            led, cli, usb_dev,
        })
    }

    // Try CLI here
    #[idle(local = [cli, usb_dev])]
    fn idle(mut _cx: idle::Context) -> ! {
        let mut cli = _cx.local.cli;
        let usb_dev = _cx.local.usb_dev;
        let ser = unsafe { SER.as_mut().unwrap() };

        let _ = cli.write(|writer| {
            uwrite!(writer,"{}","Quadcopter Online")?;
            Ok(())
        });

        // Write base cli
        loop {

            if usb_dev.poll(&mut [ser]) {

                // 64 serial characters
                let mut buf: [u8; 64] = [0; 64];
                if let Ok(count) = ser.read(&mut buf) {
                    for byte in &buf[0..count] {
                        process(&mut cli, *byte);
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
            Mono::delay(1000.millis()).await; // Wait 500 milliseconds
        }
    }

    // GPS UART RX interrupt handler
    #[task(binds = USART2, shared=[gps])]
    fn receive_gps_message(_cx: receive_gps_message::Context) {
        let mut gps = _cx.shared.gps;
        gps.lock(|gps| {
                let message_built = gps.build_message();

                if message_built {
                    parse_gps_message::spawn().unwrap();
                }
        })
    }

    #[task(shared=[gps], priority = 2)]
    async fn parse_gps_message(_cx: parse_gps_message::Context) {
        let mut gps = _cx.shared.gps;
        gps.lock(|gps| {
            gps.parse_message().unwrap();;
        });
    }


}
