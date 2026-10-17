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

#[derive(Clone, Debug)]
pub struct FfmpegOptions {
    pub logging: bool,
    pub mpegts: bool,
    pub nvenc: bool,
    pub mux_debug: bool,
}

impl Default for FfmpegOptions {
    fn default() -> Self {
        Self {
            logging: false,
            mpegts: true,
            nvenc: false,
            mux_debug: false,
        }
    }
}

#[derive(Debug)]
pub struct FfmpegArtifacts {
    pub module: Module,
    pub muxer: PathBuf,
}

impl FfmpegArtifacts {
    /// Deploy the mux helper beside the executable that loads the module.
    pub fn deploy_muxer(&self, _bin_dir: &Path) -> Result<PathBuf, String> {
        Err("FFmpeg helper deployment is not implemented".into())
    }
}

pub fn build_x264(build: &NativeBuild, root: &Path, core: &Path) -> Result<Module, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut library = module(build, &root, core, "obs-x264")?;
    library.include_dirs.push(root.join("shared/opts-parser"));
    library
        .dependencies
        .push(shared_archive(build, &root, "opts-parser")?);
    add_package(&mut library, &build.target, "x264")?;
    finish(build, &root, library)
}

pub fn build_outputs(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    options: &crate::libobs::Options,
) -> Result<Module, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut library = module(build, &root, core, "obs-outputs")?;
    for group in ["av1", "mp4-mux", "rtmp", "net-if"] {
        library.sources.push(root.join(format!(
            "plugins/obs-outputs/build/obs-outputs.{group}+.h.c"
        )));
    }
    if options.hevc {
        library
            .sources
            .push(root.join("plugins/obs-outputs/build/obs-outputs.hevc+.h.c"));
        library.definitions.push(("ENABLE_HEVC".into(), None));
    }
    for source in [
        "amf", "cencode", "hashswf", "log", "md5", "parseurl", "rtmp",
    ] {
        library
            .sources
            .push(root.join(format!("plugins/obs-outputs/librtmp/{source}.c")));
    }
    for name in ["opts-parser", "happy-eyeballs", "bpm"] {
        library.include_dirs.push(root.join("shared").join(name));
        library
            .dependencies
            .push(shared_archive(build, &root, name)?);
    }
    library
        .dependencies
        .push(crate::libobs::build_caption(build, &root)?);
    library
        .definitions
        .extend(["USE_MBEDTLS", "CRYPTO"].map(|name| (name.into(), None)));
    for package in ["mbedtls", "mbedx509", "mbedcrypto", "zlib", "jansson"] {
        add_package(&mut library, &build.target, package)?;
    }
    crate::platform::configure_outputs(&build.target, &root, &mut library)?;
    finish(build, &root, library)
}

pub fn build_ffmpeg(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    options: &crate::libobs::Options,
    features: &FfmpegOptions,
) -> Result<FfmpegArtifacts, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut library = module(build, &root, core, "obs-ffmpeg")?;
    for group in ["mux", "output", "source", "video-encoders"] {
        library
            .sources
            .push(root.join(format!("plugins/obs-ffmpeg/build/obs-ffmpeg.{group}+.h.c")));
    }
    for (enabled, group, definition) in [
        (features.logging, "logging", "ENABLE_FFMPEG_LOGGING"),
        (features.nvenc, "nvenc", "ENABLE_FFMPEG_NVENC"),
        (features.mpegts, "mpegts", "NEW_MPEGTS_OUTPUT"),
    ] {
        if enabled {
            library
                .sources
                .push(root.join(format!("plugins/obs-ffmpeg/build/obs-ffmpeg.{group}+.h.c")));
            library.definitions.push((definition.into(), None));
        }
    }
    if options.hevc {
        library.definitions.push(("ENABLE_HEVC".into(), None));
    }
    for group in ["cache", "core"] {
        library.sources.push(root.join(format!(
            "shared/media-playback/build/media-playback.{group}+.h.c"
        )));
    }
    library.include_dirs.extend([
        root.join("shared/media-playback"),
        root.join("shared/opts-parser"),
    ]);
    library
        .dependencies
        .push(shared_archive(build, &root, "opts-parser")?);
    for package in [
        "libavcodec",
        "libavfilter",
        "libavformat",
        "libavdevice",
        "libavutil",
        "libswscale",
        "libswresample",
    ] {
        add_package(&mut library, &build.target, package)?;
    }
    if features.mpegts {
        for package in ["librist", "srt"] {
            add_package(&mut library, &build.target, package)?;
        }
    }
    crate::platform::configure_ffmpeg(&build.target, &root, &mut library)?;
    let module = finish(build, &root, library)?;
    let mut mux = SharedLibrary {
        name: "obs-ffmpeg-mux".into(),
        sources: vec![root.join("plugins/obs-ffmpeg/ffmpeg-mux/build/ffmpeg-mux.core+.h.c")],
        include_dirs: vec![
            root.clone(),
            root.join("libobs"),
            root.join("plugins/obs-ffmpeg/ffmpeg-mux"),
            build.output.join("config"),
        ],
        dependencies: vec![core.into()],
        ..SharedLibrary::default()
    };
    crate::platform::configure_module(&build.target, &mut mux)?;
    // Executable naming, not the MODULE filename selected by configure_module.
    mux.file_name = None;
    if features.mux_debug {
        mux.definitions
            .push(("ENABLE_FFMPEG_MUX_DEBUG".into(), None));
    }
    for package in ["simde", "libavcodec", "libavutil", "libavformat"] {
        add_package(&mut mux, &build.target, package)?;
    }
    Ok(FfmpegArtifacts {
        module,
        muxer: build.executable(&mux)?,
    })
}

fn shared_archive(build: &NativeBuild, root: &Path, name: &str) -> Result<PathBuf, String> {
    let mut library = SharedLibrary {
        name: name.into(),
        sources: vec![root.join(format!("shared/{name}/build/{name}.core+.h.c"))],
        include_dirs: vec![
            root.into(),
            root.join("libobs"),
            root.join("shared").join(name),
            root.join("deps/libcaption"),
            root.join("deps/libcaption/caption"),
            build.output.join("config"),
        ],
        ..SharedLibrary::default()
    };
    crate::platform::configure_shared_dependency(&build.target, root, name, &mut library)?;
    add_package(&mut library, &build.target, "simde")?;
    build.static_library(&library)
}
