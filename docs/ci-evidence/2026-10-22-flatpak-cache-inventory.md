# Flatpak builder-state inventory experiment (#1269)

The retained master builder archive observed in the earlier full validation
is 2,747,488,816 compressed bytes. A later real consumer reused six dependency
modules, but neither observation identifies which state directories dominate
the archive. Do not reduce its allowance or remove module state on that basis.

The pinned action [v6.8 implementation](https://github.com/flatpak/flatpak-github-actions/blob/79327416609af08178ad73b352877e51450790b3/flatpak-builder/index.js)
uses `.flatpak-builder` as its default state/cache path and calls cache save
inside the action after builder execution, subject to `save-cache` and exact-hit
conditions. This repository preserves the same action pin, paths and guards.

The added step runs after that action, before the SDK ccache reporting/pruning
step. It records job status and GNU `du` apparent bytes, to directory depth two,
without following symlinks. It changes no cache contents. A missing state path
is explicitly marked absent; a failing build's inventory is not qualified as
a successful warm build. The report has a two-minute timeout.

These are filesystem observations after the build. They are **not** compressed
archive sizes, guaranteed reclaimable space, safe deletion candidates, or a
claim that a branch/exact-hit run published this state. GNU du counts shared
inodes once; per-directory attribution must be interpreted accordingly.

## Local RED → GREEN

RED `bc176219a` fails because the workflow has no inventory step. The GREEN
fixture executes the actual workflow shell against known downloads, object,
compiler and build directories. It verifies reported categories, unchanged
files/timestamps and directory inventory, no outside-symlink traversal, and
explicit absent-state reporting without creating a cache directory.

## Defined hosted measurement

Launch the existing full manual `ci.yml` workflow once on the pushed GREEN
source, with `native-cache-probe=false`. This preserves its complete build/test
selection and adds no new workflow or dispatch mode. The planned Flatpak job
must actually restore a master seed, build/test successfully and emit the
inventory. Preserve the run/source/job IDs and raw report before interpreting
sizes. A PR fast gate alone does not execute or qualify this full job.

No cache pruning, smaller Flatpak reservation, workflow rerun, packaged-content
change, or representative-day speedup is part of this unit. A later experiment
can target specific redundant state only if the measured contents and actual
module reuse support it. Keep watcher cancellation disabled for the intentional
full measurement, so an independent failure cannot silently erase its evidence.

Hosted measurement remains pending. Global allocation and native architecture
retention are unresolved under #1269; this is the only selected issue.
