//! Native target naming and linker conventions. Decisions use the requested
//! target triple, so a cross build never silently uses the build host's ABI.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
