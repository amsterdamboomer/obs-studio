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


## Linux OpenGL checkpoint

`graphics::build_opengl` compiles the original OpenGL unity groups and GLAD loader,
links the shared backend, and creates its unversioned lookup name. It consumes the
same libobs options and generated configuration as the core. The Linux graph
includes X11/EGL and optional Wayland support; other platforms remain unfinished.

With an isolated X server available through DISPLAY, set `OBS_BUILD_RENDER=1` on
the opt-in integration test to exercise real graphics. Local verification used an
owned Xvfb process allocated via `-displayfd`, with `LIBGL_ALWAYS_SOFTWARE=1` and
`GALLIUM_DRIVER=llvmpipe`. Both variants initialize Mesa OpenGL 4.6, create a scene,
and receive at least three 64x64 NV12 frames through the public raw-video callback
API. The callback verifies a valid row and the expected black Y sample (16).
The probe uses the unchanged C headers for video configuration and callbacks;
shutdown and display cleanup happen before its shared libraries unload.

This demonstrates the native renderer and basic frame delivery. It does not yet
validate the full Qt application, recording, capture sources, all plugins,
Wayland runtime behavior or packaged artifacts. The prototype OpenGL version
number remains tied to the current OBS 33 configuration.


## Linux recording checkpoint

The native engine now links executables as well as shared libraries and archives.
Its executable contract runs a program linked to a shared core that consumes a
static helper. Windows dependency mapping preserves archive paths and converts
DLL paths to import-library paths; runtime validation there remains outstanding.

The module graph builds x264, the complete Linux obs-outputs groups and librtmp,
and FFmpeg with media playback, VAAPI, optional MPEGTS/SRT/RIST, logging and legacy
NVENC groups. It also builds the standalone FFmpeg mux executable. Shared
opts-parser, happy-eyeballs, BPM and libcaption dependencies use native archives.
The FFmpeg options follow current CMake defaults; the full option matrix and
hardware-specific behavior remain unverified.

Set `OBS_BUILD_RECORD=1` on the opt-in integration test, with native dependencies,
ffprobe and an isolated DISPLAY available. This implies the graphics probe. Both
variants load the encoder/output modules, verify registration and exactly one
loaded SRT library, then encode an isolated blank scene using real obs_x264,
ffmpeg_aac and mp4_output. The test stops the output, releases encoders, shuts
down OBS, and validates tracks, dimensions, decoded-frame counts and duration
with ffprobe. Local recordings each contain 18 H.264 frames at 64x64 and AAC
audio, duration 0.6 seconds. They contain no desktop or device capture.

`FfmpegArtifacts::deploy_muxer` places the helper beside the running app. The
integration stages its runner with that helper, selects the same libobs variant
for helper subprocesses, and also records through `ffmpeg_muxer` to Matroska.
All four files (MP4 and Matroska, C and Rust cores) pass the same ffprobe checks. The full Qt application, capture sources, other
modules/platforms, complete test battery, packaging and controlled performance
measurements remain required. Do not treat this checkpoint as a CMake retirement.

## Linux sources and services checkpoint

`plugins::build_source` provides typed selections for image sources, FreeType
text, PulseAudio, ALSA, V4L2, X11 capture and RTMP services. V4L2 preserves its
independent UDEV switch. RTMP service updates are configurable and remain off in
the integration smoke test to keep it offline. The default plugin behavior keeps
updates enabled when requested through its typed option. X11 capture and OpenGL
share the GLAD archive builder.

Set `OBS_BUILD_SOURCES=1` on the integration test with an isolated X display and
an EWMH window manager. Local checks use Xvfb plus an isolated KWin session;
XComposite correctly declines registration on bare Xvfb without EWMH. Both cores
load these seven modules and expose their thirteen expected input/output/service
registrations. The graphics probe also creates a 64x64 white color source,
adds it to the scene and requires three NV12 frames with Y=235 after draining
previous black frames. Both cores still record validated MP4 and Matroska files.
Disabled UDEV also passes registration and rendering against both cores.
The test does not open camera or microphone devices. Hardware capture, text
rasterization, streaming and service-update behavior remain to be validated with
the full application and packaged artifact tests.

## Linux PipeWire checkpoint

`SourceModule::PipeWire` builds the original Linux PipeWire unity group and
includes the camera group when the discovered PipeWire version is at least
0.3.60. It enforces CMake's PipeWire 0.3.33 and Gio 2.76 minimums and uses the
shared GLAD archive. Enable its opt-in build/load check with
`OBS_BUILD_PIPEWIRE=1`. Both cores load and initialize the module after graphics
initialization, then render and shut down. Portal availability, camera
permissions and actual screen/camera capture remain unverified; module load alone
does not establish those capabilities.

## Linux frontend API and Qt checkpoint

`frontend::build_api` builds the original C++ frontend API as a shared library.
It preserves Linux's SONAME 30 and creates its unversioned lookup name. C++17 and
hidden inline visibility match the CMake compiler configuration. Local symbol
comparison finds exactly 101 exports in both frontend libraries, with no missing
or extra symbols. `OBS_BUILD_FRONTEND_REFERENCE` optionally enables this complete
comparison against an existing CMake artifact in the integration test.

`qt::moc` runs Qt's meta-object compiler with the native target's include paths and
definitions and compiles its generated translation unit explicitly. It preserves
unchanged output contents/mtime. `OBS_QT_MOC` selects a host tool explicitly;
default discovery uses Qt6Core's pkg-config libexec directory. Application builds
still need UI/resource generation and complete build-script input tracking.

Enable `OBS_BUILD_QT=1` to build the real VST module and frontend API. Both cores
load the VST module and expose its filter; the frontend API's no-callback defaults
are callable. VST editor/audio behavior, the full Qt application and resources,
other platforms and packaging remain outstanding.

## Mixed native languages checkpoint

Native targets now accept common compiler options plus separate C and C++ option
vectors. OBS modules use C17 for C sources and C++17/hidden inline visibility for
C++ sources. Mixed targets compile each language independently and link with the
C++ driver. The regression builds and executes a C++ standard-library program
that calls a C helper; each translation unit rejects flags intended for the other
language. All five native integration tests pass, as does the combined
core/source/Qt/PipeWire/MP4/Matroska check with frontend export comparison.

## Qt UI, resources and widgets checkpoint

`qt::uic` generates UI headers with the generated include directory first.
`qt::rcc` embeds resource collections as explicit C++ sources and records every
resource listed by RCC as a Cargo input. Both preserve unchanged generated files.
`OBS_QT_UIC` and `OBS_QT_RCC` select host tools for cross builds, alongside MOC.

`frontend::add_spinbox` adds the original shared spinbox unity source, Qt Widgets
and all three generated metaobjects, with C++17 required explicitly. The Qt opt-in
integration now runs a real QApplication using the offscreen platform: construct
OBS's virtual-camera dialog from its original UI, find its controls, decode an
embedded OBS PNG, exercise both OBS spinbox types and their metaobject, process
UI events and accept the dialog through its OK button. This passes locally.
This widget test does not yet start the complete OBS application or validate a
packaged runtime/plugin path.

## Linux frontend executable checkpoint

`frontend::build_application` compiles and links the frontend executable using an
explicit source inventory, shared Qt interfaces, an Idian archive, JSON11, BPM,
libcaption, the frontend API and a caller-provided Rust GPU status archive. It
creates UI/plugin-path configuration in the output tree. This initial frontend
configuration is Linux with browser panels disabled; other feature/platform
configurations and release paths remain under conversion.

Build the GPU archive with
`soldr cargo build -p obs-gpu-status-frontend --release`, then enable the integration
check with `OBS_BUILD_APP=1` and `OBS_BUILD_GPU_ARCHIVE` pointing to the resulting
`target/release/libobs_gpu_status_frontend.a`. The executable runs and reports
`--version` identifies the OBS-Studio-Rust application version as `0.1.0`,
based on OBS `33.0.0-cc-build`.
This establishes compilation, linking and CLI execution; GUI startup, runtime
staging and complete application behavior still need validation.

The inventory includes two forms that CMake AUTOUIC discovered through header
includes. Generated metaobjects are combined into one translation unit with the
same textual header closure CMake supplies to its moc compilation. This preserves
complete Qt types without changing original headers or requiring precompilation.
Compiler include paths and identical definitions are deduplicated while retaining
order. Build/profile/version/install settings are still prototype defaults, and
controlled performance measurements have not been completed.

## Linux staged GUI runtime checkpoint

Set `OBS_BUILD_APP_GUI=1` with `OBS_BUILD_GPU_ARCHIVE`, native dependencies,
an isolated X display and an EWMH window manager to run the complete current
Linux graph. This implies the frontend, Qt widget probe, fourteen implemented
modules, OpenGL, source rendering and the MP4/Matroska API recording checks.

The GUI test stages the executable, selected core, frontend API, renderer,
modules, shaders, application/plugin data and FFmpeg helper under a retained
`app-runtime-*` directory in the native output tree. Each core starts in a fresh
process with its own saved profile and white color-source scene. The test checks
startup, a mapped normal main window, the staged library and module paths in
the process map, restored source, screenshot, normal window-close shutdown and
persisted scene. Logs, process maps and screenshots remain in that runtime.
Both C and Rust cores pass these checks locally; both screenshots show the
restored white preview. The combined run also validates four H.264/AAC files
(MP4 and Matroska for each core). This is runtime evidence, not a controlled
build-performance comparison.

The isolated test clears desktop-session Qt plugin paths and themes so a host
desktop built with another Qt installation cannot override the linked Qt's
platform plugins. It retains the linked Qt's default plugin discovery and uses
Fusion styling. System native dependencies remain installed on the host; this
development layout does not establish portable release packaging. Hardware
capture, browser/websocket features, remaining modules/platforms,
full test tiers and controlled performance measurements are still required.

Add `OBS_BUILD_APP_RECORD=1` alongside `OBS_BUILD_APP_GUI=1` to exercise the
frontend's real recording path. Its private profile selects x264/AAC and
Matroska at 640x360/30fps; `--startrecording` starts the frontend output. The
probe requires the recording-start log, records for three seconds and closes
the window normally. ffprobe must decode at least sixty video frames and find
H.264, stereo AAC at 48kHz and the expected dimensions/duration. FFmpeg decodes
the first video frame and checks that it contains the restored white scene.
The finalized recording and ffprobe JSON remain under the retained runtime.
Both cores pass locally: each file has 98 decoded H.264 frames, 48kHz stereo AAC
and a 3.366-second container duration. Both screenshots show recording active
with the white preview. The private headless session's desktop portal backend
does not remain healthy; these checks establish no screen/camera portal capture.

## Linux NVENC checkpoint

`plugins::build_nvenc_pair` compiles common NVENC objects once, then links the C
CUDA loader and Rust CUDA loader variants separately. The latter accepts the
outer Cargo build's `obs-nvenc-plugin` archive and hides its Rust symbols. The
capability helper is a separate C++17 executable with thread/dynamic-loader
linkage. FFnvcodec headers must be version 12 or newer. The FFmpeg compatibility
encoder IDs remain independently configurable.

Build `soldr cargo build -p obs-nvenc-plugin --release` and set
`OBS_BUILD_NVENC=1` on the native integration test. This opt-in check requires
a working NVIDIA driver/GPU: the helper must report support, and both plugin
variants must load and register their H.264 encoders against their corresponding
core. `OBS_BUILD_NVENC_RUST_ARCHIVE` overrides the default archive location;
`OBS_BUILD_NVENC_REFERENCE` optionally compares all exports to a CMake module.
The local comparison finds exactly eight exports in all three modules.
`OBS_BUILD_DISABLE_NVENC_FFMPEG_IDS=1` disables the compatibility IDs.

Add `OBS_BUILD_NVENC_ENCODE=1` with recording mode to exercise the soft-frame
hardware encoder and therefore the actual CUDA driver loader. This uses 640x360
because NVENC rejects the tiny 64x64 software probe. Both variants produce
validated H.264/AAC MP4 and Matroska files locally. Source cleanup explicitly
removes public sources from their canvas, releases owned references and drains
the destruction queue before shutdown, matching the frontend sequence.

Nix verification provides the installed driver libraries through
`LD_LIBRARY_PATH=/run/opengl-driver/lib`. This checkpoint covers native module
loading and soft-frame H.264 encoding. OpenGL texture interoperability,
HEVC/AV1 encoding and other platforms still
need validation.

## Linux staged GUI NVENC checkpoint

Combining `OBS_BUILD_NVENC=1`, `OBS_BUILD_APP_GUI=1` and
`OBS_BUILD_APP_RECORD=1` now stages fifteen modules and the NVENC capability
helper. Explicit runtime variants select the C loader with the C core and the
Rust loader with the Rust core. The core's bundled-module whitelist includes
NVENC when this option is requested.

The private frontend profile selects NVENC. In addition to the existing GUI
checks, the test requires the staged NVENC module in the process map and the
actual recording encoder's NVENC settings log. Both variants pass locally,
producing 99/100 decoded H.264 frames at 640x360 with AAC audio and white images;
screenshots show recording active and both apps shut down normally. On the
isolated Mesa display, NVENC reports its expected fallback to frame upload.
This demonstrates frontend hardware encoding through both loaders; it does
not establish direct NVIDIA OpenGL texture interop.

## Linux Lua/Python scripting checkpoint

`scripting::build` compiles the original shared scripting unity groups and
C++ string cache. Lua and Python remain independent options, and a supplied
frontend API enables the original frontend scripting groups. SWIG 4 or newer
generates separate language runtimes and wrappers; `OBS_SWIG` selects the host
tool. The build tracks SWIG environment/input trees, writes generated sources
and headers only when their contents change, and explicitly compiles them.

Bindings are placed under a retained runtime root with a configurable relative
module directory (default `lib64/obs-scripting`). The scripting library uses
executable-relative discovery. Artifacts identify the enabled Lua module,
Python extension and Python wrapper explicitly; disabled-language files from
an earlier build are not an installation manifest. Release packaging must use
the selected artifacts.

Enable the opt-in check with `OBS_BUILD_SCRIPTING=1`. It builds a native probe
against the unchanged scripting API, then runs it with each core from an
unrelated working directory and private module configuration. Real Lua and
Python scripts exercise defaults, update, properties, save, reload and unload
callbacks. Process maps must contain the intended core and generated binding,
and the supported-language list must match the build options. Logs remain in
`script-scenarios/` under the native output directory.

`OBS_BUILD_SCRIPTING_FRONTEND=1` enables frontend bindings and verifies their
default callable behavior; the disabled mode checks that they are absent.
`OBS_BUILD_DISABLE_LUA=1` and `OBS_BUILD_DISABLE_PYTHON=1` select the independent
language cases. Local checks pass both languages with and without frontend
bindings, Lua-only with a relative native output path, and Python-only with
frontend bindings, against both cores. The environment uses SWIG 4.4.1,
LuaJIT and Python 3.13 development packages discovered through pkg-config.

The shared-library check remains separate from the frontend Tools application
check below. Other platforms and release packaging still require conversion
and validation.


## Linux frontend Tools and saved GUI scripts

`plugins::build_frontend_tools` compiles the original scene-switcher,
output-timer and optional scripting groups, their Linux/POSIX host sources,
shared Qt widgets, generated forms and moc units. Supplying scripting artifacts
enables the matching Lua/Python UI definitions and shared-library dependency.
The graph reuses the application's shared Qt interface inventory.

Set `OBS_BUILD_FRONTEND_TOOLS=1` with `OBS_BUILD_APP_GUI=1` to stage the plugin
and enabled scripting artifacts in each private runtime. Saved Lua/Python
scripts are restored from the scene collection. The app test checks their
load/update/save/unload callbacks and persisted settings, as well as the
actual staged scripting libraries in the process map. It copies only the
selected language artifacts, excluding stale disabled-language output.

A test-only Qt module opens Automatic Scene Switcher, Output Timer and Scripts,
checks the saved script count and each script's description/Seed property,
captures each dialog and closes it. Inspection
runs inside Qt's event loop, including Scene Switcher's nested modal loop.
The probe is part of the development test runtime and is not a release plugin.
`OBS_BUILD_DISABLE_SCRIPTING=1` exercises Tools without the scripting library.
Combine `OBS_BUILD_APP_RECORD=1` to retain the actual frontend recording checks.

Local dual-language and scripting-disabled app checks pass against both cores,
including real frontend H.264/AAC recording and the four native API recordings.
The combined Tools, Lua/Python and NVENC configuration also passes against
both cores, with the selected C/Rust NVENC loaders and helper staged together.
All six dual-language dialog screenshots and the recording main window were
visually inspected. The disabled case requires the Scripts action to be absent.

Both configurations can print one outstanding allocation during shutdown in
this headless desktop session. An allocation tracker identifies the original
portal sleep-inhibitor object (`portal_inhibit_info_create`); a diagnostic
nonblocking GLib main-context drain releases its pending callback and changes
`bnum_allocs()` from one to zero. The diagnostic is external to the production
build and app probe. This checkpoint establishes normal app exit and scripting
cleanup, not a zero-allocation shutdown log or portal capture support.

Windows/macOS Tools, remaining modules, full application/capture coverage and
release packaging are still required.

## Linux websocket server and protocol checks

`plugins::build_websocket` compiles the pinned original obs-websocket 5.7.5
server, its 52 C++ sources, generated moc/UI inputs and resources. Dependencies
include Qt Widgets/Svg/Network, Asio, nlohmann-json, websocketpp and qrcodegencpp.
Header/library overrides support dependencies outside the system prefixes.
The submodule source and pin are unchanged.

Enable `OBS_BUILD_WEBSOCKET=1` with `OBS_BUILD_APP_GUI=1`. The staged application
loads the server with each core, rejects a wrong password, authenticates a real
client and exercises scenes, inputs, settings, events, batches, errors and a
decoded white source screenshot. With `OBS_BUILD_APP_RECORD=1`, the wire client
starts recording after those checks; the retained H.264/AAC recording is then
decoded and checked. A Qt probe opens Server Settings and Connect Info, checks
that the QR image contains dark and light pixels, and captures both dialogs.

`OBS_BUILD_DISABLE_WEBSOCKET_UNITY=1` compiles the 52 separate C++ sources.
Both modes pass the GUI, wire and recording checks against both cores. Their
11 public `obs_module*` entry points agree; incidental exported C++ template
symbols differ. The core has a separate typed unity option, described below.

The existing `test/websocket-tier4` suite also passes against both staged cores:
193/193 checks and 147/147 documented request types. The external fixture
adapter aliases its private `obs-studio` configuration directory to the renamed
product's `obs-studio-rust` directory; protocol assertions remain unchanged.
The combined websocket, Tools, Lua/Python and NVENC application passes with
17 production modules plus the test-only Qt probe. Settings/Connect Info
screenshots from both cores were visually inspected.

These are functional checks, not controlled build-speed measurements. Browser,
remaining plugins, real capture profiles, Windows/macOS native graphs,
production packaging and configurable unity/PCH across the remaining graph
remain required.

## Shared source inventory and Linux Tier 2

`rust/tools/ported_sources.py` reads `libobs/build/source-sets.json` and expands
the selected unity groups for Tier 2's native-object exclusion checks. CMake
and Cargo share the optional HEVC group in that manifest. The reader matches
the actual CMake inventory exactly: 46 Linux objects with HEVC and 45 without.
The existing minimum inventory size and Ninja graph checks remain in force.
The local validation image includes pinned uv and checks offline system-Python
startup while building the image, matching the repository's Python policy.

This fixes the CI reader that found only one object after the source lists
moved out of CMake. The complete Linux Tier 2 CI result still needs verification
on the updated branch; local source parity and HEVC on/off lifecycle/symbol
checks pass (1766/1762 exports respectively, identical between cores).

## Individual libobs source compilation

`libobs::Options::unity` defaults to true, preserving the selected explicit
groups. Set it to false to compile each implementation separately. The native
engine expands C/C++ BUILD.md groups into generated one-member translation
units, preserving their group headers and keeping the source tree unchanged.
Ordinary and explicitly generated sources retain their original paths.
Wrappers are rewritten only when their contents change. Cargo rerun metadata
tracks the groups, their headers and their implementation members; compiler
includes retain direct dependency on the original contents.

The expander rejects conditional or otherwise unsupported group directives
instead of flattening them into a different source selection. It also rejects
duplicate members and includes outside the source root. Objective-C groups
still require platform implementation.

`OBS_BUILD_DISABLE_CORE_UNITY=1` selects this mode in the native integration
driver. Local checks pass lifecycle/export parity with HEVC on and off against
both cores. With HEVC enabled, all 1766 exports match the retained unity-built
core exactly, and 51 existing C test executables pass against each core with
loader resolution verified. The browser-disabled GUI, websocket wire checks,
white preview/screenshot and real frontend/API recordings also pass against
both cores. These C executables were built by CMake; their build migration is
still required. The direct native fixture also verifies member/header edits
change the loaded result; that is compiler invalidation evidence, not a full
Cargo scheduling experiment.

This establishes the core option. Other targets still need to expose it
consistently, and optional compiled PCH and controlled performance comparisons
remain outstanding.

The subsequent stock-container CI run exposed a separate missing-uv dependency:
unlike the local Docker image, both Tier 2 jobs start from stock Ubuntu. The
existing build and test jobs now provision the same pinned uv before invoking
validation. No workflow files or jobs were added; remote verification remains
required.

## Linux VLC media source

`SourceModule::Vlc` builds the original VLC core and POSIX host groups.
It discovers headers through pkg-config, `OBS_VLC_INCLUDE_DIR`, `VLC_PATH`
or standard include directories. Header discovery adds no libVLC link edge:
the original plugin still loads the optional runtime dynamically. The native
check inspects the ELF dependency list to enforce that distinction.

Set `OBS_BUILD_VLC=1` with the staged GUI and websocket checks. The real app
creates a VLC playlist source, decodes a six-second local H.264 clip, returns
a white source screenshot, reports its duration and handles pause/play/restart.
Normal shutdown saves the playlist path in the scene collection. Process maps
must contain the staged plugin and dynamically loaded libVLC.

Both cores pass those checks, frontend recording and the four API recordings.
The combined VLC, Tools, Lua/Python, websocket and NVENC application also passes
with individual-source libobs compilation: 18 production modules plus the
test-only Qt probe. That run uses an explicit include directory with libVLC's
pkg-config lookup disabled, exercising header discovery independently.

`OBS_BUILD_VLC_ABSENT=1` selects the runtime-absence check. In an environment
without libVLC, both cores log that the source is disabled, leave `vlc_source`
unregistered, retain the loaded plugin and exit normally. Process maps must
confirm the runtime is absent; a discovered system installation fails this
fixture rather than weakening the check. This does not establish packaging
of VLC, every codec/playlist mode or actual audio-device output.
