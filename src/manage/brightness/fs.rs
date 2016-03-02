use std::fs::{
    File,
    OpenOptions,
};
use std::path::{
    PathBuf,
    Path,
};
use std::io::{
    Read,
    Write,
};
use std::io::Error as IoError;
use std::num::ParseFloatError;
use std::error::Error as ErrorTrait;
use std::fmt::{
    Display,
    Formatter,
};
use std::fmt::Error as FmtError;

use super::Brightness;

pub struct FsBrightness {
    max_path: PathBuf,
    curr_path: PathBuf,
}

impl FsBrightness {
    pub fn new(backlight_dir: &str) -> Self {
        FsBrightness {
            max_path: Path::new(backlight_dir).join("max_brightness").to_owned(),
            curr_path: Path::new(backlight_dir).join("brightness").to_owned(),
        }
    }
}

impl Brightness for FsBrightness {
    type E = Error;

    fn max(&self) -> Result<f64, Self::E> {
        let mut buffer = String::new();
        let mut max_brightness = try!(File::open(&self.max_path));

        max_brightness.read_to_string(&mut buffer).ok();
        let max_brightness: f64 = try!(buffer.trim().parse());

        Ok(max_brightness)
    }

    fn current(&self) -> Result<f64, Self::E> {
        let mut buffer = String::new();
        let mut max_brightness = try!(File::open(&self.curr_path));

        max_brightness.read_to_string(&mut buffer).ok();
        let max_brightness: f64 = try!(buffer.trim().parse());

        Ok(max_brightness)
    }

    fn set(&self, value: f64) -> Result<(), Self::E> {
        let max = try!(self.max());

        if value < 0.0 || value > max {
            return Err(Error::OutOfRange);
        }

        let mut control = try!(OpenOptions::new()
            .write(true)
            .open(&self.curr_path));
        try!(control.write_fmt(format_args!("{}", value as u32)));
        Ok(())
    }
}

impl From<IoError> for Error {
    #[inline]
    fn from(err: IoError) -> Self {
        Error::Io(err)
    }
}

impl From<ParseFloatError> for Error {
    #[inline]
    fn from(err: ParseFloatError) -> Self {
        Error::Parse(err)
    }
}

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Parse(ParseFloatError),
    OutOfRange,
}

impl ErrorTrait for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref io) => io.description(),
            Error::Parse(ref p) => p.description(),
            Error::OutOfRange => "Brightness value out of range",
        }
    }

    fn cause(&self) -> Option<&ErrorTrait> {
        match *self {
            Error::Io(ref io) => Some(io),
            Error::Parse(ref p) => Some(p),
            Error::OutOfRange => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        fmt.write_str(self.description())
    }
}
