//! Clang combines sinf/cosf pairs into this system libm entry point.
//! Apple's paired implementation can differ from separate sinf/cosf calls
//! by one ULP, including for large arguments. Match the shipped C operation.

#[repr(C)]
struct SinCos {
    sin: f32,
    cos: f32,
}

unsafe extern "C" {
    fn __sincosf_stret(x: f32) -> SinCos;
}

pub(crate) fn sin_cos(x: f32) -> (f32, f32) {
    // SAFETY: system libm accepts every f32 and returns two floats by value;
    // repr(C) preserves Apple's homogeneous floating-point aggregate ABI.
    let result = unsafe { __sincosf_stret(x) };
    (result.sin, result.cos)
}
