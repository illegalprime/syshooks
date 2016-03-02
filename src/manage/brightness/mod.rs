pub mod fs;
pub mod dbus;

use std::error::Error;

pub trait Brightness {
    type E: Error;

    fn max(&self) -> Result<f64, Self::E>;
    fn current(&self) -> Result<f64, Self::E>;
    fn set(&self, value: f64) -> Result<(), Self::E>;
}
