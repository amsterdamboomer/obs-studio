//! Production application assembly and development runtime artifacts.
use crate::{NativeBuild, plugins, scripting};
use std::path::{Path, PathBuf};

/// Explicit artifacts to stage; no discovery of same-named source-tree outputs.
pub struct RuntimeInputs<'a> {
    pub root: &'a Path,
    pub executable: &'a Path,
    pub core: &'a Path,
    pub api: &'a Path,
    pub renderer: &'a Path,
    pub ffmpeg: &'a plugins::FfmpegArtifacts,
    pub nvenc_helper: Option<&'a Path>,
    pub modules: &'a [&'a plugins::Module],
    pub browser: Option<&'a plugins::BrowserArtifacts>,
    pub scripting: Option<&'a scripting::Artifacts>,
}

pub struct StagedRuntime {
    pub directory: PathBuf,
    pub executable: PathBuf,
    pub modules: Vec<String>,
    pub scripting: Option<serde_json::Value>,
}

/// Stage into an empty destination, retaining only explicitly selected inputs.
/// This is a development runtime, not a portable signed release package.
pub fn stage_runtime(
    build: &NativeBuild,
    inputs: &RuntimeInputs<'_>,
    destination: &Path,
) -> Result<StagedRuntime, String> {
    crate::platform::stage_application_runtime(build, inputs, destination)
}
