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
