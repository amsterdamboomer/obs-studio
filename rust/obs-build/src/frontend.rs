//! Native frontend support libraries and application build graph.
use crate::{NativeBuild, SharedLibrary, libobs::add_package};
use std::path::{Path, PathBuf};

pub fn build_api(build: &NativeBuild, root: &Path, core: &Path) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut library = SharedLibrary {
        name: "obs-frontend-api".into(),
        sources: vec![root.join("frontend/api/build/obs-frontend-api.core+.hpp.cpp")],
        include_dirs: vec![
            root.clone(),
            root.join("libobs"),
            root.join("frontend/api"),
            build.output.join("config"),
        ],
        dependencies: vec![core.canonicalize().map_err(|e| e.to_string())?],
        ..SharedLibrary::default()
    };
    crate::platform::configure_frontend_api(&build.target, &mut library)?;
    add_package(&mut library, &build.target, "simde")?;
    let artifact = build.shared_library(&library)?;
    let lookup = artifact.with_file_name(crate::platform::shared_filename(
        &build.target,
        &library.name,
    ));
    if artifact != lookup {
        std::fs::copy(&artifact, lookup).map_err(|e| e.to_string())?;
    }
    Ok(artifact)
}

pub fn add_spinbox(
    _build: &NativeBuild,
    _root: &Path,
    _library: &mut SharedLibrary,
) -> Result<(), String> {
    Err("Qt spinbox build is not implemented".into())
}
