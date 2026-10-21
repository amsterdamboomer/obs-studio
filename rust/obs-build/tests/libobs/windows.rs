//! Explicit Windows SDK/dependency integration in the existing application harness.
use obs_build::{NativeBuild, libobs};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[test]
fn core_builds_from_the_canonical_manifest() {
    let Some(dependencies) = std::env::var_os("OBS_BUILD_WINDOWS_DEPENDENCIES") else {
        return;
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let build = NativeBuild {
        host: obs_build::BUILD_HOST.into(),
        target: std::env::var("OBS_BUILD_WINDOWS_TARGET")
            .unwrap_or_else(|_| obs_build::BUILD_TARGET.into()),
        output: std::env::var_os("OBS_BUILD_WINDOWS_OUTPUT")
            .expect("set OBS_BUILD_WINDOWS_OUTPUT")
            .into(),
    };
    assert!(build.target.contains("windows-msvc"));
    let options = libobs::Options {
        native_dependencies: Some(dependencies.into()),
        hevc: std::env::var_os("OBS_BUILD_DISABLE_HEVC").is_none(),
        unity: std::env::var_os("OBS_BUILD_WINDOWS_SEPARATE").is_none(),
        pch: std::env::var_os("OBS_BUILD_WINDOWS_PCH").is_some(),
        ..libobs::Options::default()
    };
    let library = if let Some(archive) = std::env::var_os("OBS_BUILD_WINDOWS_RUST_ARCHIVE") {
        let pair = libobs::build_pair(&build, &root, &options, &PathBuf::from(archive)).unwrap();
        let reference = exports(&pair.reference);
        assert_eq!(reference, exports(&pair.with_rust));
        if let Some(original) = std::env::var_os("OBS_BUILD_WINDOWS_REFERENCE") {
            assert_eq!(
                reference,
                exports(Path::new(&original)),
                "Cargo must preserve the actual CMake reference exports"
            );
        }
        std::fs::write(
            build.output.join("windows-symbols.json"),
            serde_json::to_vec_pretty(&reference).unwrap(),
        )
        .unwrap();
        pair.with_rust
    } else {
        libobs::build_c_reference_with_options(&build, &root, &options).unwrap()
    };
    assert_eq!(library.file_name().unwrap(), "obs.dll");
    assert!(library.is_file());
    assert!(
        library.with_extension("lib").is_file(),
        "the public DLL needs an import library"
    );
    assert!(!exports(&library).is_empty());
    std::fs::write(
        build.output.join("windows-core-path.txt"),
        library.to_str().unwrap(),
    )
    .unwrap();
}

fn exports(library: &Path) -> BTreeSet<String> {
    let tool =
        std::env::var_os("OBS_BUILD_WINDOWS_EXPORT_TOOL").unwrap_or_else(|| "llvm-readobj".into());
    let dumpbin = Path::new(&tool)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .eq_ignore_ascii_case("dumpbin");
    let result = std::process::Command::new(tool)
        .arg(if dumpbin {
            "/exports"
        } else {
            "--coff-exports"
        })
        .arg(library)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let exports: BTreeSet<_> = String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            if dumpbin {
                let fields: Vec<_> = line.split_whitespace().collect();
                (fields.len() >= 4
                    && fields[0].parse::<u32>().is_ok()
                    && u32::from_str_radix(fields[1], 16).is_ok()
                    && u32::from_str_radix(fields[2], 16).is_ok())
                .then(|| fields[3].to_owned())
            } else {
                line.trim().strip_prefix("Name: ").map(str::to_owned)
            }
        })
        .collect();
    assert!(
        !exports.is_empty(),
        "no exports parsed from {}",
        library.display()
    );
    exports
}
