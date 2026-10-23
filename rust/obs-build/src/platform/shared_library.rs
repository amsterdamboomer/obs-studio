//! Shared-dependency resolution and its private regression check.

use std::path::PathBuf;

/// Prefer a shared dependency even when a .pc file lists static paths first.
fn package_shared_library_name(target: &str, name: &str) -> Option<String> {
    if target.contains("linux") {
        Some(format!("lib{name}.so"))
    } else if target.contains("apple") {
        Some(format!("lib{name}.dylib"))
    } else {
        // Windows requires import-library handling rather than linking the DLL.
        None
    }
}

/// Link options for a pkg-config package's `-L` directories and `-l` names.
pub fn package_link_options(target: &str, paths: &[PathBuf], libs: &[String]) -> Vec<String> {
    todo!("{target} {paths:?} {libs:?}")
}

pub fn package_shared_library(target: &str, paths: &[PathBuf], name: &str) -> Option<PathBuf> {
    let filename = package_shared_library_name(target, name)?;
    paths
        .iter()
        .map(|path| path.join(&filename))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_dependency_wins_over_an_earlier_static_archive() {
        let dir = tempfile::tempdir().unwrap();
        let archive_dir = dir.path().join("static");
        let shared_dir = dir.path().join("shared");
        std::fs::create_dir_all(&archive_dir).unwrap();
        std::fs::create_dir_all(&shared_dir).unwrap();
        std::fs::write(archive_dir.join("libz.a"), []).unwrap();
        let shared = shared_dir.join("libz.so");
        std::fs::write(&shared, []).unwrap();
        let paths = [archive_dir, shared_dir];
        assert_eq!(
            package_shared_library("x86_64-unknown-linux-gnu", &paths, "z"),
            Some(shared)
        );
        assert_eq!(
            package_shared_library("x86_64-unknown-linux-gnu", &paths, "missing"),
            None
        );
    }

    #[test]
    fn msvc_links_package_import_libraries_without_gnu_flags() {
        let dir = tempfile::tempdir().unwrap();
        let lib = dir.path().join("lib");
        std::fs::create_dir_all(&lib).unwrap();
        let import = lib.join("cmocka.lib");
        std::fs::write(&import, []).unwrap();
        let options = package_link_options(
            "x86_64-pc-windows-msvc",
            &[lib],
            &["cmocka".into(), "ws2_32".into()],
        );
        // Found import libraries link by path; others are system libraries.
        assert_eq!(
            options,
            [import.display().to_string(), "ws2_32.lib".into()]
        );
    }

    #[test]
    fn gnu_and_apple_keep_search_and_runtime_paths() {
        let dir = tempfile::tempdir().unwrap();
        let shared = dir.path().join("libcmocka.dylib");
        std::fs::write(&shared, []).unwrap();
        let paths = [dir.path().to_path_buf()];
        let options = package_link_options("aarch64-apple-darwin", &paths, &["cmocka".into()]);
        let directory = dir.path().display();
        assert_eq!(
            options,
            [
                format!("-L{directory}"),
                format!("-Wl,-rpath,{directory}"),
                shared.display().to_string(),
            ]
        );
        let options = package_link_options("x86_64-unknown-linux-gnu", &paths, &["m".into()]);
        assert_eq!(options.last().unwrap(), "-lm");
    }
}
