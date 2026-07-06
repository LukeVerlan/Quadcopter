#![no_std]
#![no_main]

const X4R_DMA_CHANNEL: u8 = 4;
pub mod cli;
pub mod util;
pub mod x4r;

extern crate alloc;

use panic_probe as _;
use defmt_rtt as _;

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
    use stm32f4xx_hal::otg_fs::{USB, UsbBus};
    use usbd_serial::SerialPort;
    use embedded_cli::cli::{CliBuilder};
    use embedded_io::Write;
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use stm32f4xx_hal::serial::{Rx, Serial, Event};
    use usb_device::bus::UsbBusAllocator;
    use stm32f4xx_hal::pac::*;
    use stm32f4xx_hal::dma::{DmaFlag, PeripheralToMemory, Stream2, StreamsTuple, Transfer};
    use crate::x4r::x4r::{X4rData, X4r, X4rError, SBUS_MESSAGE_LENGTH, X4R_SBUS_CONFIG, get_x4r_dma_config};
    use super::cli::cli::QuadCli;

    use stm32f4xx_hal::rcc::Config;

    type X4rDmaTransfer = // From the DMA Table
        Transfer<Stream2<DMA2>, 4, Rx<USART1, u8>, PeripheralToMemory, &'static mut [u8; SBUS_MESSAGE_LENGTH]>;

    static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;
    static mut SER: Option<SerialPort<'static, UsbBus<USB>>> = None;
    static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // Serial port
    #[shared]
    struct Shared {

        // Telem
        x4r_dma: Option<X4rDmaTransfer>,
        x4r_data: X4rData,
    }

    #[local]
    struct Local {

        // Onboard led
        led: PC13<Output<PushPull>>,

        // Cli
        cli: QuadCli,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,

        // Telemetry
        x4r: X4r,
        x4r_dma_buf: Option<&'static mut [u8; SBUS_MESSAGE_LENGTH]>,
    }

    #[init(local = [x4r_dma_buf1: [u8; SBUS_MESSAGE_LENGTH] = [0; SBUS_MESSAGE_LENGTH],
                    x4r_dma_buf2: [u8; SBUS_MESSAGE_LENGTH] = [0; SBUS_MESSAGE_LENGTH]])]
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
        Mono::start(cx.core.SYST, rcc.clocks.sysclk().to_Hz());

        defmt::println!("BOOT");

        // GPIO pins
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        // CLI SETUP
        // ---------------------------------------------------------------------

        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()),
            &rcc.clocks
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
        // -----------------------------------------

        let x4r_dma_config = get_x4r_dma_config();

        let mut telem_usart = Serial::<USART1, u8>::new(
            dp.USART1,
            (gpioa.pa9.into_alternate(), gpioa.pa10.into_alternate()),
            X4R_SBUS_CONFIG,
            &mut rcc
        ).unwrap();

        telem_usart.listen(Event::Idle);

        let (_tx, rx) = telem_usart.split();

        let channels = StreamsTuple::new(dp.DMA2, &mut rcc);

        let mut x4r_dma = X4rDmaTransfer::init_peripheral_to_memory(
            channels.2,
            rx,
            cx.local.x4r_dma_buf1,
            None,
            x4r_dma_config
        );

        x4r_dma.start(|_rx| {}); // Start tracking DMA

        let x4r = X4r::new();
        let x4r_data = x4r.get_data();

        // ---------------------------------------------

        (Shared {
           x4r_dma: Some(x4r_dma), x4r_data
        }, Local {
            led, cli, usb_dev, x4r, x4r_dma_buf: Some(cx.local.x4r_dma_buf2)
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

    #[task(binds=DMA2_STREAM2, local=[x4r, x4r_dma_buf, led], shared=[x4r_dma, x4r_data], priority = 2)]
    fn rx_telemetry_message(cx: rx_telemetry_message::Context) {

        let local = cx.local;
        let mut shared = cx.shared;

        local.led.toggle();

        // Clear the DMA flags and grab the dma buffer
        let buf = shared.x4r_dma.lock(|dma| {

            // Move ownership of the xfer into another object
            let mut xfer = dma.take().unwrap();

            xfer.clear_flags(DmaFlag::TransferComplete);

            // Put the current dma buffer and swap it with the old one
            let (buf, _) = xfer.next_transfer(local.x4r_dma_buf.
                take().unwrap()).unwrap();

            // Return ownership of the dma
            *dma = Some(xfer);

            buf
        });

        // defmt::println!("DMA BUF: {:02x}", buf);

        *local.x4r_dma_buf = Some(buf);

        // Parse out the packet
         let _res =local.x4r.parse(local.x4r_dma_buf.as_ref().unwrap());

        // Update the shared data
        shared.x4r_data.lock(|data| {  *data = local.x4r.get_data(); })

    }

    #[task(binds = USART1, shared = [x4r_dma], priority = 1)]
    fn check_dma(cx: check_dma::Context) {
        let mut dma = cx.shared.x4r_dma;

        dma.lock(|dma| {
            let Some(xfer) = dma.as_mut() else {
                // A reset is already in flight — nothing to check right now.
                return;
            };

            xfer.clear_idle_interrupt();
            let remaining = xfer.number_of_transfers();

            if remaining != 0 && remaining != SBUS_MESSAGE_LENGTH as u16 {
                perform_dma_reset::spawn().ok();
            }
        });
    }

    #[task(shared = [x4r_dma], priority = 1)]
    async fn perform_dma_reset(mut cx: perform_dma_reset::Context) {
        let released = cx.shared.x4r_dma.lock(|dma| {
            dma.take().map(|xfer| xfer.release())
        });

        let Some((stream, rx, buf, _)) = released else {
            // Already taken by another invocation — nothing to reset.
            defmt::println!("perform_dma_reset: skipped, already in progress");
            return;
        };

        // Message frame size, skip the rest of this message to pick up the next freshly
        Mono::delay(3.millis()).await;

        defmt::println!("Resetting the DMA");

        let config = get_x4r_dma_config();

        let mut new_xfer = X4rDmaTransfer::init_peripheral_to_memory(
            stream, rx, buf, None, config,
        );
        new_xfer.start(|_rx| {});

        cx.shared.x4r_dma.lock(|dma| {
            *dma = Some(new_xfer);
        });
    }
}
