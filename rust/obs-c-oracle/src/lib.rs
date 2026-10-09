//! Test-only: the original libobs C sources compiled with `oracle_`-prefixed
//! symbols, used as the reference in Tier 2 layout tests and Tier 3
//! differential tests (`docs/rust-port/testing-policy.md`). Delete each
//! oracle together with the C source it wraps.
//!
//! Also provides test-only stand-ins for libobs base/platform symbols
//! (`bcrash`, `os_fopen`, `os_fread_utf8`, ..., from `oracle/test_stubs.c`),
//! so any test binary using the oracle resolves them.

pub mod bmem;
pub mod dstr;
pub mod lexer;
pub mod text_lookup;
pub mod utf8;

pub mod path_extension {
    use core::ffi::c_char;

    unsafe extern "C" {
        pub fn oracle_os_get_path_extension(path: *const c_char) -> *const c_char;
    }
}

pub mod crc32 {
    unsafe extern "C" {
        pub fn oracle_calc_crc32(crc: u32, buf: *const core::ffi::c_void, size: usize) -> u32;
    }
}

pub mod vec2 {
    use core::ffi::c_int;

    /// Independent declaration of `struct vec2`. Intentionally not shared
    /// with `obs-graphics`.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    pub struct OracleVec2 {
        pub x: f32,
        pub y: f32,
    }

    unsafe extern "C" {
        pub fn oracle_vec2_abs(dst: *mut OracleVec2, v: *const OracleVec2);
        pub fn oracle_vec2_floor(dst: *mut OracleVec2, v: *const OracleVec2);
        pub fn oracle_vec2_ceil(dst: *mut OracleVec2, v: *const OracleVec2);
        pub fn oracle_vec2_close(
            v1: *const OracleVec2,
            v2: *const OracleVec2,
            epsilon: f32,
        ) -> c_int;
        pub fn oracle_vec2_norm(dst: *mut OracleVec2, v: *const OracleVec2);

        pub fn oracle_vec2_size() -> usize;
        pub fn oracle_vec2_align() -> usize;
        pub fn oracle_vec2_offset_x() -> usize;
        pub fn oracle_vec2_offset_y() -> usize;
        pub fn oracle_vec2_offset_ptr() -> usize;
    }
}

pub mod darray {
    use core::ffi::c_void;

    /// Independent declaration of `struct darray`. Intentionally not shared
    /// with `obs-util`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct OracleDarray {
        pub array: *mut c_void,
        pub num: usize,
        pub capacity: usize,
    }

    unsafe extern "C" {
        pub fn oracle_darray_free(da: *mut OracleDarray);
        pub fn oracle_darray_reserve(es: usize, da: *mut OracleDarray, capacity: usize);
        pub fn oracle_darray_ensure_capacity(es: usize, da: *mut OracleDarray, new_size: usize);
        pub fn oracle_darray_resize(es: usize, da: *mut OracleDarray, size: usize);
        pub fn oracle_darray_clear(da: *mut OracleDarray);
        pub fn oracle_darray_push_back_array(
            es: usize,
            da: *mut OracleDarray,
            array: *const c_void,
            num: usize,
        ) -> usize;
        pub fn oracle_darray_erase(es: usize, da: *mut OracleDarray, idx: usize);
        pub fn oracle_darray_pop_back(es: usize, da: *mut OracleDarray);

        pub fn oracle_darray_size() -> usize;
        pub fn oracle_darray_align() -> usize;
        pub fn oracle_darray_offset_array() -> usize;
        pub fn oracle_darray_offset_num() -> usize;
        pub fn oracle_darray_offset_capacity() -> usize;
    }
}

pub mod array_serializer {
    use core::ffi::{c_int, c_void};

    /// Independent declaration of `struct serializer`.
    #[repr(C)]
    pub struct OracleSerializer {
        pub data: *mut c_void,
        pub read: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> usize>,
        pub write: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> usize>,
        pub seek: Option<unsafe extern "C" fn(*mut c_void, i64, c_int) -> i64>,
        pub get_pos: Option<unsafe extern "C" fn(*mut c_void) -> i64>,
    }

    /// Independent declaration of `struct array_output_data`.
    #[repr(C)]
    pub struct OracleArrayOutputData {
        pub bytes: super::darray::OracleDarray,
        pub cur_pos: usize,
    }

    unsafe extern "C" {
        pub fn oracle_array_output_serializer_init(
            s: *mut OracleSerializer,
            data: *mut OracleArrayOutputData,
        );
        pub fn oracle_array_output_serializer_free(data: *mut OracleArrayOutputData);
        pub fn oracle_array_output_serializer_reset(data: *mut OracleArrayOutputData);

        pub fn oracle_serializer_size() -> usize;
        pub fn oracle_serializer_align() -> usize;
        pub fn oracle_serializer_offset_data() -> usize;
        pub fn oracle_serializer_offset_read() -> usize;
        pub fn oracle_serializer_offset_write() -> usize;
        pub fn oracle_serializer_offset_seek() -> usize;
        pub fn oracle_serializer_offset_get_pos() -> usize;

        pub fn oracle_array_output_data_size() -> usize;
        pub fn oracle_array_output_data_align() -> usize;
        pub fn oracle_array_output_data_offset_bytes() -> usize;
        pub fn oracle_array_output_data_offset_cur_pos() -> usize;
    }
}

pub mod bitstream {
    use core::ffi::c_int;

    /// Independent declaration of `struct bitstream_reader` for calling the
    /// oracle. Intentionally not shared with `obs-util`.
    #[repr(C)]
    #[allow(non_snake_case)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OracleReader {
        pub pos: u8,
        pub subPos: u8,
        pub buf: *mut u8,
        pub len: usize,
    }

    unsafe extern "C" {
        pub fn oracle_bitstream_reader_init(r: *mut OracleReader, data: *mut u8, len: usize);
        pub fn oracle_bitstream_reader_read_bits(r: *mut OracleReader, bits: c_int) -> u8;
        pub fn oracle_bitstream_reader_r8(r: *mut OracleReader) -> u8;
        pub fn oracle_bitstream_reader_r16(r: *mut OracleReader) -> u16;

        pub fn oracle_bitstream_reader_size() -> usize;
        pub fn oracle_bitstream_reader_align() -> usize;
        pub fn oracle_bitstream_reader_offset_pos() -> usize;
        pub fn oracle_bitstream_reader_offset_subPos() -> usize;
        pub fn oracle_bitstream_reader_offset_buf() -> usize;
        pub fn oracle_bitstream_reader_offset_len() -> usize;
    }
}
