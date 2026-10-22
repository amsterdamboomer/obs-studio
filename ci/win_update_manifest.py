#!/usr/bin/env -S uv run --script
"""Write obs-rust's signed Windows update manifest (#1736).

    uv run ci/win_update_manifest.py --zip OBS-Studio-Rust-<ver>-windows-x64.zip \\
        --version <ver> --commit <sha> --notes-url <url> --key rsa.pem --out feed/

Reads the release's Windows zip and writes, for the `update-feed` release:

- manifest.json: the schema of frontend/updater/manifest.hpp. One "core"
  package lists every file with its 160-bit BLAKE2b hash and size. Files are
  listed uncompressed (empty compressed_hash), so updater.exe downloads
  <base>/<branch>/core/<name> without the .zst suffix and checks the raw hash.
- manifest.json.sig: hex RSA-SHA512 (PKCS#1 v1.5) signature, the detached
  form FetchAndVerifyFile reads from <url>.sig (#1730).
- updater.exe and updater.exe.sig, when the build ships bin/64bit/updater.exe
  (only builds with ENABLE_WIN_UPDATER do).

Per-file payloads are not published here: a GitHub release holds too few
assets and no folders for the updater's <base>/<branch>/core/<name> layout.
See #1736 for what remains before ENABLE_WIN_UPDATER can default to ON.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import zipfile
from pathlib import Path

UPDATER = "bin/64bit/updater.exe"
# frontend/updater/updater.cpp.hpp IsSafeFilename.
SAFE_NAME = re.compile(r"^(?!/)(?!.*\.\.)[A-Za-z0-9._/-]+$")
VERSION = re.compile(r"(\d+)\.(\d+)\.(\d+)(?:-(beta|rc)\.?(\d+)?)?")


def parse_version(text: str) -> tuple[int, int, int, int, int]:
    """(major, minor, patch, beta, rc) as the uint8 fields of the manifest."""
    match = VERSION.search(text)
    if not match:
        raise ValueError(f"no version in {text!r}")
    major, minor, patch = (int(match.group(i)) for i in (1, 2, 3))
    kind, number = match.group(4), match.group(5)
    # Daily builds carry a date (beta.20261020), too large for uint8; count them as beta 1.
    n = int(number) if number and int(number) < 256 else 1
    beta = n if kind == "beta" else 0
    rc = n if kind == "rc" else 0
    for value in (major, minor, patch):
        if value > 255:
            raise ValueError(f"version {text!r} does not fit the manifest's uint8 fields")
    return major, minor, patch, beta, rc


def blake2_160(path: Path) -> str:
    digest = hashlib.blake2b(digest_size=20)
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def extract(archive: Path, dest: Path) -> Path:
    """Unpack the release zip, refusing entries that escape dest."""
    dest.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive) as zf:
        for name in zf.namelist():
            if name.startswith("/") or ".." in Path(name).parts:
                raise ValueError(f"unsafe zip entry {name!r}")
        zf.extractall(dest)
    entries = [p for p in dest.iterdir()]
    if len(entries) == 1 and entries[0].is_dir() and not (dest / "bin").exists():
        return entries[0]
    return dest


def find_updater(root: Path) -> Path | None:
    path = root / UPDATER
    return path if path.is_file() else None


def build_manifest(root: Path, version: str, commit: str, notes: str) -> dict:
    major, minor, patch, beta, rc = parse_version(version)
    files = []
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        name = path.relative_to(root).as_posix()
        if name == UPDATER:
            continue  # served on its own; the running updater can't replace itself
        if not SAFE_NAME.match(name):
            raise ValueError(f"updater.exe refuses the file name {name!r}")
        files.append({"compressed_hash": "", "hash": blake2_160(path), "name": name, "size": path.stat().st_size})
    return {
        "packages": [{"name": "core", "files": files, "removed_files": []}],
        "version_major": major,
        "version_minor": minor,
        "version_patch": patch,
        "beta": beta,
        "rc": rc,
        "commit": commit,
        "vc2019_redist_x64": "",
        "notes": notes,
    }


def sign(path: Path, key: Path) -> Path:
    """Write <path>.sig: hex RSA-SHA512 PKCS#1 v1.5, as CheckDataSignature expects."""
    result = subprocess.run(
        ["openssl", "dgst", "-sha512", "-sign", str(key), str(path)], check=True, capture_output=True
    )
    sig = path.with_name(path.name + ".sig")
    sig.write_text(result.stdout.hex(), encoding="ascii")
    return sig


def write_feed(root: Path, out: Path, version: str, commit: str, notes: str, key: Path) -> list[Path]:
    out.mkdir(parents=True, exist_ok=True)
    manifest = out / "manifest.json"
    manifest.write_text(json.dumps(build_manifest(root, version, commit, notes), indent=1) + "\n", encoding="utf-8")
    written = [manifest, sign(manifest, key)]
    updater = find_updater(root)
    if updater:
        copy = out / "updater.exe"
        shutil.copyfile(updater, copy)
        written += [copy, sign(copy, key)]
    return written


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--zip", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--notes-url", default="")
    parser.add_argument("--key", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--work", type=Path, default=Path("win-update-work"))
    args = parser.parse_args(argv)
    notes = f'<p><a href="{args.notes_url}">Release notes</a></p>' if args.notes_url else ""
    root = extract(args.zip, args.work)
    for path in write_feed(root, args.out, args.version, args.commit, notes, args.key):
        print(path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
