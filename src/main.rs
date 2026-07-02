#![no_std]
#![no_main]

pub mod cli;
pub mod util;

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

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use super::*;
    use super::cli::cli::{QuadCli, Writer};
    use stm32f4xx_hal::prelude::*;
    use stm32f4xx_hal::gpio::{PC13, Output, PushPull};

    // CLI
    use stm32f4xx_hal::otg_fs::{USB, UsbBus};
    use usbd_serial::SerialPort;
    use embedded_cli::cli::{CliBuilder};
    use embedded_hal::pwm::SetDutyCycle;
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use usb_device::bus::UsbBusAllocator;
    use stm32f4xx_hal::serial::{Rx, Tx};
    use stm32f4xx_hal::timer::{FTimer, Timer};
    use stm32f4xx_hal::pac::*;
    use stm32f4xx_hal::rcc::Config;


    static mut USB_BUS: Option<UsbBusAllocator<UsbBus<USB>>> = None;
    static mut SER: Option<SerialPort<'static, UsbBus<USB>>> = None;
    static mut EP_MEMORY: [u32; 1024] = [0; 1024]; // 4096 bytes of memory for the serial port

    #[shared]
    struct Shared {
    }

    #[local]
    struct Local {

        // LED
        led: PC13<Output<PushPull>>,

        // CLI
        quad_cli: QuadCli,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Clocks
        let dp = Peripherals::take().unwrap();

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
        Mono::start(cx.core.SYST, rcc.clocks.sysclk().to_Hz());

        // Gpio pins
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        let pin = gpioa.pa8.into_alternate::<1>(); // PA8 connects to TIM1_CH1

        // 4. Initialize PWM in Hz
        let (mut pwm_man, (p1,p2,p3,p4)) = dp.TIM1.pwm_us(250.micros(), &mut rcc);

        let mut p1 = p1.with(pin);

        let percent_throttle = 0.75;
        p1.set_duty(((p1.get_max_duty() as f32) * percent_throttle) as u16); // 3.3v * 3/4

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
            let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x16c0, 0x27dd))
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
        let quad_cli = QuadCli::new(cli);

        //  -------------------------------------------

        (Shared {
        }, Local {
            led, quad_cli, usb_dev,
        })
    }

    #[idle(local = [quad_cli, usb_dev])]
    fn idle(mut _cx: idle::Context) -> ! {
        let cli = _cx.local.quad_cli;
        let usb_dev = _cx.local.usb_dev;
        let ser = unsafe { SER.as_mut().unwrap() };

        ser.write(b"Quadcopter online\n").unwrap();

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
            Mono::delay(1000.millis()).await; // Wait 500 milliseconds
        }
    }
}
