#[test]
fn bundled_libobs_reports_the_public_c_api_version() {
    // SAFETY: the bundled build links the original no-argument C API function.
    let version = unsafe { obs_sys::ffi::obs_get_version() };
    assert_eq!(version >> 24, 33);
    assert!(std::path::Path::new(env!("OBS_SYS_LIBRARY")).is_file());
}
