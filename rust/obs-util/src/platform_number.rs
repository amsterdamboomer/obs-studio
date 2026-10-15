//! Safe core for `os_strtod` and `os_dtostr` (`libobs/util/platform-number.c`):
//! locale-independent conversion between `double` and its text form.

/// `os_strtod`: the value of the longest valid prefix of `s`, as `strtod`
/// reads it with `.` as the decimal point; 0 when nothing converts.
pub fn strtod(s: &[u8]) -> f64 {
    let _ = s;
    todo!("os_strtod port (#262)")
}

/// `os_dtostr`: `value` as text that reads back to the same `double`, or
/// `None` when it does not fit a buffer of `size` bytes (NUL included).
pub fn dtostr(value: f64, size: usize) -> Option<Vec<u8>> {
    let _ = (value, size);
    todo!("os_dtostr port (#262)")
}
