#![no_std]
#![no_main]

use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use stm32f4xx_hal::prelude::*;
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let rcc = cx.device.RCC.constrain();
        let _clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(96.MHz()).freeze();
        (Shared {}, Local {})
    }
}
