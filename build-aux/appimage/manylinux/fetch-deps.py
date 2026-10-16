#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# ///
"""Install the prebuilt glibc 2.28 build dependencies (/opt/obs-deps).

    uv run build-aux/appimage/manylinux/fetch-deps.py [slice.json] [--prefix /opt/obs-deps]

The dependencies (Qt 6, the FFmpeg stack, PipeWire, ...) are built once by
deps/*.sh (Dockerfile.deps) and published by obs-rust/obs-studio-libs as a
tarball, catalogued in a zackees/manifest.json Catalog. This repository pins
one release of it as an EmbeddedSlice (obs-deps.json next to this script):
the asset's URLs, size and sha256. The tarball is downloaded from the first
URL that works, verified, and extracted into --prefix, which must be the
prefix it was built for (pkg-config and CMake files hold absolute paths).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import tarfile
import tempfile
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent


def fetch(urls: list[str], dest: Path) -> str:
    errors = []
    for url in urls:
        try:
            with urllib.request.urlopen(url, timeout=60) as r, dest.open("wb") as f:
                shutil.copyfileobj(r, f, 1 << 20)
            return url
        except OSError as e:  # URLError and HTTPError are OSErrors
            errors.append(f"{url}: {e}")
    raise SystemExit("could not download the dependencies:\n  " + "\n  ".join(errors))


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("slice", nargs="?", type=Path, default=HERE / "obs-deps.json")
    p.add_argument("--prefix", type=Path, default=Path("/opt/obs-deps"))
    args = p.parse_args(argv)

    doc = json.loads(args.slice.read_text())
    if doc.get("kind") != "EmbeddedSlice" or doc.get("schema_version") != 1:
        raise SystemExit(f"{args.slice}: not a manifest.json EmbeddedSlice (schema_version 1)")
    asset = doc["asset"]
    print(f"obs-deps {doc['compiled_version']} ({asset['filename']}, {asset['size_bytes']} bytes)")

    with tempfile.TemporaryDirectory(prefix="obs-deps-") as tmp:
        archive = Path(tmp) / asset["filename"]
        url = fetch(asset["urls"], archive)
        size = archive.stat().st_size
        digest = sha256(archive)
        if size != asset["size_bytes"] or digest != asset["sha256"]:
            raise SystemExit(f"{url}: got {size} bytes, sha256 {digest}; "
                             f"expected {asset['size_bytes']} bytes, sha256 {asset['sha256']}")
        args.prefix.mkdir(parents=True, exist_ok=True)
        with tarfile.open(archive) as tar:
            # The archive holds the prefix's contents (./lib, ./include, ...).
            tar.extractall(args.prefix, filter="tar")
    marker = args.prefix / ".obs-deps-version"
    marker.write_text(f"{doc['compiled_version']} {asset['sha256']}\n")
    print(f"installed into {args.prefix} from {url}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
