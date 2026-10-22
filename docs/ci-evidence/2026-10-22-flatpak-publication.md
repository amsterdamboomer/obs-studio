# Trusted Flatpak publication and native retention evidence (#1269)

## Defined measurement

#1705 merged as `9c56136dd87b903a7a1ebd64166647d066d29f26` after PR run
39717552504 and merge-group run39717995754 passed. Its full branch consumer
39716240224 also passed; the earlier report now includes its terminal
platform coverage. That branch could not publish a master-only seed.

Ordinary master pushes never select the full Flatpak lane. After protected
warming Push39716177748 completed, dispatch exactly one full master run:
[39718597929](https://github.com/obs-rust/obs-studio/actions/runs/39718597929),
resolved to the merged source above, with native-cache-probe=false. Keep
normal full validation and trusted writers; no workflow rerun, new workflow,
label override or scope shortcut. Producer and consumer results remain open
until observed; do not infer publication from a successful local key probe.

The [compressed evidence](2026-10-22-flatpak-publication-inputs.json.gz)
preserves complete API snapshots, exact configurations and pinned janitor
replays. Usage and listings are separate non-atomic observations. Before
production: 8,105,008,215 listed bytes, 2,632,410,025 bytes of headroom under
10 GiB. A comparable second 2,747,488,816-byte builder archive alone exceeds
that headroom. Do not call this a solved allocation.

## Observed native publications during the producer

These archives were listed on refs/heads/master at the producer source;
completed native job save excerpts are preserved in the evidence bundle.
They are compressed archive bytes, not compiler working-store limits.

| Architecture | Cache ID | Archive bytes | Job |
| --- | ---: | ---: | ---: |
| Windows x64 | 9317986275 | 1,063,380,057 | 119626666708 |
| Windows ARM64 | 9317963943 | 901,055,569 | 119626666694 |
| macOS ARM64 | 9317980409 | 659,485,652 | 119626666745 |
| macOS x86_64 | 9318013377 | 667,145,814 | 119626666731 |

The four archives total **3,291,067,092 bytes**. Current shared family
allowances are only 838,860,800 Windows bytes and 524,288,000 macOS bytes.
A single protected newest entry can exceed its allowance; the declarations
are not reliable bounds for these four intended architecture seeds.

The pinned janitor's hypothetical +601-second replay of `native_complete`
selects current ARM64 Windows9317963943 and macOS9317980409 for LRU eviction,
while newer other-architecture seeds remain. Capture-time grace still
protects new entries. No delete callback was supplied; this is not an
observed deletion or attribution to GitHub/another actor.

The classifier uses the first matching family prefix. Shapes sum into one
shared footprint; they do not independently protect each architecture's
newest seed. A correction needs separate architecture protection and an
honest combined allocation, without claiming that a larger declaration
creates global capacity. Native 350M working limits remain rejected by the
earlier controlled experiment; no smaller working limit is adopted here.

At native completion the listing held **11,960,220,637 bytes**, 1,222,802,397
above10GiB, while the separately queried usage endpoint reported11,293,074,823.
Do not merge those into an atomic observation, infer eviction actors from
presence/absence, or report a speedup from this unmatched production run.

## Offline replay

The existing audit executes the pinned production janitor with saved input
and no delete callback. Reproduce the architecture selections:

```sh
publication_replay_root=$(safe-mktemp /tmp/obs-publication-replay)
uv run --no-project - "$publication_replay_root" <<'PY'
import gzip
import json
import sys
from pathlib import Path
record = json.loads(gzip.decompress(Path('docs/ci-evidence/2026-10-22-flatpak-publication-inputs.json.gz').read_bytes()))
snapshot = record['snapshots']['native_complete']
root = Path(sys.argv[1])
(root / 'inventory.json').write_text(json.dumps(snapshot['inventory']))
(root / 'ci.toml').write_text(snapshot['ci_toml'])
PY
uv run --no-project docs/ci-evidence/2026-10-21-cache-retention-audit.py --output "$publication_replay_root" --replay
safe-rm -r "$publication_replay_root"
```

## Verification status

- [x] Successful Flatpak job and observed new-namespace cache ID/key/bytes.
- [ ] Actual new-namespace consumer restore, legacy bridge skipped, module reuse,
  build/output validation and full selected coverage. Use one defined branch
  consumer to preserve master-only publication and avoid another writer burst.
- [ ] Final producer/consumer parent results; post-publication inventory captured.
- [ ] Global capacity, native architecture retention and representative durability
  remain under #1269; this report does not close those requirements.

## Observed trusted Flatpak publication

Flatpak job119626666642 completed successfully. The new master entry is
**9318164547**, key
`flatpak-builder-x86_64-7bb71f0a48a885ca4bfe218d34814d0c108cb0ee80df664119c10c6a24945d77-x86_64`,
**2,748,017,078 bytes**. The saved job metadata/log excerpts show the legacy
restore, master save input, six reused dependency modules, OBS build and
existing AppStream/build-directory/repository validations. Input
build-bundle:false; no distributable bundle or saved-profile runtime claim.
The parent full run was still active at this capture. An actual new-namespace
consumer remains unproven.

After publication the listing held11,212,285,251 bytes,474,867,011 above10GiB;
the separate usage query reported11,293,074,823. The bundle preserves missing
before-snapshot IDs. Some families received replacements; missing IDs alone
are not lost families or actor attribution. The old Flatpak namespace ID is
absent and the new seed is present. Both `targets-ccache-*` and `targets-apt-*`
have no entry in this post-publication snapshot, so queue-useful retention
is not established by the successful native/Flatpak publications. Do not
attribute those disappearances to the janitor or GitHub without direct logs.
