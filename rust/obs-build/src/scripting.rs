//! Native scripting library and generated language bindings.
use crate::NativeBuild;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Options {
    pub lua: bool,
    pub python: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            lua: true,
            python: true,
        }
    }
}

#[derive(Debug)]
pub struct Artifacts {
    pub library: PathBuf,
    pub lua_module: Option<PathBuf>,
    pub python_module: Option<PathBuf>,
    pub python_wrapper: Option<PathBuf>,
}

pub fn build(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
    _frontend: Option<&Path>,
    _options: &Options,
) -> Result<Artifacts, String> {
    Err("scripting native graph is not implemented".into())
}
