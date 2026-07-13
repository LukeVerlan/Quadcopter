// ESC mapping:
// ESC 1 - Pin PB1 with peripheral TIM3, CH4
// ESC 2 - Pin PB10 with peripheral TIM2, CH3
// ESC 3 - Pin PA0 with peripheral TIM2, CH1
// ESC 4 - Pin PA1 with peripheral TIM2, CH2

// Peripheral base addresses
pub enum PeriphBase {
    Rcc   = 0x4002_3800,
    GpioA = 0x4002_0000,
    GpioB = 0x4002_0400,
    Tim2  = 0x4000_0000,
    Tim3  = 0x4000_0400,
}

// Timer register offsets
pub enum TimReg {
    Cr1   = 0x00,
    Egr   = 0x14,
    Ccmr1 = 0x18,
    Ccmr2 = 0x1C,
    Ccer  = 0x20,
    Psc   = 0x28,
    Arr   = 0x2C,
    Ccr1  = 0x34,
    Ccr2  = 0x38,
    Ccr3  = 0x3C,
    Ccr4  = 0x40,
}
