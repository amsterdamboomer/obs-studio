//! Graphics backends built from the shared native OBS configuration.
use crate::{NativeBuild, libobs::Options};
use std::path::{Path, PathBuf};

pub fn build_opengl(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
    _options: &Options,
) -> Result<PathBuf, String> {
    Err("real OBS OpenGL backend build is not implemented".into())
}
