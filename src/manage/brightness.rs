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

pub struct Brightness {
    max_path: PathBuf,
    curr_path: PathBuf,
}

impl Brightness {
    pub fn new(backlight_dir: &str) -> Self {
        Brightness {
            max_path: Path::new(backlight_dir).join("max_brightness").to_owned(),
            curr_path: Path::new(backlight_dir).join("brightness").to_owned(),
        }
    }

    pub fn max(&self) -> Result<f64, Error> {
        let mut buffer = String::new();
        let mut max_brightness = try!(File::open(&self.max_path));

        max_brightness.read_to_string(&mut buffer);
        let max_brightness: f64 = try!(buffer.trim().parse());

        Ok(max_brightness)
    }

    pub fn current(&self) -> Result<f64, Error> {
        let mut buffer = String::new();
        let mut max_brightness = try!(File::open(&self.curr_path));

        max_brightness.read_to_string(&mut buffer);
        let max_brightness: f64 = try!(buffer.trim().parse());

        Ok(max_brightness)
    }

    pub fn set(&self, value: f64) -> Result<(), Error> {
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
