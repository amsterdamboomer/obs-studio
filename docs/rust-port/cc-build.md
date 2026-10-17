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
