fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "bundled")]
    {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let build = obs_build::NativeBuild {
            host: std::env::var("HOST").unwrap(),
            target: std::env::var("TARGET").unwrap(),
            output: std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("native"),
        };
        let library = obs_build::libobs::build(&build, &root)
            .unwrap_or_else(|error| panic!("build bundled libobs: {error}"));
        println!(
            "cargo:rustc-link-search=native={}",
            library.parent().unwrap().display()
        );
        println!("cargo:rustc-link-lib=dylib=obs");
        println!("cargo:rustc-env=OBS_SYS_LIBRARY={}", library.display());
        println!("cargo:library={}", library.display());
    }
}
