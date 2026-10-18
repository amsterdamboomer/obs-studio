//! The original browser source, browser-panel interfaces and CEF helper.
use super::Module;
use crate::NativeBuild;
use std::path::{Path, PathBuf};

pub struct Options {
    pub cef_root: PathBuf,
    pub panels: bool,
    pub unity: bool,
}

pub struct Artifacts {
    pub module: Module,
    pub helper: PathBuf,
    pub cef_root: PathBuf,
}

pub fn build(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
    _api: &Path,
    _options: &Options,
) -> Result<Artifacts, String> {
    Err("native browser module and CEF helper are not implemented".into())
}
