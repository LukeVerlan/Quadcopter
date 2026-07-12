#![no_std]
#![no_main]

pub mod cli;
pub mod util;
pub mod pwm;

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


#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1])]
mod app {
    use super::*;
    use super::cli::cli::{QuadCli, Writer};
    use stm32f4xx_hal::prelude::*;
    use stm32f4xx_hal::gpio::{PC13, Output, PushPull};

    // PWM
    use super::pwm::pwm::{EscPwm, Esc};
    use stm32f4xx_hal::timer::{PwmManager, PwmChannel};

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

    type Pwm =
        EscPwm<PwmChannel<TIM3,0>, PwmChannel<TIM3,1>, PwmChannel<TIM3,2>, PwmChannel<TIM3,3>>;

    type PwmMan =
        PwmManager<TIM3, 1000000>;

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

        // PWM
        pwm: Pwm,
        pwm_man: PwmMan,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Clocks
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
        Mono::start(cx.core.SYST, rcc.clocks.sysclk().to_Hz());

        // Gpio pins
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);

        // Led
        let led = gpioc.pc13.into_push_pull_output();

        defmt::println!("BOOT");

        // PWM SETUP
        // ------------------------------------------

        // 4. Initialize PWM in Hz
        let (mut pwm_man, (p1,p2,p3,p4)) = dp.TIM3.pwm_us(20000.micros(), &mut rcc);

        let pin1 = gpiob.pb1.into_alternate::<2>(); // PB1 connects to TIM3_CH4
        let pin2 = gpiob.pb4.into_alternate::<2>(); // PB4 connects to TIM3_CH1
        let pin3 = gpiob.pb5.into_alternate::<2>(); // PB5 connects to TIM3_CH2
        let pin4 = gpiob.pb0.into_alternate::<2>(); // PB0 connects to TIM3_CH3

        let mut p1 = p1.with(pin2);
        let mut p2 = p2.with(pin3);
        let mut p3 = p3.with(pin4);
        let mut p4 = p4.with(pin1);

        p1.enable();
        p2.enable();
        p3.enable();
        p4.enable();

        let pwm = EscPwm::new(p1, p2, p3, p4, 20000);

        // --------------------------------------------------------

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
        
        pwm_test::spawn().unwrap();
        blink::spawn().unwrap();

        (Shared {
        }, Local {
            led, quad_cli, usb_dev, pwm, pwm_man
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
            Mono::delay(1000.millis()).await;
        }
    }

    #[task(local=[pwm, pwm_man], priority = 3)]
    async fn pwm_test(_cx: pwm_test::Context) {
        let pwm = _cx.local.pwm;
        let mut pwm_man = _cx.local.pwm_man;
        let mut counter = 0;
        let mut up = true;
            loop {

                let mut _res = pwm.set_esc(Esc::Esc1, counter as f32 / 100.0);
                let mut _res = pwm.set_esc(Esc::Esc2, counter as f32 / 100.0);
                let mut _res = pwm.set_esc(Esc::Esc3, counter as f32 / 100.0);
                let mut _res = pwm.set_esc(Esc::Esc4, counter as f32 / 100.0);
                if up {
                    if counter > 24 {
                        up = false;
                    }
                    else {
                        counter += 1;
                    }
                }
                else {
                    if counter < 1 {
                        up = true;
                    }
                    else {
                        counter -= 1;
                    }
                }
                Mono::delay(200.millis()).await;
        }
    }
}
