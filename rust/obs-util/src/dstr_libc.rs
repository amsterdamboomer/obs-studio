//! Safe core of `libobs/util/dstr-libc.c`: the wide-string compares, search
//! and depad, and the case mapping `dstr_to_upper`/`dstr_to_lower` apply.
//!
//! Strings are slices of the host `wchar_t` ([`WChar`]) that end at their
//! first 0. A slice without a 0 reads as if one followed its last element,
//! so no input can index out of bounds. Comparisons are on `wchar_t` values,
//! signed where the host's `wchar_t` is, as in C. Case mapping is the C
//! library's `towupper`/`towlower` in the current locale (through
//! [`crate::platform::crt`]), so it matches libobs in any locale.
//!
//! The C variadic `dstr_printf`/`dstr_catf`/`dstr_vprintf`/`dstr_vcatf`
//! stay C (`util/dstr-variadic.c`); the conversions to and from multibyte
//! and wide strings are in the C ABI shim ([`crate::ffi::dstr_libc`]), over
//! `util/platform-conversions.c` and the ported `utf8`.

pub use crate::platform::WChar;

/// Length up to the first 0 (`wcslen`), or the slice length if it has none.
pub fn wcslen(s: &[WChar]) -> usize {
    todo!("dstr-libc port: wcslen {}", s.len())
}

/// `wstrcmpi`: compare after `towupper`; -1, 0 or 1.
pub fn wstrcmpi(str1: &[WChar], str2: &[WChar]) -> i32 {
    todo!("dstr-libc port: wstrcmpi {} {}", str1.len(), str2.len())
}

/// `wstrcmp_n`: compare at most `n` characters; -1, 0 or 1.
pub fn wstrcmp_n(str1: &[WChar], str2: &[WChar], n: usize) -> i32 {
    todo!(
        "dstr-libc port: wstrcmp_n {} {} {n}",
        str1.len(),
        str2.len()
    )
}

/// `wstrcmpi_n`: [`wstrcmp_n`] after `towupper`.
pub fn wstrcmpi_n(str1: &[WChar], str2: &[WChar], n: usize) -> i32 {
    todo!(
        "dstr-libc port: wstrcmpi_n {} {} {n}",
        str1.len(),
        str2.len()
    )
}

/// `wstrstri`: the offset of the first case-insensitive match of `find`
/// in `str`, or `None`.
pub fn wstrstri(str: &[WChar], find: &[WChar]) -> Option<usize> {
    todo!("dstr-libc port: wstrstri {} {}", str.len(), find.len())
}

/// `wcsdepad`: strip leading and trailing space, tab, CR and LF in place.
pub fn wcsdepad(str: &mut [WChar]) {
    todo!("dstr-libc port: wcsdepad {}", str.len())
}

/// The `dstr_to_upper` loop: `towupper` every character before the first 0.
pub fn to_upper(str: &mut [WChar]) {
    todo!("dstr-libc port: to_upper {}", str.len())
}

/// The `dstr_to_lower` loop: `towlower` every character before the first 0.
pub fn to_lower(str: &mut [WChar]) {
    todo!("dstr-libc port: to_lower {}", str.len())
}
