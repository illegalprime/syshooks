extern crate ears;

use std::thread::sleep;
use std::time::Duration;
use self::ears::{
    Sound,
    AudioController,
};

const VOLUME_CHANGE: &'static str = "/usr/share/sounds/freedesktop/stereo/audio-volume-change.oga";

pub fn volume_change() -> Result<(), ()> {
    let mut snd = try!(Sound::new(VOLUME_CHANGE).ok_or(()));

    snd.play();

    while snd.is_playing() {
        sleep(Duration::from_millis(500));
    }
    Ok(())
}
