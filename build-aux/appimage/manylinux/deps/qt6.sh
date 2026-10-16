#!/usr/bin/env bash
# Qt 6 for the glibc 2.28 AppImage base (obs-rust/obs-studio#926): qtbase
# (Core, Gui, Widgets, Network, OpenGL, DBus, Xml, the xcb QPA plugin), qtsvg
# and qtwayland, installed into /opt/obs-deps/qt6.
#
# Not compiled here: it is downloaded prebuilt from obs-rust/obs-studio-libs
# (tool "qt6", recipes/qt6/linux/ there, built on this same manylinux_2_28
# base), pinned by release tag and sha256. The archive is a Qt install prefix
# (bin/, lib/, include/, plugins/, lib/cmake/Qt6*), so it unpacks straight
# into PREFIX. To bump Qt: publish the new release, then update QT_VERSION and
# QT_SHA256 (the sha256 is in the catalog,
# https://raw.githubusercontent.com/obs-rust/obs-studio-libs/assets/qt6/manifest.json).
set -euo pipefail

PREFIX="${QT_PREFIX:-/opt/obs-deps/qt6}"
QT_VERSION=6.11.1
QT_SHA256=f0edfc3432bd57058bbb9a2cf18aacceb55a2564b018346778c34eb0e3369dbf
QT_FILE="qt6-${QT_VERSION}-linux-x86_64-glibc2.28-release.tar.zst"
QT_URL="${QT_URL:-https://github.com/obs-rust/obs-studio-libs/releases/download/qt6-${QT_VERSION}/${QT_FILE}}"

command -v zstd >/dev/null || dnf install -y --setopt=install_weak_deps=False zstd

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
curl -fsSLo "$work/$QT_FILE" "$QT_URL"
if ! echo "${QT_SHA256}  $work/$QT_FILE" | sha256sum -c -; then
    echo "qt6.sh: sha256 mismatch for $QT_URL (expected $QT_SHA256)" >&2
    exit 1
fi
mkdir -p "$PREFIX"
zstd -dc "$work/$QT_FILE" | tar -x -C "$PREFIX"

for f in lib/cmake/Qt6/Qt6Config.cmake plugins/platforms/libqxcb.so lib/libQt6Core.so; do
    [ -e "$PREFIX/$f" ] || { echo "qt6.sh: $f missing from $QT_FILE" >&2; exit 1; }
done
