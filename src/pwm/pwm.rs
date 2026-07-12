// EscChannel handles duty/pulse width conversions, writes using pulse width
// EscPwm groups all four EscChannels together, using Esc field and throttle (0.0 - 1.0) to write pulse widths
// (use this in main)
use embedded_hal::pwm::SetDutyCycle;

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
    percent: f32,
}

impl<P> EscChannel<P>
where P: SetDutyCycle {
    // period_us needs to match whatever period the timer is configured with
    fn new(pin: P, period_us: u32) -> Self {
        let max_duty = pin.max_duty_cycle();
        let duty_per_us = max_duty as f32 / period_us as f32;
        Self {
            pin,
            duty_per_us,
            percent: f32::NAN,
        }
    }

    fn write_pulse_us(&mut self, us: u32) {
        let us = us.clamp(PULSE_MIN_US, PULSE_MAX_US);
        let duty = (self.duty_per_us * us as f32) as u16;
        let _res = self.pin.set_duty_cycle(duty);
    }
}

// all four ESC channels together
pub struct EscPwm<P1, P2, P3, P4> {
    esc1: EscChannel<P1>,
    esc2: EscChannel<P2>,
    esc3: EscChannel<P3>,
    esc4: EscChannel<P4>,
}

impl<P1, P2, P3, P4> EscPwm<P1, P2, P3, P4>
where
    P1: SetDutyCycle,
    P2: SetDutyCycle,
    P3: SetDutyCycle,
    P4: SetDutyCycle
{
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
    fn write_pulse(&mut self, esc: &Esc, pulse_us: u32) {
        match esc {
            Esc::Esc1 => self.esc1.write_pulse_us(pulse_us),
            Esc::Esc2 => self.esc2.write_pulse_us(pulse_us),
            Esc::Esc3 => self.esc3.write_pulse_us(pulse_us),
            Esc::Esc4 => self.esc4.write_pulse_us(pulse_us),
        }
    }

    // set throttle as a percentage from 0.0 to 1.0, normalized to pulse width range
    pub fn set_esc(&mut self, esc: Esc, throttle: f32) -> Result<(), PwmError> {
        if !(0.0..=1.0).contains(&throttle) {
            return Err(PwmError::OutOfRange); // reject throttle percentages outside valid range
        }
        let range = (PULSE_MAX_US - PULSE_MIN_US) as f32;
        let pulse = PULSE_MIN_US + (throttle * range) as u32;
        self.write_pulse(&esc, pulse);
        match esc { 
            Esc::Esc1 => {self.esc1.percent = throttle}
            Esc::Esc2 => {self.esc2.percent = throttle}
            Esc::Esc3 => {self.esc3.percent = throttle}
            Esc::Esc4 => {self.esc4.percent = throttle}
        }
        Ok(())
    }

    // read throttle as a percentage from 0.0 to 1.0, normalized to pulse width range
    pub fn read_esc(&self, esc: Esc) -> f32 {
        match esc {
            Esc::Esc1 => self.esc1.percent,
            Esc::Esc2 => self.esc2.percent,
            Esc::Esc3 => self.esc3.percent,
            Esc::Esc4 => self.esc4.percent,
        }
    }
}
