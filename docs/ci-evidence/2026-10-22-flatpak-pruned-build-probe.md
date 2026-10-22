# Isolated pruned-state application rebuild (#1269)

The prior single capacity experiment (39722540093, #1712) completed
SUCCESS. Its canonical compression proxy fell from 2,703,989,420 bytes to
1,624,465,624 bytes when omitting downloads and compiler state. Fresh
source acquisition took 26.691 seconds. These observations do not establish
actual transport savings or a successful application rebuild from that state.

This default-off manual probe copies retained builder state into an isolated
temporary directory. The revised candidate retains only the cache dependency
repository, omitting downloads, ccache and transient builder scaffolding. It never
builds in, moves, deletes or prunes original state. Retained symlinks and
scratch inside original state are rejected. Copying may affect original atime.

A temporary prepared manifest beside the original preserves relative source
resolution. It adds an executable-existence check and a unique stdout marker
to the final obs module's post-install commands. This changes that module's
recipe checksum so an unchanged application-module hit cannot masquerade as
a rebuild. The helper requires execution of the marker, rejects an obs cache
hit, and requires cache hits for all unchanged dependency modules. The
original prepared manifest is unchanged and the temporary manifest is removed
on both success and failure. No source, existing C test or product packaging
policy is changed.

The isolated build uses the installed builder/SDK, xvfb, an empty compiler
cache, a separate build directory and a separate repository. It then checks
binary/icon/XML outputs and runs the existing flatpak-builder-lint exception
rules on AppStream, builddir and repo. Existing normal build/output validation
runs first and remains required. Build timeout is 30 minutes, each lint two
minutes, and the optional step 40 minutes. Failures fail the job and produce
a small failed-status JSON; no continue-on-error bypass. Temporary trees are
cleaned by context managers, including command failures.

## Verification and defined trial

RED `54645ed68`: four focused tests fail (three absent-helper errors and
one absent-wiring assertion). GREEN fixtures exercise orchestration with a
simulated command boundary: they verify copied/omitted state, relative source
resolution, required validation, original-manifest preservation, cleanup and
rejection of missing rebuild signals. Fixtures do not claim a real build.

After review, run exactly one full workflow on the pushed GREEN branch with
flatpak-pruned-build-probe=true and native-cache-probe=false. Keep complete
normal selection and validation. This is a distinct changed-state rebuild
experiment, not a retry of the completed capacity measurement. Record source,
commands, actual module reuse/rebuild output, versions, output validation,
JSON, hashes and terminal selected-platform results. The initial hosted result and revised candidate are recorded below.

No production cache writer, key, path or limit changes are adopted. A later
production transport migration needs matching restore/save identities and
actual trusted producer/consumer evidence. Global allocation, writer headroom,
independent native architecture retention and representative durability remain
unresolved under #1269. This does not run a packaged bundle or saved user
profile and does not claim runtime validation or end-to-end speedup.


## Initial hosted failure and revised dependency projection

Run39725882580 on659756295 completed Flatpak job119650969777 with
FAILURE. Its normal Flatpak build and output validation passed, but the
optional probe rejected symbolic links before starting the isolated rebuild.
The [failed record](2026-10-22-flatpak-pruned-build-failed.json) preserves
job metadata, capture-time parent status, failed JSON and log hash/excerpts.
The original generic error did not identify the link location; do not infer
which subtree contained it or claim any pruned rebuild succeeded.

[Upstream flatpak-builder 1.4.9 builder-cache.c](https://github.com/flatpak/flatpak-builder/blob/1.4.9/src/builder-cache.c)
locates the dependency OSTree repository in the cache directory and creates
it in bare-user-only mode. That source matches the installed version label;
it is not a byte identity attestation of the distributed binary.

The revised projection copies only that repository. Transient state is
regenerated, and no transient checkout tree can supply compiler outputs to
the rebuild. Symbolic links in retained dependency state remain rejected;
the error now identifies a bounded relative path. No general permission to
follow links into original state or external paths is introduced. Regression
RED9e503de5e reproduces rejection of a transient-tree link by the original
helper; the revised projection passes while preserving original state.

The revised candidate still requires an actual forced application rebuild,
dependency hits and output validation. After the initial full run is terminal,
one explicitly defined new-source experiment may qualify the changed
projection. Never retry the unchanged workflow or treat the original failure
as successful validation. Production transport/allocation remains unchanged.
