#!/usr/bin/env -S uv run --script
"""Tests for ci/win_update_manifest.py, the Windows update manifest writer (#1736).

    uv run python -m unittest ci/test_win_update_manifest.py

Signing runs openssl against a throwaway RSA key made per test run, never the
release key.
"""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import win_update_manifest as wum  # noqa: E402

HAVE_OPENSSL = shutil.which("openssl") is not None


def b2(data: bytes) -> str:
    return hashlib.blake2b(data, digest_size=20).hexdigest()


class ManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.root = self.tmp / "root"
        files = {
            "bin/64bit/obs64.exe": b"exe",
            "core/obs-filters/obs-filters.dll": b"dll",
            "data/obs-studio/themes/Yami.ovt": b"",
        }
        for name, data in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        self.files = files

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def test_parse_version(self) -> None:
        self.assertEqual(wum.parse_version("0.1.0-beta.20261020"), (0, 1, 0, 1, 0))
        self.assertEqual(wum.parse_version("obs-studio-rust-v1.2.3"), (1, 2, 3, 0, 0))
        self.assertEqual(wum.parse_version("1.2.3-rc4"), (1, 2, 3, 0, 4))
        self.assertEqual(wum.parse_version("1.2.3-beta2"), (1, 2, 3, 2, 0))
        with self.assertRaises(ValueError):
            wum.parse_version("nope")

    def test_manifest_lists_every_file_with_blake2_hash_and_size(self) -> None:
        manifest = wum.build_manifest(self.root, "0.1.0-beta.20261020", "a" * 40, "notes")
        self.assertEqual(
            (manifest["version_major"], manifest["version_minor"], manifest["version_patch"]), (0, 1, 0)
        )
        self.assertEqual(manifest["commit"], "a" * 40)
        self.assertEqual(manifest["notes"], "notes")
        self.assertEqual(manifest["vc2019_redist_x64"], "")
        [package] = manifest["packages"]
        self.assertEqual(package["name"], "core")
        self.assertEqual(package["removed_files"], [])
        self.assertEqual(
            package["files"],
            [
                {"compressed_hash": "", "hash": b2(data), "name": name, "size": len(data)}
                for name, data in sorted(self.files.items())
            ],
        )
        # Every key the C++ Manifest/Package/File structs require is present.
        for key in ("packages", "version_major", "version_minor", "version_patch", "beta", "rc", "commit"):
            self.assertIn(key, manifest)

    def test_updater_exe_is_left_out_of_the_file_list(self) -> None:
        (self.root / "bin/64bit/updater.exe").write_bytes(b"up")
        names = [f["name"] for f in wum.build_manifest(self.root, "1.0.0", "b" * 40, "")["packages"][0]["files"]]
        self.assertNotIn("bin/64bit/updater.exe", names)
        self.assertEqual(wum.find_updater(self.root), self.root / "bin/64bit/updater.exe")

    def test_unsafe_names_are_rejected(self) -> None:
        # updater.cpp.hpp IsSafeFilename refuses anything but [A-Za-z0-9._/-].
        (self.root / "data/bad name.ini").write_bytes(b"x")
        with self.assertRaises(ValueError):
            wum.build_manifest(self.root, "1.0.0", "c" * 40, "")

    def test_extract_strips_a_single_top_folder(self) -> None:
        archive = self.tmp / "OBS-Studio-Rust-1.0.0-windows-x64.zip"
        with zipfile.ZipFile(archive, "w") as zf:
            zf.writestr("bin/64bit/obs64.exe", b"exe")
            zf.writestr("data/x.ini", b"x")
        out = wum.extract(archive, self.tmp / "out")
        self.assertTrue((out / "bin/64bit/obs64.exe").is_file())
        evil = self.tmp / "evil.zip"
        with zipfile.ZipFile(evil, "w") as zf:
            zf.writestr("../escape.txt", b"x")
        with self.assertRaises(ValueError):
            wum.extract(evil, self.tmp / "evil")

    @unittest.skipUnless(HAVE_OPENSSL, "openssl not installed")
    def test_signature_is_hex_rsa_sha512_the_client_verifies(self) -> None:
        key = self.tmp / "key.pem"
        pub = self.tmp / "pub.pem"
        subprocess.run(["openssl", "genrsa", "-out", str(key), "2048"], check=True, capture_output=True)
        subprocess.run(
            ["openssl", "rsa", "-in", str(key), "-pubout", "-out", str(pub)], check=True, capture_output=True
        )
        out = self.tmp / "feed"
        written = wum.write_feed(self.root, out, "1.0.0", "d" * 40, "", key)
        self.assertEqual(sorted(p.name for p in written), ["manifest.json", "manifest.json.sig"])
        json.loads((out / "manifest.json").read_text(encoding="utf-8"))
        hexsig = (out / "manifest.json.sig").read_text(encoding="ascii")
        self.assertRegex(hexsig, r"^[0-9a-f]+$")
        raw = self.tmp / "sig.bin"
        raw.write_bytes(bytes.fromhex(hexsig))
        verify = subprocess.run(
            ["openssl", "dgst", "-sha512", "-verify", str(pub), "-signature", str(raw), str(out / "manifest.json")],
            capture_output=True,
        )
        self.assertEqual(verify.returncode, 0, verify.stderr)
        # A tampered manifest fails.
        (out / "manifest.json").write_text("{}", encoding="utf-8")
        verify = subprocess.run(
            ["openssl", "dgst", "-sha512", "-verify", str(pub), "-signature", str(raw), str(out / "manifest.json")],
            capture_output=True,
        )
        self.assertNotEqual(verify.returncode, 0)

    @unittest.skipUnless(HAVE_OPENSSL, "openssl not installed")
    def test_updater_exe_is_published_signed_when_the_build_has_it(self) -> None:
        (self.root / "bin/64bit/updater.exe").write_bytes(b"up")
        key = self.tmp / "key.pem"
        subprocess.run(["openssl", "genrsa", "-out", str(key), "2048"], check=True, capture_output=True)
        written = wum.write_feed(self.root, self.tmp / "feed", "1.0.0", "e" * 40, "", key)
        self.assertEqual(
            sorted(p.name for p in written),
            ["manifest.json", "manifest.json.sig", "updater.exe", "updater.exe.sig"],
        )


if __name__ == "__main__":
    unittest.main()
