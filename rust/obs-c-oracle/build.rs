use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    // No canonicalize(): on Windows it yields a verbatim `\\?\` path, and
    // MSVC cannot resolve `#include "util/..."` against such an include dir.
    // CARGO_MANIFEST_DIR is already absolute.
    let libobs = manifest
        .ancestors()
        .nth(2)
        .expect("rust/obs-c-oracle has a repo root two levels up")
        .join("libobs");

    let mut oracle = cc::Build::new();
    // Rust never fuses `a * b + c` into an FMA, but GCC/Clang may (clang's
    // default -ffp-contract=on does so on aarch64), which would make float
    // oracles such as vec2_norm (`x*x + y*y`) differ in the last bit from the
    // Rust port. Pin the C oracle to unfused IEEE operations. MSVC does not
    // contract under its default /fp:precise.
    if !oracle.get_compiler().is_like_msvc() {
        oracle.flag("-ffp-contract=off");
    }
    oracle
        .file("oracle/bitstream.c")
        .file("oracle/path_extension.c")
        .file("oracle/array_serializer.c")
        .file("oracle/darray.c")
        .file("oracle/crc32.c")
        .file("oracle/vec2.c")
        .file("oracle/utf8.c")
        .file("oracle/lexer.c")
        .file("oracle/text_lookup.c")
        .file("oracle/dstr.c")
        .file("oracle/dstr_libc.c")
        .file("oracle/bmem.c")
        .file("oracle/test_stubs.c")
        .include(&libobs)
        .std("c11")
        .compile("obs_c_oracle");

    println!("cargo:rerun-if-changed=oracle");
    for header in [
        "util/bitstream.c",
        "util/bitstream.h",
        "util/path-extension.c",
        "util/array-serializer.c",
        "util/array-serializer.h",
        "util/darray.h",
        "util/serializer.h",
        "util/bmem.c",
        "util/bmem.h",
        "util/utf8.c",
        "util/utf8.h",
        "util/lexer.c",
        "util/lexer.h",
        "util/text-lookup.c",
        "util/text-lookup.h",
        "util/dstr.c",
        "util/dstr-libc.c",
        "util/dstr.h",
        "util/platform.h",
        "util/base.h",
        "util/crc32.c",
        "util/crc32.h",
        "graphics/vec2.c",
        "graphics/vec2.h",
        "graphics/math-defs.h",
        "graphics/math-extra.h",
    ] {
        println!("cargo:rerun-if-changed={}", libobs.join(header).display());
    }
}
