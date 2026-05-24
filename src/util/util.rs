

use ufmt::{uDisplay, uwrite, Formatter};
use ufmt_float::{uFmt_f32, uFmt_f64};

pub struct DisplayF32(pub f32);
pub struct DisplayF64(pub f64);

impl uDisplay for DisplayF32 {
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut Formatter<W>) -> Result<(), W::Error> {
        if self.0.is_nan() {
            uwrite!(f, "NaN")
        } else {
            uwrite!(f, "{}", uFmt_f32::Five(self.0))
        }
    }
}

impl uDisplay for DisplayF64 {
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut Formatter<W>) -> Result<(), W::Error> {
        if self.0.is_nan() {
            uwrite!(f, "NaN")
        } else {
            uwrite!(f, "{}", uFmt_f64::Five(self.0))
        }
    }
}