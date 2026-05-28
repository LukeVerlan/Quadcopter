

use ufmt::{uDisplay, uwrite, Formatter};

/** Enum struct that populates the sigma */
pub struct DisplayFloat<T>(pub T);

impl<T> uDisplay for DisplayFloat<T>
where
    T: ryu::Float + PartialEq,
{
    fn fmt<W: ufmt::uWrite + ?Sized>(&self, f: &mut Formatter<W>) -> Result<(), W::Error> {
        if self.0 != self.0 { 
            uwrite!(f, "NaN")
        } else {
            let mut buf = ryu::Buffer::new();
            let s = buf.format(self.0);
            f.write_str(s)
        }
    }
}

/** Parses a UTF8 byte array as 32-bit float */
pub fn parse_f32(bytes: &[u8]) -> f32 {
    core::str::from_utf8(bytes).unwrap().parse::<f32>().unwrap_or(f32::NAN)
}

/** Parses a UTF8 byte array as 64-bit float */
pub fn parse_f64(bytes: &[u8]) -> f64 {
    core::str::from_utf8(bytes).unwrap().parse::<f64>().unwrap_or(f64::NAN)
}

/** Parses a UTF8 byte array as an u8 */
pub fn parse_u8(val: &[u8]) -> u8 {
    core::str::from_utf8(val).unwrap_or("0").parse::<u8>().unwrap_or(0)
}

