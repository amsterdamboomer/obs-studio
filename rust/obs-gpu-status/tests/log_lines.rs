//! The dialog quotes the log lines that explain the GPU state
//! (obs-rust/obs-studio#944).

use obs_gpu_status::report::Report;
use obs_gpu_status::*;

#[test]
fn graphics_lines_are_kept_and_others_dropped() {
    let mut log = LogCapture::default();
    for line in [
        "Loading up OpenGL on adapter Mesa llvmpipe (LLVM 21.1.8, 256 bits)",
        "Unable to open DRM render node.",
        "Audio monitoring device:",
        "eglInitialize failed: EGL_NOT_INITIALIZED",
        "Loading up Direct3D 11 on adapter Microsoft Basic Render Driver (0)",
        "[obs-browser]: Version 2.24.6",
        "Failed to initialize video:  Graphics module not found",
    ] {
        log.push(line);
    }
    assert_eq!(
        log.lines(),
        vec![
            "Loading up OpenGL on adapter Mesa llvmpipe (LLVM 21.1.8, 256 bits)",
            "Unable to open DRM render node.",
            "eglInitialize failed: EGL_NOT_INITIALIZED",
            "Loading up Direct3D 11 on adapter Microsoft Basic Render Driver (0)",
            "Failed to initialize video:  Graphics module not found",
        ]
    );
}

#[test]
fn capture_is_bounded_deduplicated_and_truncated() {
    let mut log = LogCapture::default();
    for _ in 0..3 {
        log.push("Unable to open DRM render node.");
    }
    assert_eq!(log.lines().len(), 1);
    for i in 0..20 {
        log.push(&format!("eglCreateContext failed {i}"));
    }
    assert_eq!(log.lines().len(), MAX_LOG_LINES);
    let mut long = LogCapture::default();
    long.push(&format!("EGL error {}", "x".repeat(1000)));
    assert_eq!(long.lines()[0].chars().count(), MAX_LOG_LINE_CHARS);
}

#[test]
fn report_carries_the_lines_into_json() {
    let report = Report::new(Some("Mesa llvmpipe (LLVM 21.1.8, 256 bits)"), vec![], None)
        .with_log_lines(vec!["Unable to open DRM render node.".into()]);
    assert_eq!(report.log_lines, vec!["Unable to open DRM render node."]);
    let j = report.to_json();
    assert!(
        j.contains("\"logLines\":[\"Unable to open DRM render node.\"]"),
        "{j}"
    );
    let empty = Report::new(Some("NVIDIA GeForce RTX 3060"), vec![], None).to_json();
    assert!(empty.contains("\"logLines\":[]"), "{empty}");
}
