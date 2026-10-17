//! Native OBS module builds using the same engine as bundled libobs.
use crate::{NativeBuild, SharedLibrary, libobs::add_package};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Module {
    pub library: PathBuf,
    pub data: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FiltersOptions {
    pub speexdsp: bool,
    pub rnnoise: bool,
}

impl Default for FiltersOptions {
    fn default() -> Self {
        Self {
            speexdsp: true,
            rnnoise: true,
        }
    }
}

pub fn build_transitions(build: &NativeBuild, root: &Path, core: &Path) -> Result<Module, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let library = module(build, &root, core, "obs-transitions")?;
    finish(build, &root, library)
}

pub fn build_filters(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    options: &FiltersOptions,
) -> Result<Module, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut library = module(build, &root, core, "obs-filters")?;
    if options.speexdsp {
        add_package(&mut library, &build.target, "speexdsp")?;
        library
            .definitions
            .push(("LIBSPEEXDSP_ENABLED".into(), None));
    }
    if options.rnnoise {
        if add_package(&mut library, &build.target, "rnnoise").is_err() {
            // Match OBS's Linux fallback when no system RNNoise is installed.
            let directory = root.join("plugins/obs-filters");
            let sources: Vec<String> = serde_json::from_slice(
                &std::fs::read(directory.join("build/rnnoise-sources.json"))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let rnnoise = SharedLibrary {
                name: "obs-rnnoise".into(),
                sources: sources
                    .iter()
                    .map(|source| directory.join(source))
                    .collect(),
                include_dirs: vec![directory.join("rnnoise/include")],
                definitions: vec![("COMPILE_OPUS".into(), None)],
                compile_options: library.compile_options.clone(),
                ..SharedLibrary::default()
            };
            library.dependencies.push(build.static_library(&rnnoise)?);
            library.include_dirs.extend(rnnoise.include_dirs);
        }
        library
            .definitions
            .push(("LIBRNNOISE_ENABLED".into(), None));
    }
    if options.speexdsp || options.rnnoise {
        library
            .sources
            .push(root.join("plugins/obs-filters/build/obs-filters.noise-suppress+.h.c"));
        library
            .definitions
            .push(("HAS_NOISEREDUCTION".into(), None));
    }
    finish(build, &root, library)
}

fn module(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    name: &str,
) -> Result<SharedLibrary, String> {
    let mut library = SharedLibrary {
        name: name.into(),
        sources: vec![root.join(format!("plugins/{name}/build/{name}.core+.h.c"))],
        include_dirs: vec![
            root.into(),
            root.join("libobs"),
            root.join("plugins").join(name),
            build.output.join("config"),
        ],
        dependencies: vec![core.canonicalize().map_err(|e| e.to_string())?],
        ..SharedLibrary::default()
    };
    crate::platform::configure_module(&build.target, &mut library)?;
    // libobs public graphics headers expose SIMDe vector types.
    add_package(&mut library, &build.target, "simde")?;
    Ok(library)
}

fn finish(build: &NativeBuild, root: &Path, library: SharedLibrary) -> Result<Module, String> {
    Ok(Module {
        data: root.join("plugins").join(&library.name).join("data"),
        library: build.shared_library(&library)?,
    })
}
