//! Native macOS capture language settings and framework dependencies.
use crate::{
    NativeBuild, SharedLibrary, SourceCompileOptions,
    libobs::Options,
    plugins::{MacosCaptureModule, Module},
};
use std::{collections::BTreeMap, path::Path};

pub fn capture(
    build: &NativeBuild,
    root: &Path,
    core: &Path,
    options: &Options,
    module: MacosCaptureModule,
) -> Result<Module, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let manifest = root.join("plugins/mac-capture/source-sets.json");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let sets: BTreeMap<String, Vec<String>> =
        serde_json::from_slice(&std::fs::read(manifest).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let key = module.name().replace('-', "_").to_uppercase();
    let sources = sets
        .get(&key)
        .ok_or_else(|| format!("missing capture source set {key}"))?;
    let directory = match module {
        MacosCaptureModule::ScreenAudio => "mac-capture",
        MacosCaptureModule::Camera | MacosCaptureModule::LegacyCamera => "mac-avcapture",
    };
    let dependencies = options
        .native_dependencies
        .as_ref()
        .ok_or("macOS capture requires an explicit OBS dependency bundle")?;
    let include = dependencies
        .join("include")
        .canonicalize()
        .map_err(|e| e.to_string())?;
    if !include.join("simde/x86/sse2.h").is_file() {
        return Err("macOS capture dependency bundle lacks simde".into());
    }
    let mut library = SharedLibrary {
        name: module.name().into(),
        file_name: Some(module.name().into()),
        sources: sources.iter().map(|path| root.join(path)).collect(),
        include_dirs: vec![
            root.clone(),
            root.join("libobs"),
            root.join("plugins").join(directory),
            build.output.join("config"),
            include,
        ],
        dependencies: vec![core.canonicalize().map_err(|e| e.to_string())?],
        compile_options: [
            "-fno-strict-aliasing",
            "-fvisibility=hidden",
            "-mmacosx-version-min=13.0",
        ]
        .map(String::from)
        .into(),
        c_compile_options: vec!["-std=gnu17".into()],
        cpp_compile_options: vec!["-std=gnu++17".into()],
        ..SharedLibrary::default()
    };
    let objc = library
        .sources
        .iter()
        .filter(|source| {
            matches!(
                source.extension().and_then(|value| value.to_str()),
                Some("m" | "mm")
            )
        })
        .cloned()
        .collect();
    let objc_options = match module {
        MacosCaptureModule::ScreenAudio => vec!["-fno-objc-arc".into()],
        MacosCaptureModule::Camera => vec!["-fobjc-arc".into(), "-fmodules".into()],
        MacosCaptureModule::LegacyCamera => vec!["-fobjc-arc".into()],
    };
    library.source_compile_options.push(SourceCompileOptions {
        sources: objc,
        options: objc_options,
    });
    let frameworks: &[&str] = match module {
        MacosCaptureModule::ScreenAudio => &[
            "AudioUnit",
            "Cocoa",
            "CoreAudio",
            "CoreFoundation",
            "CoreMedia",
            "CoreVideo",
            "IOSurface",
        ],
        MacosCaptureModule::Camera | MacosCaptureModule::LegacyCamera => &[
            "AVFoundation",
            "Cocoa",
            "CoreFoundation",
            "CoreMedia",
            "CoreVideo",
            "CoreMediaIO",
            "IOSurface",
        ],
    };
    for framework in frameworks {
        library
            .link_options
            .extend(["-framework".into(), (*framework).into()]);
    }
    if matches!(module, MacosCaptureModule::ScreenAudio) {
        library
            .link_options
            .extend(["-weak_framework".into(), "ScreenCaptureKit".into()]);
    }
    if matches!(module, MacosCaptureModule::LegacyCamera) {
        library
            .include_dirs
            .push(root.join("plugins/mac-avcapture/legacy"));
    }
    if !options.unity {
        build.separate_target_sources(&root, &mut library)?;
    }
    if options.pch {
        library.precompiled_headers = crate::groups::headers(&root, &library.sources)?;
    }
    Ok(Module {
        library: build.shared_library(&library)?,
        data: root.join("plugins").join(directory).join("data"),
    })
}
