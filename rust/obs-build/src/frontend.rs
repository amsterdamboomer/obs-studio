//! Native frontend support libraries and application build graph.
use crate::NativeBuild;
use std::path::{Path, PathBuf};

pub fn build_api(_build: &NativeBuild, _root: &Path, _core: &Path) -> Result<PathBuf, String> {
    Err("frontend API build is not implemented".into())
}
