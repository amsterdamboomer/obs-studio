#!/usr/bin/env bash
# The rest of what the Ubuntu 26.04 AppImage build took from apt and AlmaLinux 8
# lacks or ships too old (obs-rust/obs-studio#926), into $PREFIX:
#   pipewire   client library, modules, SPA plugins and client.conf
#              (AlmaLinux 8 has 0.3.6; OBS needs >= 0.3.33)
#   fdk-aac    the obs-libfdk encoder
#   qrcodegen  obs-websocket's connect dialog
#   json asio simde   header-only
#   misc.sh [name...]   (default: all, in order)
set -euo pipefail

PREFIX="${PREFIX:-/opt/obs-deps}"
# CMake 4 refuses projects (svt-av1's bundled cpuinfo, ...) that still declare
# cmake_minimum_required < 3.5; let them configure with 3.5 policies.
export CMAKE_POLICY_VERSION_MINIMUM=3.5
JOBS="$(nproc)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

git_src() {
    git init -q "$work/$1"
    git -C "$work/$1" fetch -q --depth 1 "$2" "$3"
    git -C "$work/$1" checkout -q FETCH_HEAD
    cd "$work/$1"
}

pipewire() {
    # 1.2.7
    git_src pipewire https://gitlab.freedesktop.org/pipewire/pipewire.git cc7439187f61dd73b81ca69f5dbccbb52ce970b2
    meson setup build --prefix="$PREFIX" --libdir=lib --buildtype=release \
        -Dsession-managers=[] -Dexamples=disabled -Dtests=disabled -Dman=disabled -Ddocs=disabled \
        -Dsystemd=disabled -Dsystemd-user-service=disabled -Dpipewire-alsa=disabled -Dpipewire-jack=disabled \
        -Dpipewire-v4l2=disabled -Djack=disabled -Djack-devel=false -Dbluez5=disabled -Dlibcamera=disabled \
        -Dgstreamer=disabled -Dgstreamer-device-provider=disabled -Dvulkan=disabled -Dlibpulse=disabled \
        -Droc=disabled -Davahi=disabled -Decho-cancel-webrtc=disabled -Dlibusb=disabled -Dlibmysofa=disabled \
        -Dsdl2=disabled -Dsndfile=disabled -Dlibffado=disabled -Dflatpak=disabled -Dselinux=disabled \
        -Dlv2=disabled -Draop=disabled -Dx11=disabled -Dx11-xfixes=disabled -Dlibcanberra=disabled \
        -Dreadline=disabled -Dopus=disabled -Dcompress-offload=disabled -Dsnap=disabled \
        -Dudev=disabled -Dalsa=disabled -Dpw-cat=disabled -Dvideotestsrc=disabled -Daudiotestsrc=disabled \
        -Ddbus=enabled
    ninja -C build && ninja -C build install
}
fdkaac() {
    # v2.0.3
    git_src fdkaac https://github.com/mstorsjo/fdk-aac.git 716f4394641d53f0d79c9ddac3fa93b03a49f278
    cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX="$PREFIX" \
        -DCMAKE_INSTALL_LIBDIR=lib -DBUILD_SHARED_LIBS=ON -DBUILD_PROGRAMS=OFF
    cmake --build build && cmake --install build
}
qrcodegen() {
    # nayuki QR-Code-generator v1.8.0, built the way Debian's libqrcodegencpp is
    git_src qrcodegen https://github.com/nayuki/QR-Code-generator.git 720f62bddb7226106071d4728c292cb1df519ceb
    cd cpp
    g++ -O2 -fPIC -shared -Wl,-soname,libqrcodegencpp.so.1 -o libqrcodegencpp.so.1 qrcodegen.cpp
    install -Dm755 libqrcodegencpp.so.1 "$PREFIX/lib/libqrcodegencpp.so.1"
    ln -sf libqrcodegencpp.so.1 "$PREFIX/lib/libqrcodegencpp.so"
    install -Dm644 qrcodegen.hpp "$PREFIX/include/qrcodegen/qrcodegen.hpp"
    mkdir -p "$PREFIX/lib/pkgconfig"
    cat >"$PREFIX/lib/pkgconfig/qrcodegencpp.pc" <<EOF
prefix=$PREFIX
libdir=\${prefix}/lib
includedir=\${prefix}/include/qrcodegen
Name: qrcodegencpp
Description: QR Code generator library (C++)
Version: 1.8.0
Libs: -L\${libdir} -lqrcodegencpp
Cflags: -I\${includedir}
EOF
}
json() {
    # nlohmann/json v3.11.3
    git_src json https://github.com/nlohmann/json.git 9cca280a4d0ccf0c08f47a99aa71d1b0e52f8d03
    cmake -S . -B build -G Ninja -DCMAKE_INSTALL_PREFIX="$PREFIX" -DJSON_BuildTests=OFF -DJSON_Install=ON
    cmake --install build
}
asio() {
    # asio-1-30-2
    git_src asio https://github.com/chriskohlhoff/asio.git 12e0ce9e0500bf0f247dbd1ae894272656456079
    cp -r asio/include/asio asio/include/asio.hpp "$PREFIX/include/"
}
simde() {
    # v0.8.2
    git_src simde https://github.com/simd-everywhere/simde.git 71fd833d9666141edcd1d3c109a80e228303d8d7
    mkdir -p "$PREFIX/include" && cp -r simde "$PREFIX/include/"
    mkdir -p "$PREFIX/lib/pkgconfig"
    printf 'prefix=%s\nincludedir=${prefix}/include\nName: simde\nDescription: SIMD Everywhere\nVersion: 0.8.2\nCflags: -I${includedir}\n' \
        "$PREFIX" >"$PREFIX/lib/pkgconfig/simde.pc"
}

ALL=(pipewire fdkaac qrcodegen json asio simde)
for dep in "${@:-${ALL[@]}}"; do
    echo "==> $dep"
    (cd "$work" && "$dep")
done
