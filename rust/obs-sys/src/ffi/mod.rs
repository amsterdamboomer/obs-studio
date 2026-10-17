//! Declarations from the unchanged libobs public headers.
unsafe extern "C" {
    /// Returns the libobs C API version from `libobs/obs-config.h`.
    pub fn obs_get_version() -> u32;
}
