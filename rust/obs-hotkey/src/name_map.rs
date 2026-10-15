//! Safe core for `libobs/obs-hotkey-name-map.c`: key code <-> key name.
//!
//! C keeps a uthash map in `obs->hotkeys`, built once with `pthread_once`,
//! and scans the key list when there is no core. Both find the first key
//! with the name, so this keeps one lazily built map for every caller.

use std::ffi::CStr;

include!(concat!(env!("OUT_DIR"), "/keys.rs"));

/// `OBS_KEY_NONE`.
pub const KEY_NONE: i32 = 0;

/// `OBS_KEY_LAST_VALUE`: the number of keys.
pub fn key_count() -> usize {
    KEY_NAMES.len()
}

/// `obs_key_to_name`: the name of `key`, or `None` when it is not a key
/// (C logs and returns `""` then).
pub fn key_name(key: i32) -> Option<&'static CStr> {
    let _ = key;
    todo!("hotkey name map port (#833)")
}

/// `obs_key_from_name`: the first key named exactly `name`, or
/// `OBS_KEY_NONE`.
pub fn key_from_name(name: &CStr) -> i32 {
    let _ = name;
    todo!("hotkey name map port (#833)")
}
