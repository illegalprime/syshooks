pub mod fs;
pub mod dbus;
pub mod xcb;

use std::error::Error;

pub trait Brightness {
    type E: Error;

    fn max(&self) -> Result<f64, Self::E>;
    fn min(&self) -> Result<f64, Self::E>;
    fn current(&self) -> Result<f64, Self::E>;
    fn set(&self, value: f64) -> Result<(), Self::E>;

    fn change_n_clip(&self, delta: f64) -> Result<(), Self::E> {
        let (min, max) = (try!(self.min()), try!(self.max()));
        let current = try!(self.current());

        let next = if current + delta > max {
            max
        } else if current + delta < min {
            min
        } else {
            current + delta
        };

        self.set(next)
    }
}
