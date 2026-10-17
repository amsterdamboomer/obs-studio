//! Build the unchanged native C tests through the Cargo native engine.
use crate::NativeBuild;
use std::path::{Path, PathBuf};

pub struct Executable {
    pub name: String,
    pub path: PathBuf,
}

pub fn build_c_tests(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
    _hevc: bool,
) -> Result<Vec<Executable>, String> {
    Err("native C test compilation is not implemented".into())
}
