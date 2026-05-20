#![no_std]
#![no_main]
extern crate alloc;

mod icm42688p;

// Need for panic silence
use panic_halt as _;

use rtic::app;
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1_000_00);

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [EXTI0])]
mod app {
    use super::*;
    use stm32f4xx_hal::prelude::*;
    use stm32f4xx_hal::gpio::{PC13, Output, PushPull};
    // use stm32f4xx_hal::pac;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: PC13<Output<PushPull>>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let dp = cx.device;

        // Clocks
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(96.MHz()).freeze();

        // Clock startup
        Mono::start(cx.core.SYST, clocks.sysclk().to_Hz());

        let gpioc = dp.GPIOC.split();
        let led = gpioc.pc13.into_push_pull_output();

        blink::spawn().unwrap();

        (Shared {

        }, Local {
            led
        })
    }

    #[task(local=[led])]
    async fn blink(_cx: blink::Context) {
        let led = _cx.local.led;
        loop {
            led.toggle();
            Mono::delay(1000.millis()).await; // Wait 500 milliseconds
        }
    }
}
