use std::env::args;

mod notify;
mod manage;

use manage::brightness::Brightness;
use manage::brightness::fs::FsBrightness;
use manage::volume::Mixer;

use notify::volume::Volume;
use notify::volume;

const BRIGHTNESS_DIR: &'static str = "/sys/class/backlight/intel_backlight/";

fn main() {
    match args().nth(1).as_ref().map(|s| s as &str) {
        Some("volume") => set_volume(),
        Some("brightness") => set_brightness(),
        _ => unimplemented!(),
    };
}

fn set_volume() {
    let mut args = args().skip(2);

    let (mult, set, toggle_mute) = if let Some(arg) = args.next() {
        if arg == "up" {
            (1.0, false, false)
        } else if arg == "down" {
            (-1.0, false, false)
        } else if arg == "set" {
            (1.0, true, false)
        } else if arg == "toggle-mute" {
            (1.0, false, true)
        } else {
            unimplemented!();
        }
    } else {
        unimplemented!();
    };

    let master = Mixer::new("default", "Master").unwrap();

    if toggle_mute {
        master.toggle_mute();
    } else {
        let percent: f32 = match args.next().and_then(|p| p.parse().ok()) {
            Some(p) => p,
            None => return, // TODO
        };
        if set {
            master.set_volume(percent / 100.0);
        } else {
            master.change_volume_clip(percent * mult / 100.0);
        }
    }

    let vol_status = if master.is_muted() {
        Volume::Muted
    } else {
        Volume::Percent((master.volume() * 100.0) as u32)
    }; 
    
    volume::show_volume(vol_status);
}

fn set_brightness() {
    let mut args = args().skip(2);

    let (mult, set) = if let Some(arg) = args.next() {
        if arg == "up" {
            (1.0, false)
        } else if arg == "down" {
            (-1.0, false)
        } else if arg == "set" {
            (1.0, true)
        } else {
            unimplemented!();
        }
    } else {
        unimplemented!();
    };

    let percent: f64 = match args.next().and_then(|p| p.parse().ok()) {
        Some(p) => p,
        None => return, // TODO
    };

    let bright_control = FsBrightness::new(BRIGHTNESS_DIR);

    let max = bright_control.max().ok().unwrap();
    let mut next = mult * percent * max / 100.0 + if set {
        0.0
    } else {
        bright_control.current().ok().unwrap()
    };

    if next > max {
        next = max;
    } else if next < 0.0 {
        next = 0.0;
    }

    bright_control.set(next).ok().unwrap();
    notify::brightness::show_brightness((next * 100.0 / max) as u32).ok().unwrap();
}
