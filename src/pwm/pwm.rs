// EscChannel handles duty/pulse width conversions, writes using pulse width
// EscPwm groups all four EscChannels together, using Esc field and throttle (0.0 - 1.0) to write pulse widths
// (use this in main)
use stm32f4xx_hal::hal_02::PwmPin;

// minimum/maximum pulse width in microseconds
// (check actual values for our ESCs, these are placeholders)
pub const PULSE_MIN_US: u32 = 1_000;
pub const PULSE_MAX_US: u32 = 2_000;

// identifies one of the four ESC outputs
pub enum Esc {
    Esc1,
    Esc2,
    Esc3,
    Esc4,
}

pub enum PwmError {
    OutOfRange, // requested throttle outside of 0.0 - 1.0 range
}

// one ESC's PWM channel: a HAL pin and the microseconds to duty conversion
pub struct EscChannel<P> {
    pin: P,
    duty_per_us: f32,
}

impl<P> EscChannel<P> {
    // period_us needs to match whatever period the timer is configured with
    pub fn new(pin: P, period_us: u32) -> Self {
        let max_duty = pin.get_max_duty();
        let duty_per_us = max_duty as f32 / period_us as f32;
        Self {
            pin,
            duty_per_us
        }
    }

    pub fn write_pulse_us(&mut self, us: u32) {
        let us = us.clamp(PULSE_MIN_US, PULSE_MAX_US);
        let duty = (self.duty_per_us * us as f32) as u16;
        self.pin.set_duty(duty);
    }

    pub fn read_pulse_us(&self) -> u32 {
        (self.pin.get_duty() as f32 / self.duty_per_us) as u32
    }

    pub fn enable(&mut self) {
        self.pin.enable();
    }

    pub fn disable(&mut self) {
        self.pin.set_duty(0);
        self.pin.disable();
    }
}

// all four ESC channels together
pub struct EscPwm<P1, P2, P3, P4> {
    esc1: EscChannel<P1>,
    esc2: EscChannel<P2>,
    esc3: EscChannel<P3>,
    esc4: EscChannel<P4>,
}

impl<P1, P2, P3, P4> EscPwm<P1, P2, P3, P4> {
    // same period_us as before is shared across all four
    pub fn new(p1: P1, p2: P2, p3: P3, p4: P4, period_us: u32) -> Self {
        Self {
            esc1: EscChannel::new(p1, period_us),
            esc2: EscChannel::new(p2, period_us),
            esc3: EscChannel::new(p3, period_us),
            esc4: EscChannel::new(p4, period_us),
        }
    }

    // write a pulse width in microseconds to one ESC
    pub fn write_pulse(&mut self, esc: Esc, pulse_us: u32) {
        match esc {
            Esc::Esc1 => self.esc1.write_pulse_us(pulse_us),
            Esc::Esc2 => self.esc2.write_pulse_us(pulse_us),
            Esc::Esc3 => self.esc3.write_pulse_us(pulse_us),
            Esc::Esc4 => self.esc4.write_pulse_us(pulse_us),
        }
    }

    // read back an ESC's current pulse width in microseconds
    pub fn read_pulse(&self, esc: Esc) -> u32 {
        match esc {
            Esc::Esc1 => self.esc1.read_pulse_us(),
            Esc::Esc2 => self.esc2.read_pulse_us(),
            Esc::Esc3 => self.esc3.read_pulse_us(),
            Esc::Esc4 => self.esc4.read_pulse_us(),
        }
    }

    // set throttle as a percentage from 0.0 to 1.0, normalized to pulse width range
    pub fn set_esc(&mut self, esc: Esc, throttle: f32) -> Result<(), PwmError> {
        if !(0.0..=1.0).contains(&throttle) {
            return Err(PwmError::OutOfRange); // reject throttle percentages outside valid range
        }
        let range = (PULSE_MAX_US - PULSE_MIN_US) as f32;
        let pulse = PULSE_MIN_US + (throttle * range) as u32;
        self.write_pulse(esc, pulse);
        Ok(())
    }

    // read throttle as a percentage from 0.0 to 1.0, normalized to pulse width range
    pub fn read_esc(&self, esc: Esc) -> f32 {
        let us = self.read_pulse(esc);
        let span = (PULSE_MAX_US - PULSE_MIN_US) as f32;
        (us.saturating_sub(PULSE_MIN_US) as f32 / span).clamp(0.0, 1.0)
    }

    // enable all four PWM outputs
    pub fn enable_all(&mut self) {
        self.esc1.enable();
        self.esc2.enable();
        self.esc3.enable();
        self.esc4.enable();
    }

    // disable all four PWM outputs
    pub fn disable_all(&mut self) {
        self.esc1.disable();
        self.esc2.disable();
        self.esc3.disable();
        self.esc4.disable();
    }
}
