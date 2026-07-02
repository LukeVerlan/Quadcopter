
pub enum Esc {
    Esc1,
    Esc2,
    Esc3,
    Esc4,
}

// Minimum pulse = 1 millisecond
pub const PULSE_MIN_US: u16 = 1_000;
// Maximum pulse = 2 milliseconds
pub const PULSE_MAX_US: u16 = 2_000;

#[derive(Debug)]
pub enum PwmError {
    InvalidEsc,
}

// Pulse width data for all four ESCs
#[derive(Copy, Clone)]
pub struct PwmData {
    pub esc1_us: u16,
    pub esc2_us: u16,
    pub esc3_us: u16,
    pub esc4_us: u16,
}

pub struct EscPwm {
    pub data: PwmData,
}

impl EscPwm {

    // Write a pulse width in µs to an ESC (clamped to [1000, 2000])
    pub fn write_pulse(&mut self, esc: Esc, pulse_us: u16) -> Result<(), PwmError> {
        let pulse = pulse_us.clamp(PULSE_MIN_US, PULSE_MAX_US) as u32;

        match esc {
            Esc::Esc1 => {
                self.data.esc1_us = us as u16;
                Self::write(PeriphBase::Tim3 as u32, TimReg::Ccr4 as u32, pulse);
            }
            Esc::Esc2 => {
                self.data.esc2_us = us as u16;
                Self::write(PeriphBase::Tim2 as u32, TimReg::Ccr3 as u32, pulse);
            }
            Esc::Esc3 => {
                self.data.esc3_us = us as u16;
                Self::write(PeriphBase::Tim2 as u32, TimReg::Ccr1 as u32, pulse);
            }
            Esc::Esc4 => {
                self.data.esc4_us = us as u16;
                Self::write(PeriphBase::Tim2 as u32, TimReg::Ccr2 as u32, pulse);
            }
        }

        Ok(())
    }

    // Read back the current pulse width for an ESC (in us).
    pub unsafe fn read_pulse(&self, esc: Esc) -> Result<u16, PwmError> {
        let us = match esc {
            Esc::Esc1 => Self::read(PeriphBase::Tim3 as u32, TimReg::Ccr4 as u32),
            Esc::Esc2 => Self::read(PeriphBase::Tim2 as u32, TimReg::Ccr3 as u32),
            Esc::Esc3 => Self::read(PeriphBase::Tim2 as u32, TimReg::Ccr1 as u32),
            Esc::Esc4 => Self::read(PeriphBase::Tim2 as u32, TimReg::Ccr2 as u32),
        };
        Ok(us as u16)
    }

    // Set throttle as [0.0, 1.0] instead of us (normalize to pulse min/max range)
    pub unsafe fn write_throttle(&mut self, esc: Esc, throttle: f32) -> Result<(), PwmError> {
        let t = throttle.clamp(0.0, 1.0);
        let pulse = PULSE_MIN_US + (t * (PULSE_MAX_US - PULSE_MIN_US) as f32) as u16;
        self.write_pulse(esc, pulse)
    }

    #[inline(always)]
    unsafe fn read(base: u32, offset: u32) -> u32 {
        core::ptr::read_volatile((base + offset) as *const u32)
    }

    #[inline(always)]
    unsafe fn write(base: u32, offset: u32, value: u32) {
        core::ptr::write_volatile((base + offset) as *mut u32, value);
    }

}



// Struct EscPwm {
// } 

// impl EscPwm {
  // fn new(NOT SELF) -> Self { ... }
  // fn set_esc(&mut self, enum: ESC, percentage) -> Result<>
  // fn read_esc(&mut self, enum: ESC) -> Percentage: f32 


