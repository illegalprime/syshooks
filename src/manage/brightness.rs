use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::io::Error as IoError;
use std::num::ParseFloatError;

const BRIGHTNESS_DIR: &'static str = "/sys/class/backlight/intel_backlight/";

pub fn get_max_brightness() -> Result<f64, Error> {
    let mut buffer = String::new();
    let max_path = Path::new(BRIGHTNESS_DIR).join("max_brightness");

    let mut max_brightness = try!(File::open(max_path));
    max_brightness.read_to_string(&mut buffer);
    let max_brightness: f64 = try!(buffer.trim().parse());

    Ok(max_brightness)
}

pub fn get_brightness() -> Result<f64, Error> {
    let mut buffer = String::new();
    let max_path = Path::new(BRIGHTNESS_DIR).join("brightness");

    let mut max_brightness = try!(File::open(max_path));
    max_brightness.read_to_string(&mut buffer);
    let max_brightness: f64 = try!(buffer.trim().parse());

    Ok(max_brightness)
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

pub enum Error {
    Io(IoError),
    Parse(ParseFloatError),
}
