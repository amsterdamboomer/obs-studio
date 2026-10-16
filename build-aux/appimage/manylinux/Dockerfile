# glibc 2.28 build environment for the AppImage (obs-rust/obs-studio#926).
# manylinux_2_28 is AlmaLinux 8 (glibc 2.28) with gcc-toolset-14 as the
# default compiler. Libraries AlmaLinux 8 ships new enough come from dnf;
# the rest are built from source into /opt/obs-deps by deps/*.sh, one layer
# each so a change rebuilds only that dependency and what follows it.
FROM quay.io/pypa/manylinux_2_28_x86_64@sha256:4131ddd7acb9c6c966af02fe7e71edc51e078c2d41f475966697676430bfa2ae

RUN dnf install -y --setopt=install_weak_deps=False \
        nasm which patchelf file \
        desktop-file-utils squashfs-tools ccache \
        libX11-devel libXext-devel libXfixes-devel libXrender-devel libXcomposite-devel \
        libXinerama-devel libXrandr-devel libXi-devel libXcursor-devel libxshmfence-devel \
        libxcb-devel xcb-util-devel xcb-util-wm-devel xcb-util-image-devel xcb-util-keysyms-devel \
        xcb-util-renderutil-devel xcb-util-cursor-devel libxkbcommon-devel libxkbcommon-x11-devel \
        wayland-devel wayland-protocols-devel \
        mesa-libEGL-devel mesa-libgbm-devel libglvnd-devel libdrm-devel libva-devel \
        vulkan-headers vulkan-loader-devel \
        fontconfig-devel freetype-devel glib2-devel dbus-devel systemd-devel \
        alsa-lib-devel pulseaudio-libs-devel speexdsp-devel libv4l-devel \
        pciutils-devel libuuid-devel jansson-devel libcurl-devel openssl-devel \
        luajit-devel uthash-devel websocketpp-devel \
 && dnf clean all

# ninja and meson from PyPI: AlmaLinux 8's ninja 1.8 cannot build Qt 6.10
# (multiple-output rules need >= 1.10), and its meson 0.58 is older than
# PipeWire 1.2 needs (>= 0.61).
RUN /opt/python/cp312-cp312/bin/pip install --no-cache-dir ninja==1.11.1.4 meson==1.5.2 \
 && ln -sf /opt/python/cp312-cp312/bin/ninja /usr/local/bin/ninja \
 && ln -sf /opt/python/cp312-cp312/bin/meson /usr/local/bin/meson

ENV PREFIX=/opt/obs-deps
ENV PKG_CONFIG_PATH=/opt/obs-deps/lib/pkgconfig:/opt/obs-deps/lib64/pkgconfig:/opt/obs-deps/share/pkgconfig
ENV CMAKE_PREFIX_PATH=/opt/obs-deps
ENV LD_LIBRARY_PATH=/opt/obs-deps/lib:/opt/obs-deps/lib64
ENV PATH=/opt/obs-deps/bin:$PATH


COPY build-aux/appimage/manylinux/deps/codecs.sh /opt/deps/codecs.sh
RUN /opt/deps/codecs.sh ogg vorbis opus vpx svtav1 aom x264 theora lame mbedtls srt librist nvcodec
RUN /opt/deps/codecs.sh ffmpeg
COPY build-aux/appimage/manylinux/deps/misc.sh /opt/deps/misc.sh
RUN /opt/deps/misc.sh

# Qt last: nothing above needs it, so a Qt bump (a new pinned prebuilt) re-runs only this layer.
COPY build-aux/appimage/manylinux/deps/qt6.sh /opt/deps/qt6.sh
RUN /opt/deps/qt6.sh
ENV CMAKE_PREFIX_PATH=/opt/obs-deps/qt6:/opt/obs-deps
ENV PKG_CONFIG_PATH=/opt/obs-deps/qt6/lib/pkgconfig:/opt/obs-deps/lib/pkgconfig:/opt/obs-deps/lib64/pkgconfig:/opt/obs-deps/share/pkgconfig
ENV LD_LIBRARY_PATH=/opt/obs-deps/qt6/lib:/opt/obs-deps/lib:/opt/obs-deps/lib64

# --- what building OBS itself needs ------------------------------------------
RUN dnf install -y --setopt=install_weak_deps=False libXScrnSaver-devel extra-cmake-modules \
 && dnf clean all
# SWIG 4 for the Lua scripting bindings (AlmaLinux 8 has 3.0); build-time only.
RUN /opt/python/cp312-cp312/bin/pip install --no-cache-dir swig==4.2.1 \
 && ln -sf /opt/python/cp312-cp312/bin/swig /usr/local/bin/swig
# uv, pinned by version and checksum (as in ../Dockerfile).
ARG UV_VERSION=0.12.13
ARG UV_SHA256=745765a3b6e360ad76743599ae5c42e9278c7edf8bbff9fc76d05bf2623a04dd
RUN curl -fsSLo /tmp/uv.tar.gz "https://github.com/astral-sh/uv/releases/download/${UV_VERSION}/uv-x86_64-unknown-linux-gnu.tar.gz" \
 && echo "${UV_SHA256}  /tmp/uv.tar.gz" | sha256sum -c - \
 && tar -xzf /tmp/uv.tar.gz -C /usr/local/bin --strip-components=1 \
 && rm /tmp/uv.tar.gz
# The Rust toolchain pinned in rust-toolchain.toml.
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
COPY rust-toolchain.toml /tmp/rust-toolchain.toml
RUN toolchain="$(sed -n 's/^channel = "\(.*\)"/\1/p' /tmp/rust-toolchain.toml)" \
 && curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "$toolchain" \
 && rm /tmp/rust-toolchain.toml
RUN git config --system --add safe.directory '*'
