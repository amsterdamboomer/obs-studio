# Cargo-driven OBS native build conversion

Tracking issue: <https://github.com/obs-rust/obs-studio/issues/1332>.

The requested final state is a working OBS Rust application built through Cargo
and `cc::Build`, including shared libraries, plugins, generated files, graphics,
Qt, resources, tests and platform packaging. A core-only archive or test fixture
does not complete the conversion.

## Requirements carried forward

- Native C/C++ builds must be very fast. Reuse explicit unity groups, make unity
  configurable, and implement optional PCH with compiler-compatible settings and
  correct dependency invalidation. The user's instruction that unity and PCH are
  optional supersedes BUILD.md's always-on rule for this conversion.
- Build through soldr. Demonstrate real compiler cache hits, cold/warm timings,
  no-change builds and correctness after editing source, headers and generated
  inputs. A setup-soldr archive restore alone does not prove native cache reuse.
- Preserve the public C ABI, dynamic loading, plugin discovery, library identity,
  exported symbols and the existing Rust implementation. Build both libobs
  variants and exercise the unchanged compatibility and differential suites.
- Keep the application and bundled crates on one native build description.
- Generate files in build output directories; explicitly consume generated
  outputs so stale source-tree files cannot shadow them (PR #1323).
- Preserve the complete feature surface on Linux, Windows and macOS. Verify a
  real running application, scenes, recording, plugins, shutdown and applicable
  websocket tests. Packaging validation must run packaged artifacts.
- Ordinary Cargo users can provide documented native tools; soldr manages the
  toolchain and caching. Do not require a nested soldr invocation inside build.rs.
- Reuse existing CI workflows; no new workflow files are authorized.

## Implementation order and evidence

1. `obs-build`: typed native build inputs, actual cc::Build compilation, shared
   linking, plugin loading and unity on/off contracts. Start with failing tests.
2. `obs-sys`: the real libobs graph, native dependencies, generated configuration
   and Rust twin; unchanged C tests and symbol parity.
3. Frontend/plugins, Qt code generation and resources, platform linking and
   packaging; run the existing application test battery on these artifacts.
4. Optional PCH, cache/incremental performance measurements and existing CI
   integration. Retire CMake only when complete feature/test parity is evidenced.

The existing CMake build remains the artifact and behavior reference during the
conversion. Intermediate progress must not be reported as full completion.

## Linux libobs checkpoint

The bundled `obs-sys` feature now compiles the real C libobs through `cc::Build`,
links `libobs.so.30`, and generates configuration, version and core-module source
files under Cargo's output directory. It uses the same canonical source manifest
as the CMake reference. libcaption remains a separate static archive; system
libraries remain dynamically linked, including when pkg-config lists a static
archive directory before its shared-library directory.

Local evidence for this checkpoint:

- The native build engine's three tests pass (dependency selection, generated
  source/unity toggle, and a dynamically loaded plugin observing a rebuilt core).
- Bundled libobs passes its public C API version and startup/shutdown tests.
- Editing only the JSON manifest triggers CMake regeneration. Changing a
  target-qualified C compiler flag reruns Cargo's bundled native build; compiler
  metadata does not add unintended static-library linkage.
- Its 1,766 exported symbols exactly match the local CMake C reference.
- 51 unchanged, already-built C test executables pass against both libraries.
  `ldd` confirms each executable loads the intended replacement `libobs.so.30`.
  This validates the replacement library; it does not demonstrate a Cargo build
  of those test executables or the Rust implementation twin.

This checkpoint is Linux C libobs only. The application and plugins,
full test battery, macOS/Windows graphs, installation/packaging, optional PCH and
controlled build/cache benchmarks remain required. The generated version and
install paths are prototype defaults and still need release configuration.
Native dependencies are discovered using pkg-config; local Nix verification used
`PKG_CONFIG_PATH` and `CPATH` for installed packages and uthash respectively.


## Linux Rust implementation twin checkpoint

`libobs::build_pair` accepts the release `libobs-rust` static archive from an
outer Cargo invocation. It compiles common C objects once, then links the C
reference and `libobs-with-rust` into separate directories with the same SONAME.
The Rust archive is whole-archive linked and its internal symbols are hidden by
the existing export map. The twin is never installed.

Build the release archive with `soldr cargo build -p libobs-rust --release`.
With native dependencies available, set `OBS_BUILD_RUST_ARCHIVE` to the absolute
`target/release/liblibobs_rust.a` path and run
`soldr cargo test -p obs-build --features libobs-integration --test libobs`.
`OBS_BUILD_TEST_OUTPUT` optionally retains artifacts in a chosen output directory.
The test defaults to a temporary output directory. Set
`OBS_BUILD_DISABLE_HEVC=1` to exercise the disabled-HEVC configuration. GNU `nm`
is required for the Linux export comparison.

Local evidence: both variants pass startup/shutdown and public data API roundtrip
checks, and each exports exactly the same 1,766 symbols (1,762 with HEVC disabled).
The opt-in test compares complete export sets and checks HEVC export availability.
The same 51 unchanged C
executables pass against both newly built variants with their loaded library
paths checked. The rest of the complete application/platform/test/packaging
requirements above remain outstanding.


## Linux transitions and filters checkpoint

`plugins::build_transitions` and `plugins::build_filters` build actual OBS
modules through the native engine and link them to the shared C library.
The filters options preserve independent SpeexDSP and RNNoise switches. The
bundled RNNoise fallback uses a canonical JSON C source list shared with CMake;
the CMake target retains the same 18 source/header entries.

The opt-in integration test loads these modules through `obs_open_module` and
`obs_init_module`, then verifies transition/filter registration against both
libobs variants. Each variant runs in a separate process: the shared SONAME
allows plugin dependencies to retain a previous libobs instance if both variants
are tested in the same process. This matches existing C test isolation and normal
OBS execution. These checks do not yet exercise rendering or audio processing.


All four SpeexDSP/RNNoise combinations pass module registration against both
variants, as does disabled HEVC with the default modules. The actual CMake
RNNoise fallback target also compiles all seven objects after the canonical-list
migration; Clippy, gersemi and the build-group/platform guards pass. System
RNNoise discovery is implemented, but this host exercises the bundled fallback.
