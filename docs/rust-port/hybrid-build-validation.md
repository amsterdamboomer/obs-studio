# Hybrid default validation (#1587)

Implementation PR: [#1588](https://github.com/obs-rust/obs-studio/pull/1588).
The developer default is hybrid libobs. Release approval remains pending;
this report does not waive release-criteria.md or authorize publication.

## Build timing

[Raw commands and measurements](benchmarks/hybrid-build-2026-10-20.json)
compare baseline `90e4edc41` with hybrid implementation `174109fad`. Both use
one Linux dependency image, GCC 15.2.0, Rust 1.98.1, CMake 4.2.3,
RelWithDebInfo native code, release Rust code, and four compilation jobs on
a Ryzen 7 3700X. Each variant starts with a separate empty compiler cache and
fresh output tree; crate downloads can be warm. Configuration is timed
separately. The baseline builds the two default libobs products (`libobs`,
`libobs-with-rust`); the hybrid builds only production `libobs`.

| Scoped libobs operation | Baseline | Hybrid |
| --- | ---: | ---: |
| Configure | 8.223 s | 7.544 s |
| Fresh outputs and initially empty compiler cache | 11.406 s | 14.041 s |
| No changes, median of three runs | 0.115 s | 0.120 s |
| Touch generated configuration header, warm cache | 7.970 s | 5.995 s |
| Append a comment to that header | 11.184 s | 5.779 s |

The fresh build was slower; the warm and edited-header rebuilds were faster
in these samples. The shared host was busy (load averages are recorded), and
there is only one sample for each rebuild type. These figures cannot establish
a general build-speed improvement. They exclude frontend/plugins, packaging,
image setup and full-application build time. An earlier exploratory pass used
a shared compiler cache and is deliberately excluded from this comparison.
The raw child-CPU fields exclude detached cache-daemon work and are not total
compiler CPU measurements.

To reproduce, create baseline and hybrid source snapshots, mount them read-only
as `/baseline` and `/source` in the recorded image, and use separate `/build`
output and compiler-cache directories for each variant. Execute the recorded
configure/build commands. After the fresh build, run the same build three times;
then touch its generated `config/obsconfig.h` and rebuild. Finally append an
otherwise inert comment to that header and rebuild. Keep the same toolchain,
profile, job count and source snapshots; record host load and all outcomes.

## Compatibility and test time

- The real Linux production and C-reference libraries export identical sets
  of **1,766 symbols**. Their SONAMEs match. The Ninja graph confirms all
  **61 selected ported C implementations** are absent from production and
  present in the explicit reference.
- With HEVC disabled, a separate real build exports **1,762 identical symbols**,
  with no HEVC symbols leaking from the aggregate Rust archive.
- The unchanged C suite registered **341 tests per backend**, with no failures
  and the same three hardware-dependent skips. Loader-path checks verified
  the C reference for OFF and production hybrid for ON. Wall times were
  **392.31 s** and **390.62 s**, respectively. These are backend runs of the
  same suite, not before/after test-speed evidence.
- The full Rust workspace unit, differential and documentation tests pass.
  The opt-in native integration category compiles with `libobs-integration`.
  An obsolete NVENC test requiring the installed C loader was updated to
  require the production Rust loader and explicit C reference instead.
- `./lint` passed before the final test/report additions; final checks and PR
  results are recorded on the PR.

Compatibility test coverage has not been reduced to create a speed claim.
Ordinary application builds avoid the C reference; an explicit compatibility
run still builds and exercises both implementations. No test-time reduction
is established by these measurements.

## Packaged application and remaining release evidence

A production glibc-2.28 AppImage built successfully through the existing bosn
`appimage-228` task, including the unchanged C++ websocket plugin. Its first
attempt encountered a compiler-cache daemon outage; the successful retry
used four compilation jobs with caching disabled and reused completed objects.
This is functional build evidence, not a clean full-app timing comparison.

Saved-profile runtime validation, the packaged websocket Tier 4 suite,
Flatpak offline builds and supported-platform CI results are tracked on the PR.
They must not be inferred from the libobs-only checks above. Hardware recording
performance and fourteen consecutive daily stability builds remain separate
release requirements. The checked-in release evidence remains unapproved,
and both publication workflows fail closed until its review is recorded.
