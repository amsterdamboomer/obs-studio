# Native compiler-cache transport candidate (#1269)

This is preparation for the [recorded transport plan](https://github.com/obs-rust/obs-studio/issues/1269#issuecomment-6335937323),
not a deployed backend or a completed allocation fix. #1715 separately
qualifies an isolated Flatpak rebuild. #1723 remains held; #1269 stays open.

## Archive boundary

`ci/native_cache_transport.py` packages native cache files/directories with
standard-library gzip level1 and tar. Hardlinked regular files are represented
as complete regular payloads; symlinks and special files fail explicitly.
Nothing is silently pruned. The gzip format is a portability candidate whose
actual native transfer/compression cost still needs measurement, not an
asserted improvement over Actions' zstd transport.

Metadata records schema/format, full producer commit, platform, architecture,
configuration, compiler, SDK and cache-format identity, compressed/expanded
bytes, entry count and SHA256. Consumers must supply the expected identity;
changing any identity field rejects the seed. Compiler-cache source freshness
remains the compiler cache's responsibility: matching archive identity alone
does not certify a source/object hit or compiler correctness.

The archive is strictly below2GiB; expanded data is at most4GiB and250,000
entries. These are format limits, not retention reservations or measured
working-store requirements. Archive and metadata limits apply independently.
Extraction verifies size/digest, rejects traversal/links/special entries and
Windows-ambiguous names, and stages all output before exclusively reserving a missing destination.
Verified files are linked into that owned directory; callers consume it only
after successful return, and a failed installation removes owned partial data. Existing output/destination paths are rejected. No network,
release/tag/asset, credentials, cache writer/key/path, budget, workflow or
required build/test lane has changed.

## RED → GREEN

- REDd2003bc0a: six focused cases fail against packaging/restore stubs.
- RED14a08064e: real hardlinked cache content fails packaging; an otherwise
  valid archive containing a Windows device filename is incorrectly accepted.
  This regression uses matching metadata so rejection is not accidentally
  caused by an unrelated entry-count mismatch.
- GREEN local focused signal: ten archive tests plus two stdlib checks pass.
  Covers real tar/gzip round trip, all identity fields, byte corruption,
  existing destination, malicious entries, actual hardlinks, pack/restore
  resource limits and cleanup. No network/compiler/native build is simulated
  as production evidence, and no expensive CI test lane is introduced.

## Remaining qualification before adoption

Integrate the trusted publisher and read-only consumer with immutable
checksummed identities, correct cold fallback, bounded per-architecture
retention/in-flight accounting and an explicit public cache namespace.
Preserve build-job read-only tokens and existing native test coverage. Measure
actual native archives, packing/transfer/extraction, compiler hits/misses and
publisher runner cost, including source/toolchain/configuration changes.
Validate real Windows and both macOS architectures before replacing existing
Actions writers or using this as global-capacity evidence. Actual release
publication/permissions remain a proposed route; no assets have been created.

Review follow-up RED3651d6f66 covers a destination created concurrently at
publication and case-insensitive collisions (including directory components).
Publication now uses exclusive mkdir ownership rather than POSIX rename;
path aliases are rejected on pack and restore.

## Local generation protocol (not hosted publication)

REDde912a329 adds five failing protocol cases. The implementation publishes
an immutable archive first and an immutable generation manifest last. That
manifest is the commit point; there is no mutable latest-pointer replacement.
Read-only consumers discover at most two matching complete generations,
verify downloaded bytes and archive metadata, and report a cold fallback
without installing partial data when no usable seed exists.

A local in-memory release fixture exercises actual archive bytes: round trip,
failed promotion retaining the previous usable seed, two completed retained
generations, missing/incompatible cold fallback, and untrusted context rejected
before external operations. Fifteen archive/protocol cases plus two stdlib
checks pass in0.419s. This is not a GitHub API, publication, native build or
performance measurement.

The proposed external bound is6GiB and6 assets per OS/architecture release,
including incomplete uploads, with two completed steady-state generations and
one publication in flight. Four native architecture releases therefore have a
24GiB hard maximum, independent of the10GiB Actions cache quota. These are
explicit candidate ceilings, not measured need or an adopted storage policy.
No assets/permissions have changed. Orphan recovery, retry reproducibility,
the actual GitHub adapter and workflow/consumer integration remain to be
finished and qualified before this draft is ready.

API references for the adapter design:

- [Release asset size/digest metadata](https://docs.github.com/en/rest/releases/assets).
- [Repository GITHUB_TOKEN event behavior](https://docs.github.com/en/actions/concepts/security/github_token).

Publication must use the explicit repository job token, never an inherited PAT
or custom token that could trigger product tag/release workflows. The actual
trusted job wiring and cache-only release namespace still require verification.

## Retry and abandoned-upload recovery

REDe7fc556c4 demonstrates non-reproducible gzip retry bytes, failed retries,
old orphan uploads exhausting admission, and starter-state manifests hiding a
usable prior seed. Gzip now fixes header timestamp/name while preserving
snapshot file metadata, so an unchanged snapshot produces identical bytes.
A retry verifies/reuses existing immutable assets without clobbering them.

Only uploaded, nonempty, digest-bearing archive/manifest pairs are complete.
A serialized publisher can reclaim incomplete generations after a ten-minute
creation grace period; valid complete pairs and reusable current-generation
assets remain protected. Recent orphans still count against admission limits:
publication refuses when necessary rather than exceeding the bound or
removing a retained usable seed. No background deletion job was added.

Twenty archive/protocol cases plus two stdlib checks pass in0.501s. Coverage
includes reproducibility across different wall times, a no-upload retry,
expired orphan recovery, recent orphan protection/budget refusal, and ignoring
starter manifests while retaining a real usable generation. The fixture is
local; GitHub adapter, actual publication and native qualification remain open.
