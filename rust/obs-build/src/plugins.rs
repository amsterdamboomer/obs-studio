//! Native OBS module builds using the same engine as bundled libobs.
use crate::NativeBuild;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Module {
    pub library: PathBuf,
    pub data: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FiltersOptions {
    pub speexdsp: bool,
    pub rnnoise: bool,
}

impl Default for FiltersOptions {
    fn default() -> Self {
        Self {
            speexdsp: true,
            rnnoise: true,
        }
    }
}

pub fn build_transitions(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
) -> Result<Module, String> {
    Err("real OBS transition module build is not implemented".into())
}

pub fn build_filters(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
    _options: &FiltersOptions,
) -> Result<Module, String> {
    Err("real OBS filter module build is not implemented".into())
}
