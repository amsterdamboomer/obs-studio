//! The native libobs graph, shared by bundled obs-sys and the application build.
use crate::NativeBuild;
use std::path::{Path, PathBuf};

/// Build the real C libobs, including generated configuration and native dependencies.
pub fn build(build: &NativeBuild, root: &Path) -> Result<PathBuf, String> {
    let _ = (build, root);
    Err("the real libobs native build is not implemented".into())
}
