//! Safe core of the service capability and connect-info getters in
//! `libobs/obs-service.c.h` (#1899): `obs_service_get_protocol()`,
//! `_get_connect_info()`, `_can_try_to_connect()`,
//! `_get_preferred_output_type()`, `_get_max_fps()`, `_get_max_bitrate()`,
//! `_get_supported_resolutions()` and the supported codec lists.
//!
//! Each getter checks its arguments, writes its out-parameters' defaults and
//! hands the call to the plugin's optional callback in
//! `struct obs_service_info`. [`Callbacks`] is that callback table for one
//! service: a method returns `None` (or `false`) when the plugin left the
//! callback NULL. What the plugin returns is passed through untouched, so
//! the string and list types stay opaque.
//!
//! The order of checks and writes is the C's, including what
//! `test/cmocka/test_service_getters.c` marks "characterized, not
//! endorsed": `max_bitrate` zeroes both out-parameters before it checks the
//! service, while `max_fps` and `supported_resolutions` check first; the
//! codec getters take a service that is never null (the C dereferences it).
//! Null logging goes through [`NullLog`] with the C's function and parameter
//! names, `"obs_service_get_info"` for connect info included.

use core::ffi::CStr;

/// What the plugin's callbacks of one service do. `None` / `false`: the
/// callback is NULL.
pub trait Callbacks {
    /// `const char *`, passed through.
    type Str: Copy;
    /// `const char **`, passed through.
    type List: Copy;
    /// `struct obs_service_resolution *`, written by the plugin.
    type Resolutions;

    /// `get_protocol`, required at registration.
    fn protocol(&self) -> Self::Str;
    fn connect_info(&self, kind: u32) -> Option<Self::Str>;
    fn can_try_to_connect(&self) -> Option<bool>;
    fn output_type(&self) -> Option<Self::Str>;
    /// Calls `get_max_fps`; false when it is NULL.
    fn max_fps(&self, fps: &mut i32) -> bool;
    /// Calls `get_max_bitrate` with the caller's out-parameters, NULL ones
    /// included; false when it is NULL.
    fn max_bitrate(&self, video: Option<&mut i32>, audio: Option<&mut i32>) -> bool;
    /// Calls `get_supported_resolutions`; false when it is NULL.
    fn resolutions(&self, resolutions: &mut Self::Resolutions, count: &mut usize) -> bool;
    fn video_codecs(&self) -> Option<Self::List>;
    fn audio_codecs(&self) -> Option<Self::List>;
}

/// `obs_ptr_valid`'s debug line: `"%s: Null '%s' parameter"`.
pub trait NullLog {
    fn null_param(&self, func: &'static CStr, param: &'static CStr);
}

/// `obs_service_get_protocol`.
pub fn protocol<S: Callbacks>(_service: Option<&S>, _log: &impl NullLog) -> Option<S::Str> {
    todo!("#1899")
}

/// `obs_service_get_connect_info`.
pub fn connect_info<S: Callbacks>(
    _service: Option<&S>,
    _log: &impl NullLog,
    _kind: u32,
) -> Option<S::Str> {
    todo!("#1899")
}

/// `obs_service_can_try_to_connect`.
pub fn can_try_to_connect<S: Callbacks>(_service: Option<&S>, _log: &impl NullLog) -> bool {
    todo!("#1899")
}

/// `obs_service_get_preferred_output_type`.
pub fn preferred_output_type<S: Callbacks>(
    _service: Option<&S>,
    _log: &impl NullLog,
) -> Option<S::Str> {
    todo!("#1899")
}

/// `obs_service_get_max_fps`.
pub fn max_fps<S: Callbacks>(_service: Option<&S>, _log: &impl NullLog, _fps: Option<&mut i32>) {
    todo!("#1899")
}

/// `obs_service_get_max_bitrate`.
pub fn max_bitrate<S: Callbacks>(
    _service: Option<&S>,
    _log: &impl NullLog,
    _video: Option<&mut i32>,
    _audio: Option<&mut i32>,
) {
    todo!("#1899")
}

/// `obs_service_get_supported_resolutions`. `none` is the value written to
/// `resolutions` before the callback (NULL).
pub fn supported_resolutions<S: Callbacks>(
    _service: Option<&S>,
    _log: &impl NullLog,
    _resolutions: Option<&mut S::Resolutions>,
    _count: Option<&mut usize>,
    _none: S::Resolutions,
) {
    todo!("#1899")
}

/// `obs_service_get_supported_video_codecs`.
pub fn video_codecs<S: Callbacks>(_service: &S) -> Option<S::List> {
    todo!("#1899")
}

/// `obs_service_get_supported_audio_codecs`.
pub fn audio_codecs<S: Callbacks>(_service: &S) -> Option<S::List> {
    todo!("#1899")
}
