use std::env::args;

mod notify;
mod manage;
mod audio;

use manage::brightness::Brightness;
use manage::brightness::dbus::DbusBrightness;
use manage::volume::Mixer;

use notify::volume::Volume;
use notify::volume;

use audio::notifications;

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
        println!("{:?}", master.toggle_mute());
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

    if let Err(e) = volume::show_volume(vol_status) {
        println!("Error showing volume notification: {}", e);
    }
    if let Err(_) = notifications::volume_change() {
        println!("Could not find volume change audio clip");
    }
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

    let percent: f64 = args.next().and_then(|p| p.parse().ok()).unwrap();

    let bright_control = DbusBrightness::new().unwrap();

    if set {
        bright_control.set(percent).unwrap();
    } else {
        bright_control.change_n_clip(mult * percent).unwrap();
    }
    let current = bright_control.current().unwrap() as u32;
    notify::brightness::show_brightness(current).unwrap();
}
