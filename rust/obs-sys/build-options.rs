//! Options shared by the build script and its configuration contracts.
use std::ffi::OsString;

pub const VARIABLES: [&str; 2] = ["OBS_SYS_UNITY", "OBS_SYS_PCH"];

#[derive(Debug, PartialEq, Eq)]
pub struct BuildModes {
    pub unity: bool,
    pub pch: bool,
}

impl BuildModes {
    pub fn from_lookup(_lookup: impl FnMut(&str) -> Option<OsString>) -> Result<Self, String> {
        Err("bundled native build modes are not implemented".into())
    }
}
