extern crate dbus;

use self::dbus::{Connection, BusType, Message};
use self::dbus::Error as DbusError;
use self::dbus::arg::Array;

use super::Brightness;

pub struct DbusBrightness {
    connection: Connection,
}

impl DbusBrightness {
    pub fn new() -> Result<Self, DbusError> {
        Ok(DbusBrightness {
            connection: try!(Connection::get_private(BusType::Session)),
        })
    }

    fn send(&self, msg: Message) -> Result<Message, DbusError> {
		self.connection.send_with_reply_and_block(msg, 2000)
    }
}

impl Brightness for DbusBrightness {
    type E = DbusError;

    fn current(&self) -> Result<f64, Self::E> {
		let rpc = match Message::new_method_call(
                "org.gnome.SettingsDaemon",
                "/org/gnome/SettingsDaemon/Power",
                "org.gnome.SettingsDaemon.Power.Screen",
                "GetPercentage") {
            Ok(m) => m,
            Err(s) => return Err(DbusError::new_custom("CREATE_MESSAGE", &s)),
        };
		let response = try!(self.send(rpc));
		match response.get1::<u32>() {
            Some(level) => Ok(level as f64),
            None => Err(DbusError::new_custom("NO_RETURN",
                "dbus method call did not return an unsigned int")),
        }
    }

    fn max(&self) -> Result<f64, Self::E> {
        Ok(100f64)
    }

    fn min(&self) -> Result<f64, Self::E> {
        Ok(0f64)
    }

    fn set(&self, value: f64) -> Result<(), Self::E> {
		let rpc = match Message::new_method_call(
                "org.gnome.SettingsDaemon",
                "/org/gnome/SettingsDaemon/Power",
                "org.gnome.SettingsDaemon.Power.Screen",
                "SetPercentage") {
            Ok(m) => m,
            Err(s) => return Err(DbusError::new_custom("CREATE_MESSAGE", &s)),
        };
        let rpc = rpc.append1(value as u32);
        self.send(rpc).map(|_| ())
    }
}
