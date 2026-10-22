# Flatpak cache fallback migration (#1269)

The pinned v6.8 action appends `-x86_64` to our custom key and restores
with `flatpak-builder-x86_64` as its fallback prefix. The old custom key
put the manifest hash before the architecture, so changed manifests could
not find the retained seed. #1703 records the original mismatch and the
real seed's six reused dependency modules.

Keep the existing original-manifest hash as the identity input, but put
`x86_64` before it. The action still appends the architecture; this repeated
suffix is intentional and matches its unmodified implementation.

Before the action, perform a lookup-only query using its exact primary and
fallback prefix. Only when neither matches, restore the exact measured legacy
master key. The bridge precedes the independent compiler-cache restore, so
that newer compiler snapshot overlays the old builder snapshot. The normal
action then restores/publishes in the new namespace. Publication is still
master-only, with unchanged action pin, build flags and module validation.
The lookup's matched-key output, including fallback hits, prevents downloading
a legacy archive when a new-namespace archive is already available.

The pinned [cache restore implementation](https://github.com/actions/cache/blob/27d5ce7f107fe9357f9df03efb73ab90386fccae/src/restoreImpl.ts)
passes lookupOnly to the cache API and records matched-key state on a hit.
Its [restore-only state provider](https://github.com/actions/cache/blob/27d5ce7f107fe9357f9df03efb73ab90386fccae/src/stateProvider.ts)
exposes that state as `cache-matched-key`. Do not use `cache-hit` to decide
migration: a useful prefix hit reports false for exact-key equality.

The [pinned-body probe](2026-10-22-flatpak-migration-probe.cjs) and
[results](2026-10-22-flatpak-migration-probe.json) demonstrate a miss before
publication, an exact legacy bridge candidate, and successful exact and
changed-manifest matching after a **synthetic** publication. Reproduce with
the same action download/cache listing inputs as #1703's fallback probe.
This proves the key contract locally, not real cache transport or reuse.

RED `297fd4b45` fails both focused namespace/migration tests. GREEN keeps the
lookup and action keys identical and preserves a literal family prefix,
which the existing cache policy validator requires.

## Hosted validation plan

After pushing GREEN, launch exactly one full manual run on that immutable
source with native-cache-probe=false. Confirm real legacy restore, successful
builder/module reuse and all selected validation. This branch run cannot
publish a master-only seed; observe a later trusted producer and an actual
new-namespace consumer before declaring migration complete. Preserve the
run/source/job IDs and logs. Do not cancel the intentional full run merely
because the PR head changes or a separate gate fails.

No pruning, quota decrease, compressed-byte saving, runtime speedup or
representative-day retention is claimed. First trusted publication temporarily
requires both namespaces until ordinary janitor/eviction retires the old one;
the current inventory's free space is a snapshot, not a future guarantee.
