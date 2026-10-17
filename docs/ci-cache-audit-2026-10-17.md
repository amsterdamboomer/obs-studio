# CI cache audit: 2026-10-17

Implementation and measurement tracker:
[obs-studio#1286](https://github.com/obs-rust/obs-studio/issues/1286).

## Rust compiler cache: verified blocker

The pinned soldr 0.9.35 still reproduces
[zccache#1943](https://github.com/zackees/zccache/issues/1943). A nested Rust
module changed from a comment to `compile_error!`, preserving its size and
nanosecond mtime. Touching the unchanged crate root forced Cargo to invoke
rustc. The cached command returned success with a HIT; the same command
with `SOLDR_RUSTC_WRAPPER=none` failed with the injected diagnostic.

| Step | Exit | Probe error reported | Seconds |
| --- | ---: | --- | ---: |
| Warm valid source | 0 | No | 2.888 |
| Cached changed source | 0 | No: false success | 2.052 |
| Uncached changed source | 101 | Yes | 1.621 |

These durations describe a tiny correctness fixture, not a performance
benchmark. [Captured result](ci-evidence/2026-10-17-rust-cache-source-probe.json).
Keep the local/CI gate's wrapper bypass until this reproduction passes on
the version we adopt. The standalone Linux C/C++ PCH cache upgrade in
[#1265](https://github.com/obs-rust/obs-studio/pull/1265) is separate from
soldr's embedded Rust cache and does not prove this issue fixed.

Reproduce from the repository root with the same executable CI pins:

```sh
uv run ci/cache_source_probe.py \
  --directory build_ci_audit/source-probe-new \
  --soldr /absolute/path/to/soldr --expected-version 0.9.35
```

Use a new scratch directory for every run. The probe preserves logs and
`result.json`, and exits 1 when the cache fails the source-safety check.
The explicit binary and expected-version check matter: in this checkout,
`uv run` found soldr 0.9.34 on its environment PATH while the shell's soldr
was 0.9.35. Both reproduced the stale success; the table records the explicit
0.9.35 run. No cache or daemon was purged to obtain these results.

The required gate's existing source probe now exercises a nested module
edit preserving size and mtime, so it guards the workaround against this
specific failure mode rather than checking only an ordinary append.

## Timing persistence and scheduled retention

[#1288](https://github.com/obs-rust/obs-studio/pull/1288) enables hidden-file
uploads for `.tier2-cost` and rejects missing or unmeasured OFF/ON timing
files before writing a cache. Master artifact production and subsequent
consumption must be observed before declaring the timing pipeline repaired.

The [failed scheduled run](https://github.com/obs-rust/obs-studio/actions/runs/39002695404)
was rejected before job execution: its annotation says the nested `ci-mode`
job requested `actions: read` but the caller allowed none. This was a workflow
permission error, not a cache script failure.
[#1289](https://github.com/obs-rust/obs-studio/pull/1289) grants that permission
and removes the independent nightly compiler-cache purge. Cache retention
belongs to the existing merge-group/master janitor and `ci.toml`; scheduled
builds reuse its warm entries. A live full dispatch remains the startup check.

## A day of baseline CI traffic

The window 2026-10-16 17:00 → 2026-10-17 17:00 UTC contains 473 Pull/Push
workflow runs. Both workflow listings extend before the start of the window,
so the listing limit did not truncate it. Detailed jobs and actual changed
files were inspected for 117 runs, sampled across event/outcome and time,
with extra documentation candidates whose diffs were verified. These are
sample medians, not weighted population estimates.

| Successful gate cohort | Samples | Median created-to-gate time |
| --- | ---: | ---: |
| Code PR | 9 | 16m 56s |
| Code merge group | 8 | 15m 54s |
| Docs PR | 1 | 4m 01s |
| Docs merge group | 2 | 6m 58s |
| CI/config merge group | 2 | 2m 55s |

The sparse docs samples do not establish a stable typical duration.
Across the full listing, merge groups had 35 successful, 163 canceled and
6 failed runs. The sampled canceled code merge groups alone consumed
194.45 runner-minutes. That is observed sampled waste, not an extrapolated
total, and the run outcomes alone do not establish why cancellation occurred.
The window spans earlier CI/build rollouts; this baseline cannot attribute
a later difference to any single cache or concurrency change.

[Summary, outcome counts, listing coverage and per-lane medians](ci-evidence/2026-10-16_17-ci-baseline-summary.json)
and [sample IDs, diffs, gate and runner times](ci-evidence/2026-10-16_17-ci-baseline-samples.json)
are retained. Cache inventory in the summary is a snapshot at collection
time, not a reconstruction of cache contents at the historical window end.
The full raw run/job/cache responses remain under `build_ci_audit/metrics-before/`
in the measurement checkout; the collector reproduces that evidence.

`ci/cache_metrics.py` resolves source cohorts from actual compare diffs,
including an unambiguous PR branch/lifetime fallback for older runs whose
API metadata no longer contains a PR association. Missing/ambiguous diffs
remain unknown. It reports latest run attempts only and does not reconstruct
overwritten attempts. In addition, every merge-group CI Gate summary now
lists completed setup/cache step durations from the already-fetched job
metadata, without adding another API request to the gate.

Post-change measurements must use a separate window and preserve these
cohort/coverage distinctions. The day-long post-change comparison remains
open until the configuration changes have landed and that window exists.
