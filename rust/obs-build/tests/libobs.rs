//! Opt-in integration against real native dependencies and libobs-rust.
use obs_build::{NativeBuild, libobs};
use std::{ffi::c_void, path::PathBuf};

#[test]
fn c_and_rust_twins_preserve_the_public_data_and_lifecycle_apis() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let archive = PathBuf::from(
        std::env::var_os("OBS_BUILD_RUST_ARCHIVE")
            .expect("build libobs-rust in release mode, then set OBS_BUILD_RUST_ARCHIVE"),
    );
    let output = tempfile::tempdir().unwrap();
    let build = NativeBuild {
        host: env!("OBS_BUILD_HOST").into(),
        target: env!("OBS_BUILD_TARGET").into(),
        output: output.path().into(),
    };
    let libraries =
        libobs::build_pair(&build, &root, &libobs::Options::default(), &archive).unwrap();
    assert_ne!(libraries.reference, libraries.with_rust);
    for path in [&libraries.reference, &libraries.with_rust] {
        // SAFETY: these signatures come from the unchanged obs.h/obs-data.h.
        // Symbols are called only while their library is loaded, data handles
        // are released, and each library owns its complete startup/shutdown.
        unsafe {
            let library = libloading::Library::new(path).unwrap();
            let startup = library
                .get::<unsafe extern "C" fn(
                    *const std::ffi::c_char,
                    *const std::ffi::c_char,
                    *mut c_void,
                ) -> bool>(b"obs_startup")
                .unwrap();
            let shutdown = library
                .get::<unsafe extern "C" fn()>(b"obs_shutdown")
                .unwrap();
            let create = library
                .get::<unsafe extern "C" fn() -> *mut c_void>(b"obs_data_create")
                .unwrap();
            let set = library
                .get::<unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char, i64)>(
                    b"obs_data_set_int",
                )
                .unwrap();
            let get = library
                .get::<unsafe extern "C" fn(*mut c_void, *const std::ffi::c_char) -> i64>(
                    b"obs_data_get_int",
                )
                .unwrap();
            let release = library
                .get::<unsafe extern "C" fn(*mut c_void)>(b"obs_data_release")
                .unwrap();
            assert!(startup(
                c"en-US".as_ptr(),
                std::ptr::null(),
                std::ptr::null_mut()
            ));
            let data = create();
            assert!(!data.is_null());
            set(data, c"value".as_ptr(), -9223372036854775807);
            assert_eq!(get(data, c"value".as_ptr()), -9223372036854775807);
            release(data);
            shutdown();
        }
    }
}
