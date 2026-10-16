#!/bin/bash
# Cross-compile check (cmake/cross/) in a container, so the host needs only
# Docker: no soldr, LLVM or SDK. Runs through ci/docker-task.sh (the stand-in
# for zackees/bosn#666): an ephemeral container per checkout, one at a time.
#
#   cmake/cross/docker.sh [--clean|--clean-all] [--wait] <alias>...
#   e.g. cmake/cross/docker.sh mac-arm64 win-x64
#
# Code and caches:
#   /src           the checkout, copy-on-write overlay: read-only underneath,
#                  writes vanish after the run
#   /build         per checkout (obs-cross-build-<id>): one CMake tree per
#                  alias, and the soldr CMake downloads
#   /root/.soldr   shared (obs-cross-soldr): soldr's LLVM, Apple SDK, xwin,
#                  Rust toolchain and compiler cache
#   /src/.deps     shared (obs-cross-deps): OBS's pinned prebuilt
#                  Windows/macOS dependencies
# The first run downloads soldr, LLVM, the SDK and the deps; later runs
# reattach the caches and only recompile what changed.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)

task=()
aliases=()
for arg in "$@"; do
  case $arg in
  --clean | --clean-all | --wait) task+=("$arg") ;;
  -*) echo "usage: $0 [--clean|--clean-all] [--wait] <alias>..." >&2; exit 2 ;;
  *) aliases+=("$arg") ;;
  esac
done
[ "${#aliases[@]}" -gt 0 ] || { echo "usage: $0 [--clean|--clean-all] [--wait] <alias>... (win-x64, mac-arm64, mac-x64)" >&2; exit 2; }

# One container builds every requested alias in turn.
script='set -e; cd /src
for alias in "$@"; do
  echo "== cross $alias"
  cmake -S cmake/cross -B "/build/$alias" -G Ninja -DOBS_CROSS_TARGET="$alias"
  cmake --build "/build/$alias" -- -k 0
done'

# The image only needs cmake, ninja, git and curl: the Tier 2 one has them.
rust_toolchain=$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$repo/rust-toolchain.toml")
exec bash "$repo/ci/docker-task.sh" run --name obs-cross --repo "$repo" \
  --image obs-rust-linux-validate --dockerfile "$repo/rust/tools/linux-validate/Dockerfile" \
  --context "$repo/rust/tools/linux-validate" --build-arg "RUST_TOOLCHAIN=$rust_toolchain" \
  --source overlay --source-dest /src \
  --stack-volume build:/build \
  --machine-volume soldr:/root/.soldr \
  --machine-volume deps:/src/.deps \
  ${task[@]+"${task[@]}"} -- bash -c "$script" cross "${aliases[@]}"
