//! Build the unchanged native C tests through the Cargo native engine.
pub mod graph;

use crate::{NativeBuild, SharedLibrary, libobs::add_package};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub struct Executable {
    pub name: String,
    pub path: PathBuf,
    pub runtime: Option<graph::Test>,
    /// Resolved link artifacts; used to verify core selection only when applicable.
    pub dependencies: Vec<PathBuf>,
}

impl Executable {
    pub fn command(&self, root: &Path, output: &Path) -> Result<std::process::Command, String> {
        let path = dunce::canonicalize(&self.path)
            .map_err(|error| format!("native test executable {}: {error}", self.path.display()))?;
        match &self.runtime {
            Some(runtime) => runtime.command(root, output, &path),
            None => Ok(std::process::Command::new(path)),
        }
    }
}

pub struct Options {
    pub outputs: bool,
    pub filters: bool,
    pub pipewire: bool,
    /// Plugin availability is independent of libobs audio monitoring.
    pub pulseaudio_plugin: bool,
    pub hevc: bool,
    pub pulseaudio: bool,
    pub per_target_sanitizers: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            outputs: true,
            filters: true,
            pipewire: true,
            pulseaudio_plugin: true,
            hevc: true,
            pulseaudio: true,
            per_target_sanitizers: true,
        }
    }
}

/// Build the unchanged standalone WHIP clock and POSIX host tests.
pub fn build_webrtc_tests(build: &NativeBuild, root: &Path) -> Result<Vec<Executable>, String> {
    let root = dunce::canonicalize(root).map_err(|e| e.to_string())?;
    let mut base = SharedLibrary {
        include_dirs: vec![root.clone(), root.join("plugins/obs-webrtc")],
        ..SharedLibrary::default()
    };
    crate::platform::configure_webrtc_tests(&build.target, &mut base)?;
    add_package(&mut base, &build.target, "cmocka")?;
    let mut result = Vec::new();
    for (name, extension) in [
        ("test_whip_rtp_timestamp", "c"),
        ("test_whip_host_os_name", "cpp"),
    ] {
        let library = SharedLibrary {
            name: name.into(),
            sources: vec![root.join(format!("test/cmocka/{name}.{extension}"))],
            ..base.clone()
        };
        result.push(Executable {
            name: name.into(),
            path: build.executable(&library)?,
            runtime: None,
            dependencies: vec![],
        });
    }
    for directory in ["test/cmocka", "plugins/obs-webrtc"] {
        println!("cargo:rerun-if-changed={}", root.join(directory).display());
    }
    Ok(result)
}

/// Build unchanged cmocka tests that require the native graphics module.
pub fn build_graphics_tests(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    graphics: &Path,
) -> Result<Vec<Executable>, String> {
    let root = dunce::canonicalize(root).map_err(|e| e.to_string())?;
    let graphics = dunce::canonicalize(graphics).map_err(|e| e.to_string())?;
    let quoted = serde_json::to_string(graphics.to_str().ok_or("non-UTF8 graphics test path")?)
        .map_err(|e| e.to_string())?;
    let name = "test_group_hide_transition_audio";
    let mut library = SharedLibrary {
        name: name.into(),
        sources: vec![root.join(format!("test/cmocka/{name}.c"))],
        include_dirs: vec![
            root.join("libobs"),
            root.join("deps/libcaption"),
            build.output.join("config"),
        ],
        definitions: vec![("GRAPHICS_MODULE_PATH".into(), Some(quoted))],
        dependencies: vec![core.into(), crate::libobs::build_caption(build, &root)?],
        ..SharedLibrary::default()
    };
    crate::platform::configure_c_tests(build, &mut library, None, core)?;
    for package in ["cmocka", "simde"] {
        add_package(&mut library, &build.target, package)?;
    }
    println!("cargo:rerun-if-changed={}", library.sources[0].display());
    println!(
        "cargo:rerun-if-changed={}",
        root.join("libobs/data").display()
    );
    Ok(vec![Executable {
        name: name.into(),
        path: build.executable(&library)?,
        runtime: None,
        dependencies: vec![core.into()],
    }])
}

/// Build the unchanged VLC compatibility tests against the real native modules.
pub fn build_vlc_tests(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    vlc: &crate::plugins::Module,
    graphics: &Path,
) -> Result<Vec<Executable>, String> {
    let root = dunce::canonicalize(root).map_err(|e| e.to_string())?;
    let mut base = SharedLibrary {
        include_dirs: vec![
            root.clone(),
            root.join("libobs"),
            root.join("test/cmocka"),
            root.join("plugins/vlc-video"),
            build.output.join("config"),
        ],
        dependencies: vec![core.into()],
        ..SharedLibrary::default()
    };
    crate::platform::configure_c_tests(build, &mut base, None, core)?;
    for package in ["cmocka", "simde"] {
        add_package(&mut base, &build.target, package)?;
    }
    for (name, path) in [
        ("VLC_MODULE_PATH", vlc.library.as_path()),
        ("VLC_DATA_PATH", vlc.data.as_path()),
        ("GRAPHICS_MODULE_PATH", graphics),
    ] {
        let path = dunce::canonicalize(path).map_err(|e| e.to_string())?;
        let quoted = serde_json::to_string(path.to_str().ok_or("non-UTF8 VLC test path")?)
            .map_err(|e| e.to_string())?;
        base.definitions.push((name.into(), Some(quoted)));
    }
    let mut result = Vec::new();
    for name in [
        "test_vlc_drain_gate",
        "test_vlc_host_path",
        "test_vlc_channel_order",
        "test_vlc_audio_track",
        "test_vlc_audio_pacing",
        "test_vlc_playlist_gap",
    ] {
        let mut library = SharedLibrary {
            name: name.into(),
            sources: vec![root.join(format!("test/cmocka/{name}.c"))],
            ..base.clone()
        };
        if name == "test_vlc_audio_pacing" {
            for package in ["libavcodec", "libavformat", "libavutil"] {
                add_package(&mut library, &build.target, package)?;
            }
        }
        result.push(Executable {
            name: name.into(),
            path: build.executable(&library)?,
            runtime: None,
            dependencies: vec![core.into()],
        });
    }
    for directory in ["test/cmocka", "plugins/vlc-video", "libobs/data"] {
        println!("cargo:rerun-if-changed={}", root.join(directory).display());
    }
    Ok(result)
}

pub fn build_c_tests(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    options: &Options,
) -> Result<Vec<Executable>, String> {
    let root = dunce::canonicalize(root).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    let graph_path = root.join("test/cmocka/build/test-graph.json");
    println!("cargo:rerun-if-changed={}", graph_path.display());
    let mut graph = graph::Graph::load(&graph_path)?;
    if !options.per_target_sanitizers {
        for target in graph.targets.values_mut() {
            target.sanitizers = None;
        }
    }
    let mut facts =
        crate::platform::native_test_facts(&build.target, options.pulseaudio, options.hevc);
    facts.extend([
        ("HAVE_OBS_OUTPUTS".into(), options.outputs),
        (
            "C_COMPILER_CLANG".into(),
            crate::platform::native_test_compiler_is_clang(build),
        ),
        ("HAVE_OBS_FILTERS".into(), options.filters),
        ("HAVE_OBS_PIPEWIRE".into(), options.pipewire),
        ("HAVE_OBS_PULSEAUDIO".into(), options.pulseaudio_plugin),
    ]);
    let mut core_interface = SharedLibrary {
        include_dirs: vec![
            root.join("libobs"),
            root.join("deps/libcaption"),
            build.output.join("config"),
        ],
        dependencies: vec![core.into()],
        ..Default::default()
    };
    if options.hevc {
        core_interface
            .definitions
            .push(("ENABLE_HEVC".into(), None));
    }
    add_package(&mut core_interface, &build.target, "simde")?;
    headers_only(&mut core_interface, "jansson")?;
    crate::platform::configure_c_tests(build, &mut core_interface, None, core)?;
    let mut cmocka_interface = SharedLibrary::default();
    add_package(&mut cmocka_interface, &build.target, "cmocka")?;
    let thread_options = crate::platform::native_test_thread_options(&build.target);
    let mut interfaces = BTreeMap::from([
        (
            "Threads::Threads".into(),
            graph::Dependency {
                compile_options: thread_options.clone(),
                link_options: thread_options,
                ..Default::default()
            },
        ),
        ("OBS::libobs".into(), core_interface.into()),
        ("OBS::native-test-cmocka".into(), cmocka_interface.into()),
        (
            "m".into(),
            graph::Dependency {
                link_options: crate::platform::native_test_math_options(&build.target),
                ..Default::default()
            },
        ),
    ]);
    let caption = crate::libobs::build_caption(build, &root)?;
    interfaces.insert(
        "OBS::caption".into(),
        graph::Dependency {
            artifacts: vec![caption.clone()],
            include_dirs: vec![root.join("deps/libcaption")],
            ..Default::default()
        },
    );
    for component in ["avcodec", "avformat", "avutil"] {
        let mut dependency = SharedLibrary::default();
        add_package(&mut dependency, &build.target, &format!("lib{component}"))?;
        interfaces.insert(format!("FFmpeg::{component}"), dependency.into());
    }
    let mut zlib = SharedLibrary::default();
    add_package(&mut zlib, &build.target, "zlib")?;
    interfaces.insert("ZLIB::ZLIB".into(), zlib.into());
    // This Linux-only compatibility builder inherits the compiler's system
    // header search (including CFLAGS/CPATH). Uthash is header-only and already
    // required by the built core; it contributes no artifact or link flags.
    interfaces.insert("Uthash::Uthash".into(), graph::Dependency::default());
    if options.pipewire {
        let mut dependency = SharedLibrary::default();
        add_package(&mut dependency, &build.target, "libpipewire-0.3")?;
        interfaces.insert("PipeWire::PipeWire".into(), dependency.into());
    }
    if options.pulseaudio_plugin {
        let mut dependency = SharedLibrary::default();
        add_package(&mut dependency, &build.target, "libpulse")?;
        interfaces.insert("PulseAudio::PulseAudio".into(), dependency.into());
    }
    if options.outputs {
        let mut happy = graph::Dependency {
            artifacts: vec![crate::deps::shared_archive(build, &root, "happy-eyeballs")?],
            include_dirs: crate::platform::native_test_happy_includes(&build.target, &root),
            ..Default::default()
        };
        happy.artifacts.push(core.into());
        interfaces.insert("OBS::happy-eyeballs".into(), happy);
        let mut tls = SharedLibrary::default();
        for package in ["mbedtls", "mbedx509", "mbedcrypto"] {
            add_package(&mut tls, &build.target, package)?;
        }
        interfaces.insert("MbedTLS::mbedtls".into(), tls.into());
        for name in ["opts-parser", "bpm"] {
            let mut artifacts = vec![
                crate::deps::shared_archive(build, &root, name)?,
                core.into(),
            ];
            if name == "bpm" {
                artifacts.push(caption.clone());
            }
            interfaces.insert(
                format!("OBS::{name}"),
                graph::Dependency {
                    artifacts,
                    include_dirs: vec![root.join("shared").join(name)],
                    ..Default::default()
                },
            );
        }
        let mut jansson = SharedLibrary::default();
        add_package(&mut jansson, &build.target, "jansson")?;
        interfaces.insert("jansson::jansson".into(), jansson.into());
        for name in ["crypt32", "iphlpapi", "winmm", "ws2_32"] {
            interfaces.insert(
                name.into(),
                graph::Dependency {
                    link_options: crate::platform::native_test_system_library(&build.target, name),
                    ..Default::default()
                },
            );
        }
        for name in ["Foundation", "Security"] {
            interfaces.insert(
                format!("OBS::native-test-{}", name.to_lowercase()),
                graph::Dependency {
                    link_options: crate::platform::native_test_framework_options(
                        &build.target,
                        name,
                    ),
                    ..Default::default()
                },
            );
        }
    }
    let artifacts = graph.compile(build, &root, &facts, &interfaces)?;
    for (name, test) in graph.select(&facts)?.tests {
        result.push(Executable {
            name: name.into(),
            path: artifacts
                .get(&test.target)
                .ok_or("selected native test artifact missing")?
                .clone(),
            runtime: Some(test.clone()),
            dependencies: graph.targets[&test.target].dependency_artifacts(&facts, &interfaces)?,
        });
    }
    for directory in ["libobs", "test/cmocka", "deps/libcaption"] {
        println!("cargo:rerun-if-changed={}", root.join(directory).display());
    }
    Ok(result)
}

fn headers_only(library: &mut SharedLibrary, package: &str) -> Result<(), String> {
    let headers = pkg_config::Config::new()
        .cargo_metadata(false)
        .probe(package)
        .map_err(|e| e.to_string())?;
    library.include_dirs.extend(headers.include_paths);
    let mut definitions: Vec<_> = headers.defines.into_iter().collect();
    definitions.sort();
    library.definitions.extend(definitions);
    Ok(())
}
