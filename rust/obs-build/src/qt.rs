//! Explicit Qt code generation for native Cargo targets.
use crate::SharedLibrary;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Generate one moc translation unit using the target's includes and definitions.
/// Cross builds can provide a host executable through OBS_QT_MOC.
pub fn moc(library: &mut SharedLibrary, header: &Path, output: &Path) -> Result<PathBuf, String> {
    println!("cargo:rerun-if-env-changed=OBS_QT_MOC");
    println!("cargo:rerun-if-changed={}", header.display());
    let tool = if let Some(path) = std::env::var_os("OBS_QT_MOC") {
        PathBuf::from(path)
    } else {
        PathBuf::from(pkg_config::get_variable("Qt6Core", "libexecdir").map_err(|e| e.to_string())?)
            .join("moc")
    };
    let header = header.canonicalize().map_err(|e| e.to_string())?;
    let mut command = Command::new(tool);
    for directory in &library.include_dirs {
        command.arg("-I").arg(directory);
    }
    for (name, value) in &library.definitions {
        command.arg(match value {
            Some(value) => format!("-D{name}={value}"),
            None => format!("-D{name}"),
        });
    }
    let result = command
        .arg(&header)
        .output()
        .map_err(|e| format!("execute Qt moc: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "Qt moc {} failed: {}",
            header.display(),
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    if result.stdout.is_empty() {
        return Err(format!("Qt moc produced no code for {}", header.display()));
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if std::fs::read(output).ok().as_deref() != Some(result.stdout.as_slice()) {
        std::fs::write(output, result.stdout).map_err(|e| e.to_string())?;
    }
    library.sources.push(output.into());
    Ok(output.into())
}

pub fn uic(_library: &mut SharedLibrary, _form: &Path, _output: &Path) -> Result<PathBuf, String> {
    Err("Qt UI generation is not implemented".into())
}

pub fn rcc(
    _library: &mut SharedLibrary,
    _resources: &Path,
    _output: &Path,
) -> Result<PathBuf, String> {
    Err("Qt resource generation is not implemented".into())
}
