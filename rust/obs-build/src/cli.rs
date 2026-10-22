//! Safe command parser and production application-build orchestration.
use crate::{NativeProfile, application};
use std::{ffi::OsString, path::PathBuf};

pub enum Invocation {
    Help,
    Build(Request),
}

pub struct Request {
    pub root: PathBuf,
    pub output: PathBuf,
    pub target: String,
    pub profile: NativeProfile,
    pub options: application::BuildOptions,
}

pub fn parse<I, S>(_arguments: I) -> Result<Invocation, String>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    Err("production application build command is not implemented".into())
}
