# Isolated pruned-state application rebuild (#1269)

The prior single capacity experiment (39722540093, #1712) completed
SUCCESS. Its canonical compression proxy fell from 2,703,989,420 bytes to
1,624,465,624 bytes when omitting downloads and compiler state. Fresh
source acquisition took 26.691 seconds. These observations do not establish
actual transport savings or a successful application rebuild from that state.

This default-off manual probe copies retained builder state into an isolated
temporary directory, omitting top-level downloads and ccache. It never
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
JSON, hashes and terminal selected-platform results. No hosted result yet.

No production cache writer, key, path or limit changes are adopted. A later
production transport migration needs matching restore/save identities and
actual trusted producer/consumer evidence. Global allocation, writer headroom,
independent native architecture retention and representative durability remain
unresolved under #1269. This does not run a packaged bundle or saved user
profile and does not claim runtime validation or end-to-end speedup.
