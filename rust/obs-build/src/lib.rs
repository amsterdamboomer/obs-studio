//! Native compilation and dynamic linking for the Cargo-driven OBS build.
//!
//! This engine owns native artifacts. Cargo remains responsible for compiling
//! Rust crates; native compiles go through cc::Build and inherit soldr's tools.
#![forbid(unsafe_code)]

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct SharedLibrary {
    pub name: String,
    pub sources: Vec<PathBuf>,
    pub include_dirs: Vec<PathBuf>,
    pub dependencies: Vec<PathBuf>,
    pub unity: bool,
}

#[derive(Clone, Debug)]
pub struct NativeBuild {
    pub host: String,
    pub target: String,
    pub output: PathBuf,
}

impl NativeBuild {
    /// Compile through cc::Build, then link a native shared library.
    pub fn shared_library(&self, _library: &SharedLibrary) -> Result<PathBuf, String> {
        Err("native shared-library compilation is not implemented".into())
    }
}
