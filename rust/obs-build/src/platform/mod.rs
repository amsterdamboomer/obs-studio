//! Native target naming and linker conventions. Decisions use the requested
//! target triple, so a cross build never silently uses the build host's ABI.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

mod libobs_linux;

pub fn configure_libobs(
    target: &str,
    root: &Path,
    options: &crate::libobs::Options,
    sets: &std::collections::BTreeMap<String, Vec<String>>,
    library: &mut crate::SharedLibrary,
) -> Result<(), String> {
    if target.contains("linux") {
        libobs_linux::configure(target, root, options, sets, library)
    } else {
        Err(format!(
            "real libobs native graph for {target} is not implemented yet"
        ))
    }
}

pub fn shared_filename(target: &str, name: &str) -> String {
    if target.contains("windows") {
        format!("{name}.dll")
    } else if target.contains("apple") {
        format!("lib{name}.dylib")
    } else {
        format!("lib{name}.so")
    }
}

pub fn configure_shared_link(
    target: &str,
    command: &mut Command,
    artifact: &Path,
    objects: &[PathBuf],
    dependencies: &[PathBuf],
    msvc: bool,
) -> Result<(), String> {
    if target.contains("windows") {
        if msvc {
            command.arg("/LD").args(objects);
            command.args(dependencies.iter().map(|p| p.with_extension("lib")));
            command
                .arg("/link")
                .arg(format!("/OUT:{}", artifact.display()))
                .arg(format!(
                    "/IMPLIB:{}",
                    artifact.with_extension("lib").display()
                ));
        } else {
            command.arg("-shared").args(objects);
            command.args(dependencies.iter().map(|p| p.with_extension("dll.a")));
            command.arg("-o").arg(artifact).arg(format!(
                "-Wl,--out-implib,{}",
                artifact.with_extension("dll.a").display()
            ));
        }
    } else {
        command.arg(if target.contains("apple") {
            "-dynamiclib"
        } else {
            "-shared"
        });
        command
            .args(objects)
            .args(dependencies)
            .arg("-o")
            .arg(artifact);
        let filename = artifact
            .file_name()
            .ok_or("shared library has no filename")?
            .to_string_lossy();
        if target.contains("apple") {
            command.arg(format!("-Wl,-install_name,@rpath/{filename}"));
            command.arg("-Wl,-rpath,@loader_path");
        } else {
            command.arg(format!("-Wl,-soname,{filename}"));
            command.arg("-Wl,--no-undefined");
            command.arg("-Wl,-rpath,$ORIGIN");
        }
        for dependency in dependencies {
            let parent = dependency
                .parent()
                .ok_or("dependency has no parent directory")?;
            command.arg(format!("-Wl,-rpath,{}", parent.display()));
        }
    }
    Ok(())
}

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

pub fn package_shared_library(target: &str, paths: &[PathBuf], name: &str) -> Option<PathBuf> {
    let filename = package_shared_library_name(target, name)?;
    paths
        .iter()
        .map(|path| path.join(&filename))
        .find(|path| path.is_file())
}

pub fn configure_libobs_variant(
    build: &crate::NativeBuild,
    root: &Path,
    options: &crate::libobs::Options,
    sets: &std::collections::BTreeMap<String, Vec<String>>,
    library: &mut crate::SharedLibrary,
    rust_archive: Option<&Path>,
) -> Result<(), String> {
    if build.target.contains("linux") {
        libobs_linux::configure_variant(
            root,
            options,
            sets,
            library,
            rust_archive,
            &build.output.join("config"),
        )
    } else {
        Err(format!(
            "real libobs variant for {} is not implemented yet",
            build.target
        ))
    }
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
}
