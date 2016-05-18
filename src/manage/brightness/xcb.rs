extern crate xcb;

use std::io::{
    Write,
    stderr,
};

use xcb::base::Connection;
use xcb::xproto;
use xcb::xproto::Atom;
use xcb::ffi::xproto as xproto_ffi;
use xcb::randr;
use xcb::randr::Output;
use xcb::ffi::randr as randr_ffi;

pub enum Op {
    Get,
    Set(u32),
    Inc(u32),
    Dec(u32),
}

pub fn manage(op: Op) {

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

            match op {
                Op::Get => {
                    println!("{}", (current - min) * 100 / (max - min))
                },
                _ => unimplemented!(),
            };
        }

        unsafe {
            xproto_ffi::xcb_screen_next(&mut iter as *mut xproto_ffi::xcb_screen_iterator_t);
        }
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

fn backlight_set(connection: &Connection, output: Output, atom: Atom, is_legacy: bool, value: u64) {
    unimplemented!()
}


