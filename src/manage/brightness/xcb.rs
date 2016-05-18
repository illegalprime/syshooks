extern crate xcb as xcb_ffi;

use std::io::{
    Write,
    stderr,
    Error,
    ErrorKind,
};

use self::xcb_ffi::base::Connection;
use self::xcb_ffi::xproto;
use self::xcb_ffi::xproto::Atom;
use self::xcb_ffi::ffi::xproto as xproto_ffi;
use self::xcb_ffi::randr;
use self::xcb_ffi::randr::Output;
use self::xcb_ffi::ffi::randr as randr_ffi;

#[derive(Debug)]
pub struct Display {
    output: Output,
    min: i32,
    max: i32,
    current: i32,
}

pub struct XcbBrightness {
    connection: Connection,
    atom: Atom,
    displays: Vec<Display>,
}

impl XcbBrightness {
    pub fn connect() -> Self {
        create_session()
    }
}

impl super::Brightness for XcbBrightness {
    type E = Error;

    fn max(&self) -> Result<f64, Error> {
        Ok(100f64)
    }

    fn min(&self) -> Result<f64, Error> {
        Ok(0f64)
    }

    fn current(&self) -> Result<f64, Error> {
        if let Some(display) = self.displays.first() {
            match backlight_get(&self.connection, display.output, self.atom) {
                Some(current) => {
                    let min = display.min as f64;
                    let max = display.max as f64;
                    Ok(100f64 * (current as f64 - min) / (max - min))
                },
                None => Err(sentinel_e()),
            }
        } else {
            Err(sentinel_e())
        }
    }

    fn set(&self, value: f64) -> Result<(), Error> {
        if let Some(display) = self.displays.first() {
            let min = display.min as f64;
            let max = display.max as f64;
            let new = value * (max - min) / 100.0 + min;
            backlight_set(&self.connection, display.output, self.atom, new as u32);
            Ok(())
        } else {
            Err(sentinel_e())
        }
    }
}

fn create_session() -> XcbBrightness {
    let mut displays = Vec::new();
    let (connection, _) = Connection::connect(None)
        .expect("Could not connect to the x server!");

    let reply = randr::query_version(&connection, 1, 2).get_reply()
        .expect("RANDR query version returned an error");

    if reply.major_version() != 1 || reply.minor_version() < 2 {
        panic!("RandR version {}.{} is too old!",
               reply.major_version(), reply.minor_version());
    }

    let backlight_new = xproto::intern_atom(&connection, true, "Backlight")
        .get_reply()
        .expect("intern backlight atom returned an error");

    let atom = if backlight_new.atom() == xproto_ffi::XCB_ATOM_NONE {

        let backlight_legacy = xproto::intern_atom(&connection, true, "BACKLIGHT")
            .get_reply()
            .expect("intern legacy backlight atom returned an error");

        if backlight_legacy.atom() == xproto_ffi::XCB_ATOM_NONE {
            panic!("No outputs have backlight property");
        }

        backlight_legacy.atom()
    } else {
        backlight_new.atom()
    };

    let mut iter = unsafe {
        xproto_ffi::xcb_setup_roots_iterator(connection.get_setup().ptr)
    };

    while iter.rem > 0 {

        let root = unsafe {
            (*iter.data).root
        };

        let resources_cookie = randr::get_screen_resources_current(&connection, root);
        let resources = match resources_cookie.get_reply() {
            Ok(reply) => reply,
            Err(e) => {
                writeln!(&mut stderr(), "RandR get screen resources error {:?}", e).ok();
                continue
            },
        };

        let outputs = unsafe {
            randr_ffi::xcb_randr_get_screen_resources_current_outputs(resources.ptr)
        };

        for o in 0..resources.num_outputs() {
            let output = unsafe {
                *outputs.offset(o as isize)
            };

            let current = match backlight_get(&connection, output, atom) {
                Some(c) => c,
                None => continue,
            };

            let prop = match randr::query_output_property(&connection, output, atom).get_reply() {
                Ok(reply) => reply,
                Err(e) => {
                    writeln!(&mut stderr(), "backlight info query error: {:?}", e).ok();
                    continue
                },
            };

            if !prop.range() || unsafe {
                randr_ffi::xcb_randr_query_output_property_valid_values_length(prop.ptr)
            } != 2 {
               writeln!(&mut stderr(), "got invalid range for backlight property").ok();
               continue
            }

            let (min, max) = unsafe {
                let range = randr_ffi::xcb_randr_query_output_property_valid_values(prop.ptr);
                (*range, *range.offset(1))
            };

            displays.push(Display {
                output: output,
                min: min,
                max: max,
                current: current,
            });
        }

        unsafe {
            xproto_ffi::xcb_screen_next(&mut iter as *mut xproto_ffi::xcb_screen_iterator_t);
        }
    }

    XcbBrightness {
        connection: connection,
        atom: atom,
        displays: displays,
    }
}

fn backlight_get(connection: &Connection, output: Output, atom: Atom) -> Option<i32> {
    let property = match randr::get_output_property(
            connection, output, atom, xproto_ffi::XCB_ATOM_NONE, 0, 4, false, false
    ).get_reply() {
        Ok(p) => p,
        Err(_) => return None,
    };
    if property.num_items() != 1 || property.format() != 32 {
        None
    } else {
        Some(unsafe {
            *(randr_ffi::xcb_randr_get_output_property_data(property.ptr) as *const i32)
        })
    }
}

fn backlight_set(connection: &Connection, output: Output, atom: Atom, value: u32) {
    randr::change_output_property(
        connection,
        output,
        atom,
        xproto_ffi::XCB_ATOM_INTEGER,
        32,
        xproto_ffi::XCB_PROP_MODE_REPLACE as u8,
        &[value as i32],
    );
    connection.flush();
}

fn sentinel_e() -> Error {
    Error::new(ErrorKind::Other, "XcbBrightness does not yet support good error handling.")
}


