use std::env::args;
use std::mem::drop;

mod notify;
mod manage;

use manage::brightness::Brightness;
use manage::volume::Mixer;

const BRIGHTNESS_DIR: &'static str = "/sys/class/backlight/intel_backlight/";

fn main() {
    let master = Mixer::new("default\0", "Master\0").unwrap();
    println!("Made a new Master mixer!");
    let (min, max) = master.volume_range();
    println!("Master range: {:?}", (min, max));
    println!("Setting volume to half");
    master.set_volume(0.5);
    println!("Is mixer mono? {}", master.is_mono());
    println!("Is mixer muted? {}", master.is_muted());
    println!("toggling mute...");
    master.toggle_mute().ok();
    println!("Is mixer muted? {}", master.is_muted());
    println!("Dropping mixer..");
    drop(master);
}

fn set_brightness() {
    let mut args = args().skip(1);

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

    let percent: f64 = if let Some(arg) = args.next() {
        arg.parse().unwrap()
    } else {
        unimplemented!();
    };

    let bright_control = Brightness::new(BRIGHTNESS_DIR);

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
