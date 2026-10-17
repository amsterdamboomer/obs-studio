//! Graphics backends built from the shared native OBS configuration.
use crate::{NativeBuild, SharedLibrary, libobs::Options};
use std::path::{Path, PathBuf};

pub fn build_opengl(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    options: &Options,
) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut library = SharedLibrary {
        name: "obs-opengl".into(),
        sources: vec![root.join("libobs-opengl/build/opengl.core+.h.c")],
        include_dirs: vec![
            root.clone(),
            root.join("libobs"),
            root.join("libobs-opengl"),
            root.join("deps/glad/include"),
            build.output.join("config"),
        ],
        dependencies: vec![core.canonicalize().map_err(|e| e.to_string())?],
        ..SharedLibrary::default()
    };
    crate::platform::configure_opengl(&build.target, &root, options, &mut library)?;
    let mut glad = SharedLibrary {
        name: "obsglad".into(),
        sources: vec![root.join("deps/glad/src/glad.c")],
        include_dirs: vec![root.join("deps/glad/include")],
        compile_options: library.compile_options.clone(),
        ..SharedLibrary::default()
    };
    crate::platform::configure_glad(&build.target, &root, &mut glad)?;
    library.dependencies.push(build.static_library(&glad)?);
    let artifact = build.shared_library(&library)?;
    let linker_name = artifact.with_file_name(crate::platform::shared_filename(
        &build.target,
        "obs-opengl",
    ));
    if artifact != linker_name {
        std::fs::copy(&artifact, &linker_name).map_err(|e| e.to_string())?;
    }
    Ok(artifact)
}
