//! Session D-Bus sleep inhibitor from `util/platform-nix-dbus.c.h`.

use std::ffi::CStr;

/// The four services, in the same preference order as the C implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    ScreenSaver,
    PowerManagement,
    MateSession,
    GnomeSession,
}

pub const SERVICES: [Service; 4] = [
    Service::ScreenSaver,
    Service::PowerManagement,
    Service::MateSession,
    Service::GnomeSession,
];

/// Host calls used by the safe state machine. GIO lives in the C ABI shim.
pub trait Host {
    type Connection;

    fn connect(&self) -> Result<Self::Connection, String>;
    fn has_owner(&self, connection: &Self::Connection, service: Service) -> bool;
    fn inhibit(
        &self,
        connection: &Self::Connection,
        service: Service,
        reason: &CStr,
    ) -> Result<u32, String>;
    fn uninhibit(
        &self,
        connection: &Self::Connection,
        service: Service,
        cookie: u32,
    ) -> Result<(), String>;
    fn log(&self, level: i32, message: &str);
}

/// Opaque `struct dbus_sleep_info` state.
pub struct DbusInfo<H: Host> {
    host: H,
    connection: H::Connection,
    service: Service,
    cookie: u32,
}

impl<H: Host> DbusInfo<H> {
    /// `dbus_sleep_info_create`.
    pub fn create(_host: H) -> Option<Self> {
        todo!()
    }

    pub fn service(&self) -> Service {
        self.service
    }

    pub fn cookie(&self) -> u32 {
        self.cookie
    }

    /// `dbus_inhibit_sleep`.
    pub fn set_active(&mut self, _reason: &CStr, _active: bool) {
        todo!()
    }
}
