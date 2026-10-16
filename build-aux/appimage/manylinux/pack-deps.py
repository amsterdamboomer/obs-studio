#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# ///
"""Pack /opt/obs-deps (built by Dockerfile.deps) for obs-rust/obs-studio-libs.

    uv run build-aux/appimage/manylinux/pack-deps.py --version 2026.10.16-1 --out build_appimage

Writes, into --out:
  obs-deps-<version>-linux-x86_64-glibc2.28.tar.xz   the release asset
  obs-deps-<version>.slice.json   the EmbeddedSlice to commit as obs-deps.json
  obs-deps-<version>.release.json the Release entry for the obs-studio-libs Catalog
Both describe the asset as zackees/manifest.json does: filename, size, sha256
and the URL obs-studio-libs serves it from.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import sys
import tarfile
from pathlib import Path

REPO = "obs-rust/obs-studio-libs"
TOOL = "obs-deps"
PLATFORM = {"os": "linux", "arch": "x86_64", "libc": "glibc", "os_version": "glibc2.28"}
CATALOG_URL = f"https://raw.githubusercontent.com/{REPO}/main/assets/{TOOL}/manifest.json"


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--version", required=True, help="release version, e.g. 2026.10.16-1")
    p.add_argument("--prefix", type=Path, default=Path(os.environ.get("PREFIX", "/opt/obs-deps")))
    p.add_argument("--out", type=Path, required=True)
    args = p.parse_args(argv)

    args.out.mkdir(parents=True, exist_ok=True)
    name = f"{TOOL}-{args.version}-linux-x86_64-glibc2.28.tar.xz"
    archive = args.out / name
    with tarfile.open(archive, "w:xz", preset=6) as tar:
        for entry in sorted(args.prefix.iterdir()):
            if entry.name == ".obs-deps-version":
                continue
            tar.add(entry, arcname=entry.name)
    h = hashlib.sha256()
    with archive.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    asset = {
        "filename": name,
        "media_type": "application/x-xz",
        "size_bytes": archive.stat().st_size,
        "sha256": h.hexdigest(),
        "urls": [f"https://github.com/{REPO}/releases/download/{TOOL}-{args.version}/{name}"],
    }
    slice_doc = {
        "kind": "EmbeddedSlice",
        "schema_version": 1,
        "tool": TOOL,
        "compiled_for": {"platform": PLATFORM, "variant": {}, "channel": args.version},
        "compiled_version": args.version,
        "asset": asset,
        "online_url": CATALOG_URL,
    }
    release = {
        "version": args.version,
        "published_at": datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": [{"platform": PLATFORM, "variant": {}, "asset": asset}],
    }
    (args.out / f"{TOOL}-{args.version}.slice.json").write_text(json.dumps(slice_doc, indent=2) + "\n")
    (args.out / f"{TOOL}-{args.version}.release.json").write_text(json.dumps(release, indent=2) + "\n")
    print(f"{archive} {asset['size_bytes']} bytes sha256 {asset['sha256']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
