# Optional Flatpak capacity experiment (#1269)

The trusted producer published a2,748,017,078-byte builder archive. Its
working downloads directory is about1,026,740,380 apparent bytes, and the
builder state also carries compiler entries published separately. These
observations alone do not prove a safe or beneficial exclusion.

The existing full manual workflow accepts `flatpak-cache-probe`, default
false. It adds a bounded probe **after Flatpak output validation**, preserving
all full-build/test selection and existing production cache flags, keys,
publication guards and limits. No new workflow file or pruning is introduced.
The native-cache-probe input remains independent and false for this trial.

## Measurements

`ci/flatpak_cache_probe.py` records installed builder/zstd versions and:

1. Copies downloaded sources into an isolated temporary warm state. Measures
   `flatpak-builder --download-only` with the actual prepared manifest/SDK.
2. Measures the same acquisition command against a separate fresh state.
   Both build-directory arguments are temporary; neither command receives
   the original state directory. Copy/setup time is outside the acquisition
   timers, so the pair is not total restore/build latency.
3. Streams canonical POSIX tar through zstd and counts compressed bytes for
   full state, excluding downloads, and excluding downloads plus compiler
   state. No compressed archive files are written. These are **proxy sizes**,
   not the pinned Actions cache implementation's actual archives/transport.

Acquisition commands are bounded by GNU timeout at180seconds, compression
pipelines at120seconds, with a ten-second kill grace. The optional step has
a15-minute deadline. Temporary copies are cleaned on normal/error paths.
Scratch inside original state and symbolic links in downloaded sources are
rejected rather than permitting recursive compression or writes through a
copied link. Original state is read only for copying and compression; no
original files are moved, deleted or recompressed. Reading may affect atime.

A command failure remains a failure. The CLI writes a failed-status JSON
and raises; the measurement artifact is uploaded after failure too. No
continue-on-error bypass is used. Seven-day artifact retention applies only
to the small measurement JSON, not a relocated binary cache backend.

## RED → GREEN / local verification

RED `f1062b947` fails the absent helper and absent opt-in wiring. Fixtures
execute real GNU tar/zstd/timeout with a local acquisition executable. They
check the exclusions' measured byte differences, copied-state isolation,
original contents/mtimes, scratch cleanup on failure, opt-in defaults and
placement after output validation. Additional boundary RED evidence rejects
scratch inside the original state before the guard is implemented.

CI tests load the helper through standard-library importlib, following the
repository's standard-library-only rule. No Rust/C source or packaged content
changes; no artificial port tiers are claimed for an infrastructure probe.

## Defined hosted trial and adoption gate

After #1711's currently active new-namespace consumer39720695807 completes,
run exactly one full workflow on the pushed probe GREEN branch with
flatpak-cache-probe=true and native-cache-probe=false. Preserve the resolved
source, job IDs, command logs, versions and JSON. Do not rerun a full workflow
for an unrelated failure. Keep watcher cancellation disabled for the trial.

No hosted result is observed yet. One paired source-fetch observation does
not establish representative network cost or an end-to-end speedup. It also
does not prove a pruned state can rebuild the application: any adoption needs
actual changed-cache producer/consumer build/output validation, required
coverage and an honest global allocation. Native architecture retention and
repository headroom remain unresolved under #1269; no smaller native limit
or production exclusion is adopted by this unit.
