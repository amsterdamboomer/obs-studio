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
            command.args(dependencies.iter().map(|p| dependency_link_name(p, true)));
            command
                .arg("/link")
                .arg(format!("/OUT:{}", artifact.display()))
                .arg(format!(
                    "/IMPLIB:{}",
                    artifact.with_extension("lib").display()
                ));
        } else {
            command.arg("-shared").args(objects);
            command.args(dependencies.iter().map(|p| dependency_link_name(p, false)));
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

pub fn configure_module(target: &str, library: &mut crate::SharedLibrary) -> Result<(), String> {
    if target.contains("linux") {
        library.file_name = Some(format!("{}.so", library.name));
        library.compile_options.extend(
            ["-std=gnu17", "-fno-strict-aliasing", "-fvisibility=hidden"].map(String::from),
        );
        library
            .link_options
            .extend(["-lm", "-pthread"].map(String::from));
        Ok(())
    } else {
        Err(format!(
            "native OBS module graph for {target} is not implemented yet"
        ))
    }
}

pub fn configure_opengl(
    target: &str,
    root: &Path,
    options: &crate::libobs::Options,
    library: &mut crate::SharedLibrary,
) -> Result<(), String> {
    if !target.contains("linux") {
        return Err(format!(
            "native OpenGL graph for {target} is not implemented yet"
        ));
    }
    configure_module(target, library)?;
    library.file_name = Some("libobs-opengl.so.33".into());
    library
        .sources
        .push(root.join("libobs-opengl/build/opengl.nix+.h.c"));
    for package in ["simde", "libdrm", "gl", "egl", "x11", "x11-xcb", "xcb"] {
        crate::libobs::add_package(library, target, package)?;
    }
    if options.wayland {
        library
            .sources
            .push(root.join("libobs-opengl/build/opengl.nix-wayland+.h.c"));
        for package in ["wayland-egl", "wayland-client"] {
            crate::libobs::add_package(library, target, package)?;
        }
    }
    library.link_options.push("-ldl".into());
    Ok(())
}

pub fn configure_glad(
    target: &str,
    root: &Path,
    library: &mut crate::SharedLibrary,
) -> Result<(), String> {
    if target.contains("linux") {
        library.sources.push(root.join("deps/glad/src/glad_egl.c"));
        Ok(())
    } else {
        Err(format!(
            "native GL loader graph for {target} is not implemented yet"
        ))
    }
}

fn dependency_link_name(path: &Path, msvc: bool) -> PathBuf {
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("dll"))
    {
        path.with_extension(if msvc { "lib" } else { "dll.a" })
    } else {
        // Static archives already name linker inputs; they are not DLLs.
        path.into()
    }
}

pub fn executable_filename(target: &str, name: &str) -> String {
    if target.contains("windows") {
        format!("{name}.exe")
    } else {
        name.into()
    }
}

pub fn configure_executable_link(
    target: &str,
    command: &mut Command,
    artifact: &Path,
    objects: &[PathBuf],
    dependencies: &[PathBuf],
    msvc: bool,
) -> Result<(), String> {
    command.args(objects);
    if target.contains("windows") {
        command.args(
            dependencies
                .iter()
                .map(|path| dependency_link_name(path, msvc)),
        );
        if msvc {
            command
                .arg("/link")
                .arg(format!("/OUT:{}", artifact.display()));
        } else {
            command.arg("-o").arg(artifact);
        }
    } else {
        command.args(dependencies).arg("-o").arg(artifact);
        command.arg(if target.contains("apple") {
            "-Wl,-rpath,@executable_path"
        } else {
            "-Wl,-rpath,$ORIGIN"
        });
        for dependency in dependencies {
            let parent = dependency
                .parent()
                .ok_or("dependency has no parent directory")?;
            command.arg(format!("-Wl,-rpath,{}", parent.display()));
        }
    }
    Ok(())
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
