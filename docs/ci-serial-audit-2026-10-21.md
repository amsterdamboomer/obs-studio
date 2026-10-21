# Linux Tier 2 concurrency audit — 2026-10-21

Issue: [#1634](https://github.com/obs-rust/obs-studio/issues/1634).
Native binaries built from `0b10a2f61389076e7eea82343a4d14f80b5b1723`.

## Findings

The ten CTest `RUN_SERIAL` properties prevent their test process from
overlapping **any** other CTest process in the same invocation. They do not
reserve a CPU, isolate the host, or imply that each test needs its own machine.
Eight have no identified resource conflict across processes. One mutates a
shared display; one is especially sensitive to CPU scheduling.

| Test | Resource / reason | Proposed scheduling |
|---|---|---|
| `test_async_delay_audio` | Process-owned source and real-time audio assertions | Concurrent; retain assertions and timeout |
| `test_audio_clock_drift` | Synthetic source timestamps; does not change the system clock | Concurrent |
| `test_deinterlace_2x_field_order` | Real-time field delivery; software-renderer CPU contention | Concurrent with bounded renderer workers; eight capped full-suite trials passed this test |
| `test_inhibit_sleep_dbus` | Private GTestDBus and fake service | Concurrent; preserve private bus |
| `test_media_cover_art_seek` | Process-owned media/source; timestamp-named temporary files | Concurrent |
| `test_pipewire_screencast_resume` | Private PipeWire daemon, bus and temporary runtime directory | Concurrent; preserve wrapper |
| `test_pipewire_screencast_shm` | Same private PipeWire wrapper | Concurrent; preserve wrapper |
| `test_screencast_portal_teardown` | Private GTestDBus and fake portal; environment changes affect only its process | Concurrent |
| `test_vlc_audio_pacing` | Process-owned VLC source; timestamp-named files; real-time pacing assertions | Concurrent; retain assertions and timeout |
| `test_xshm_resolution_change` | Changes global RandR resolution on its X server | Private `xvfb-run -a`; keep serial fallback if unavailable |

Many guards arrived together in `142aad0a7` (#837), when parallel CTest was
introduced. Their presence alone is not evidence of a machine-wide dependency.

### CPU contention

Mesa llvmpipe normally creates rendering workers according to available cores.
Eight test processes can each create a worker pool. `LP_NUM_THREADS=1` bounds
each software renderer without changing OBS frame-rate settings or assertions.
One observed deinterlace process had 14 threads with the default and 8 with
the cap; the cap does not limit every thread in the process.
See [Mesa's environment documentation](https://docs.mesa3d.org/envvars.html)
and [llvmpipe documentation](https://docs.mesa3d.org/drivers/llvmpipe.html).

The candidate appends that environment setting to Linux CTest registrations
and sets it in the Linux validation harness. Existing environment entries,
test names, skip behavior and timeouts are preserved. It gives XSHM its own
display and removes the other broad barriers. No C test body or public header
is changed.

### Restrictions to retain

Rust integration helpers using `--exact <test> --test-threads=1` already start
independent child processes. The one-thread option applies to a child running
one test; it does not serialize the parent suite. Keep it.

Allocation counters (`bnum_allocs`), log-handler replacements, callback
stand-ins and oracle globals are shared **within a process**. Existing child
isolation or locks protect those checks. Separate processes can share one
machine. Examples: `rust/obs-util/tests/common/mod.rs`,
`rust/obs-output/tests/output/main.rs`,
`rust/obs-callback/tests/callback/proc_parity.rs`, and
`rust/obs-missing-files/tests/missing_files/main.rs`.

Build serialization (Cargo target locks and Windows x86 build limits) is a
different concern; this audit does not propose removing it.

Cmocka cases inside a single executable still run sequentially. Many reuse
the process's libobs singleton and fixtures. Parallelizing executables
already permits concurrency on one machine without making those fixtures
thread-safe or splitting every case into a separate runner.

## Runtime evidence

The experiment uses the existing Linux validation image and build volume,
four allowed CPU cores (`taskset -c 0-3`) and `ctest -j8`. OFF loads
`libobs-c-reference`; ON loads hybrid `libobs`. An `ldd` check verifies the
loaded backend. The build compared 1,766 exported symbols and excluded 63
ported C implementations from hybrid libobs.

This is a shared Ryzen 7 3700X host. Affinity is not exclusive CPU reservation;
host load varied substantially. Results measure CTest execution, excluding
compilation, downloads, runner provisioning, queue delay and final CI gate.

| Configuration | C reference (OFF) | Hybrid (ON) |
|---|---:|---:|
| Original ten serial guards, complete suite | 325.5 s, pass | 338.3 s, pass |
| Remove all ten, private XSHM, default renderer workers | 99.8 s, **deinterlace failure** | 95.6 s, pass |
| Keep deinterlace serial, remove other nine | 173.9 s, **deinterlace failure** | 168.1 s, **output-delay failure** |
| Remove all ten, private XSHM, one renderer worker | 100.1 s, pass | 98.3 s, pass |
| Actual candidate registrations, repeat 1 | 96.6 s, pass | 94.1 s, pass |
| Actual candidate registrations, repeat 2 | 93.9 s, pass | 98.1 s, **output-delay failure** |
| Actual candidate registrations, repeat 3 | 93.7 s, pass | 95.8 s, pass |

All six actual-source runs retained 346 registered CTests, 2,146 cmocka case
starts, all 40 deinterlace cases, identical internal skip names and the same
three top-level skips. Their output was not truncated. Five passed; one hit
the output-delay race. Deinterlace passed all eight capped full-suite runs
(the initial exploratory pair plus the six actual-source runs).

An additional original-policy OFF run used the repository's two-shard
planner with the measured baseline costs: 172 and 174 tests, respectively.
The shards took 155.9 s (deinterlace failed) and 176.8 s (passed), measured
sequentially on the same host. Their sum is 332.8 test-runner seconds; the
longer is 176.8 s. The longer time is only an estimate of ideal concurrent
latency, not an observed concurrent workflow time. This plan may differ
from the historical costs used by a particular CI run.

Their combined full outputs contain 2,146 cmocka case starts. Both initial
capped full runs also contain 2,146 starts, all 40 deinterlace cases, and the
same internal skipped-case names as the combined old shards. They contain
no truncated outputs. This checks execution below CTest's executable count.

Every complete run registered 346 CTests. The same three unavailable
PulseAudio tests were skipped: `test_pulse_monitor_device`,
`test_pulse_stream_restart`, `test_pulse_two_sources`. Initial passing outputs
were truncated by CTest; subsequent validation retains full output to compare
internal cmocka execution and skips as well.

### Failures are retained

* Unbounded parallel OFF failed `linear_2x_top_first_50i_into_50`: three skipped
  fields, where the unchanged assertion permits none. This is the timing
  sensitivity tracked by [#456](https://github.com/obs-rust/obs-studio/issues/456).
* OFF also failed deinterlace when it was still serial and ran first, alone
  within CTest. Serialization cannot protect it from other host workloads.
* ON with deinterlace serial failed `test_output_delay_e2e` at its inactive
  assertion after `deactivate`. `libobs/obs-output.c.h` signals `deactivate`
  before clearing `output->active`. This matches the ordering problem already
  tracked by [#1391](https://github.com/obs-rust/obs-studio/issues/1391).
  It is not proof of a cross-test resource collision.

No assertion, expected frame count, test selection or retry policy was relaxed
to hide these failures. A successful small group or single full run is not
enough evidence for a reliability claim.

The second actual hybrid run reproduced `force_stop_skips_delay` at line 484
with the renderer cap enabled. Thus the cap does not make the full suite
reliably green: the output lifecycle race remains a separate blocker to a
confident rollout. Do not replace this failure with a new whole-suite serial
guard; the signal/active-state ordering needs its own fix and regression proof.

## Sharding decision

The existing workflow uses two shards per backend: four test jobs. Once the
concurrent suite is reliable, one complete invocation per backend would use
two jobs and avoid duplicated runner setup. Keep OFF and ON coverage.

Recommendation: retain the current CI shard matrix while the candidate is
reviewed and #1391 is resolved. Then trial one shard per backend and measure
actual gate latency and runner cost. This investigation does not change the
workflow matrix or claim that timing flakes have been eliminated.

The unsharded capped trial is about 3.3 times faster than the original
unsharded suite on this host. That is **not** a measured CI gate speedup or a
comparison with two concurrent CI shards. The repeated actual-source runs
support the concurrency mechanism but expose the remaining race. The broader
representative-day runner-cost/gate study remains
[#1581](https://github.com/obs-rust/obs-studio/issues/1581).

## Reproduction and artifacts

Verification: 27 focused configuration, environment and shard tests passed;
the changed CMake scheduling files passed pinned gersemi; Bash syntax and
`git diff --check` passed. The full local gate was not run. No Rust source or
protocol implementation changed; the runtime evidence here is Tier 2 against
both backends. The candidate remains for review; PR CI decides a merge.

Build the native suite using `OBS_TIER2_STAGE=build
rust/tools/linux-validate/run.sh`, following `docs/ci-tooling.md`. Run baseline
and candidate scheduling against the same tree/image and both library paths,
with fresh HOME/XDG directories per run. Capture registry and complete output:

```sh
ctest --test-dir /build/tree --show-only=json-v1
LP_NUM_THREADS=1 taskset -c 0-3 xvfb-run -a \
  '--server-args=-screen 0 1280x1024x24 -noreset' \
  ctest --test-dir /build/tree -j8 --output-on-failure \
  --test-output-size-passed 10485760 --test-output-size-failed 10485760 \
  --output-junit /audit/result.xml
```

The command runs inside the repository's bounded `ci/docker-task.sh` wrapper;
set `LD_LIBRARY_PATH` to `/build/tree/libobs-c-reference` for OFF and
`/build/tree/libobs` for ON. Save every failure, rather than retrying until
green. Native code is identical across the comparison; CMake reconfiguration
applies the candidate registrations.

Local raw logs, JUnit, generated registry and experiment scripts are retained
in `build_ci_audit/serial1634/`. The [companion JSON](ci-serial-audit-2026-10-21.json) retains run
summaries, including unsuccessful experiments and observed host load.
