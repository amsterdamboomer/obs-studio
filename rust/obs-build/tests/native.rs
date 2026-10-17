//! Build-system integration contracts; these compile and load actual C artifacts.
use obs_build::{NativeBuild, SharedLibrary};
use std::{fs, path::Path};

fn target(name: &str, source: &Path, dependencies: Vec<std::path::PathBuf>) -> SharedLibrary {
    SharedLibrary {
        name: name.into(),
        sources: vec![source.into()],
        include_dirs: vec![],
        dependencies,
        unity: true,
        ..SharedLibrary::default()
    }
}

unsafe fn answer(library: &Path, symbol: &[u8], dependencies: &[&Path]) -> i32 {
    // SAFETY: the fixture below defines this symbol with the C ABI and no args;
    // its owning library stays loaded for the entire call.
    unsafe {
        // Load dependencies explicitly as well: Windows does not search sibling
        // target directories for DLLs. Keep their handles alive through the call.
        let _dependencies: Vec<_> = dependencies
            .iter()
            .map(|path| libloading::Library::new(path).unwrap())
            .collect();
        let lib = libloading::Library::new(library).unwrap();
        let function = lib.get::<unsafe extern "C" fn() -> i32>(symbol).unwrap();
        function()
    }
}

#[test]
fn native_plugin_links_shared_core_and_observes_rebuilt_dependency() {
    let dir = tempfile::tempdir().unwrap();
    let core_source = dir.path().join("core.c");
    let plugin_source = dir.path().join("plugin.c");
    fs::write(
        &core_source,
        "__declspec_placeholder__ int obs_fixture_answer(void) { return 42; }"
            .replace("__declspec_placeholder__", EXPORT),
    )
    .unwrap();
    fs::write(&plugin_source, format!("extern int obs_fixture_answer(void); {EXPORT} int obs_fixture_plugin(void) {{ return obs_fixture_answer() + 1; }}")).unwrap();
    let build = NativeBuild {
        host: env!("OBS_BUILD_HOST").into(),
        target: env!("OBS_BUILD_TARGET").into(),
        output: dir.path().join("artifacts"),
    };
    let core = build
        .shared_library(&target("fixture-core", &core_source, vec![]))
        .unwrap();
    let plugin = build
        .shared_library(&target(
            "fixture-plugin",
            &plugin_source,
            vec![core.clone()],
        ))
        .unwrap();
    // SAFETY: both sources above define the expected no-argument C functions.
    assert_eq!(
        unsafe { answer(&plugin, b"obs_fixture_plugin", &[&core]) },
        43
    );
    fs::write(
        &core_source,
        format!("{EXPORT} int obs_fixture_answer(void) {{ return 84; }}"),
    )
    .unwrap();
    build
        .shared_library(&target("fixture-core", &core_source, vec![]))
        .unwrap();
    // SAFETY: the rebuilt core preserves the function ABI; it changes its result.
    assert_eq!(
        unsafe { answer(&plugin, b"obs_fixture_plugin", &[&core]) },
        85
    );
}

#[test]
fn unity_toggle_preserves_exports_and_explicit_generated_source_wins() {
    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("source");
    let generated_dir = dir.path().join("generated");
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&generated_dir).unwrap();
    fs::write(
        source_dir.join("manifest.c"),
        "#error stale source tree manifest selected\n",
    )
    .unwrap();
    let generated = generated_dir.join("manifest.c");
    fs::write(
        &generated,
        format!("{EXPORT} int obs_fixture_count(void) {{ return 17; }}"),
    )
    .unwrap();
    let extra = source_dir.join("extra.c");
    fs::write(&extra, "int obs_fixture_other(void) { return 23; }").unwrap();
    for unity in [false, true] {
        let build = NativeBuild {
            host: env!("OBS_BUILD_HOST").into(),
            target: env!("OBS_BUILD_TARGET").into(),
            output: dir.path().join(format!("output-{unity}")),
        };
        let mut library = target("fixture-manifest", &generated, vec![]);
        library.sources.push(extra.clone());
        library.include_dirs.push(source_dir.clone());
        library.unity = unity;
        let artifact = build.shared_library(&library).unwrap();
        // SAFETY: the generated fixture defines this no-argument C function.
        assert_eq!(unsafe { answer(&artifact, b"obs_fixture_count", &[]) }, 17);
    }
}

#[test]
fn executable_runs_against_its_shared_core_dependency() {
    let dir = tempfile::tempdir().unwrap();
    let core_source = dir.path().join("core.c");
    let executable_source = dir.path().join("main.c");
    let helper_source = dir.path().join("helper.c");
    fs::write(
        &helper_source,
        "int fixture_private_value(void) { return 42; }",
    )
    .unwrap();
    fs::write(
        &core_source,
        format!("extern int fixture_private_value(void); {EXPORT} int fixture_value(void) {{ return fixture_private_value(); }}"),
    )
    .unwrap();
    fs::write(
        &executable_source,
        "extern int fixture_value(void); int main(void) { return fixture_value() == 42 ? 0 : 7; }",
    )
    .unwrap();
    let build = NativeBuild {
        host: env!("OBS_BUILD_HOST").into(),
        target: env!("OBS_BUILD_TARGET").into(),
        output: dir.path().join("artifacts"),
    };
    let helper = build
        .static_library(&target("fixture-helper", &helper_source, vec![]))
        .unwrap();
    let core = build
        .shared_library(&target("fixture-core", &core_source, vec![helper]))
        .unwrap();
    let executable = build
        .executable(&target(
            "fixture-app",
            &executable_source,
            vec![core.clone()],
        ))
        .unwrap();
    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let search = std::env::join_paths(
        std::iter::once(core.parent().unwrap().to_path_buf())
            .chain(std::env::split_paths(&old_path)),
    )
    .unwrap();
    assert!(
        std::process::Command::new(executable)
            .env("PATH", search)
            .status()
            .unwrap()
            .success()
    );
}

const EXPORT: &str = "\n#if defined(_WIN32)\n__declspec(dllexport)\n#endif\n";
