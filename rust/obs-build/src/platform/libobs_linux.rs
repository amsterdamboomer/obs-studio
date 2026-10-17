use crate::{
    SharedLibrary,
    libobs::{Options, add_package, append_sources},
};
use std::{collections::BTreeMap, path::Path};

pub fn configure(
    target: &str,
    root: &Path,
    options: &Options,
    sets: &BTreeMap<String, Vec<String>>,
    library: &mut SharedLibrary,
) -> Result<(), String> {
    append_sources(sets, "LIBOBS_LINUX_SOURCES", root, library)?;
    append_sources(sets, "LIBOBS_LINUX_PORTED_SOURCES", root, library)?;
    library.file_name = Some("libobs.so.30".into());
    library
        .definitions
        .push(("OBS_INSTALL_PREFIX".into(), Some("\"/usr/local\"".into())));
    library
        .definitions
        .push(("SIMDE_ENABLE_OPENMP".into(), None));
    library.compile_options.extend(
        [
            "-std=gnu17",
            "-fno-strict-aliasing",
            "-fvisibility=hidden",
            "-fopenmp-simd",
        ]
        .map(String::from),
    );
    library
        .link_options
        .extend(["-pthread", "-ldl", "-lm"].map(String::from));
    for package in ["x11", "x11-xcb", "xcb", "uuid"] {
        add_package(library, target, package)?;
    }
    if options.pulseaudio {
        add_package(library, target, "libpulse")?;
        library
            .sources
            .push(root.join("libobs/build/libobs.pulse+.h.c"));
    } else {
        library
            .sources
            .push(root.join("libobs/build/libobs.null-audio+.h.c"));
    }
    if options.gio {
        add_package(library, target, "gio-2.0")?;
        library
            .sources
            .push(root.join("libobs/build/libobs.nix-dbus+.h.c"));
    }
    if options.wayland {
        for package in ["wayland-client", "xkbcommon"] {
            add_package(library, target, package)?;
        }
        library
            .sources
            .push(root.join("libobs/build/libobs.nix-wayland+.h.c"));
        if options.gio {
            library
                .sources
                .push(root.join("libobs/build/libobs.nix-portal-shortcuts+.h.c"));
            library
                .definitions
                .push(("HAVE_GLOBAL_SHORTCUTS_PORTAL".into(), None));
        }
    }
    Ok(())
}
