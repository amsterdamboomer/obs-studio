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

/// Add the shared Qt spinbox interface sources to a native target.
pub fn add_spinbox(
    build: &NativeBuild,
    root: &Path,
    library: &mut SharedLibrary,
) -> Result<(), String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    crate::platform::configure_cpp17(&build.target, library);
    let directory = root.join("shared/qt/obs-spinbox");
    library
        .sources
        .push(directory.join("build/qt-obs-spinbox.core+.hpp.cpp"));
    library.include_dirs.push(directory.clone());
    add_package(library, &build.target, "Qt6Widgets")?;
    for name in ["AutoSelectLineEdit", "DoubleSpinBox", "SpinBox"] {
        crate::qt::moc(
            library,
            &directory.join(format!("{name}.hpp")),
            &build
                .output
                .join(&library.name)
                .join(format!("qt/moc_{name}.cpp")),
        )?;
    }
    Ok(())
}

pub fn build_application(
    _build: &NativeBuild,
    _root: &Path,
    _core: &Path,
    _api: &Path,
    _gpu_status_archive: &Path,
) -> Result<PathBuf, String> {
    Err("full frontend application build is not implemented".into())
}
