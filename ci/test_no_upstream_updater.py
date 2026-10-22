#!/usr/bin/env -S uv run --script
"""OBS-Studio-Rust must not take updates from upstream OBS Studio.

Usage:
    uv run python -m unittest ci/test_no_upstream_updater.py

The presets set SPARKLE_APPCAST_URL and SPARKLE_PUBLIC_KEY, and
frontend/cmake/feature-sparkle.cmake turns the updater on when both are set.
Upstream's feed and key made OBS-Studio-Rust offer, and accept, upstream OBS
Studio as an update. Since #1730 the presets name obs-rust's own feed and key:
publish.yaml writes the appcasts and a signed branches.json as assets of the
fixed `update-feed` GitHub release after each daily release, signed with the
private keys kept in the OBS_RUST_SPARKLE_PRIVATE_KEY and
OBS_RUST_UPDATE_RSA_PRIVATE_KEY repository secrets.

Windows (#1729): frontend/utility/AutoUpdateThread.cpp.hpp and
frontend/updater/ fetch and install upstream's signed manifests from
obsproject.com. frontend/cmake/os-windows.cmake builds them only when the
ENABLE_WIN_UPDATER option is on, and it defaults to OFF; shared frontend code
starts update checks on ENABLE_WIN_UPDATER, not on _WIN32.
"""

import importlib.util
import json
import plistlib
import re
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UPSTREAM_KEY = "HQ5/Ba9VHOuEWaM0jtVjZzgHKFJX9YTl+HNVpgNF0iM="
FEED_PREFIX = "https://github.com/obs-rust/obs-studio/releases/download/update-feed/"
OBS_RUST_SPARKLE_KEY = "x7PaevvHxyhTaZdEFDhjr6+SqKf631gWXuIHowm6D3I="
# First base64 line of obs-rust's RSA-4096 update metadata key (OBSPublicRSAKey.pem).
OBS_RUST_RSA_KEY_LINE = "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAramOrg+JnU+xAYh7WYbU"
UPSTREAM_RSA_KEY_LINE = "MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAl3sverw9HQ+rYQNn9Ca7"


def cache_value(preset: dict, name: str) -> str:
    value = preset.get("cacheVariables", {}).get(name)
    if isinstance(value, dict):
        value = value.get("value")
    return value or ""


class NoUpstreamUpdaterTests(unittest.TestCase):
    def test_presets_do_not_use_upstream_sparkle_feed(self) -> None:
        presets = json.loads((ROOT / "CMakePresets.json").read_text(encoding="utf-8"))
        for preset in presets.get("configurePresets", []):
            with self.subTest(preset=preset.get("name")):
                self.assertNotIn("obsproject.com", cache_value(preset, "SPARKLE_APPCAST_URL"))
                self.assertNotEqual(cache_value(preset, "SPARKLE_PUBLIC_KEY"), UPSTREAM_KEY)

    def test_macos_presets_use_obs_rust_feed_and_key(self) -> None:
        presets = json.loads((ROOT / "CMakePresets.json").read_text(encoding="utf-8"))
        macos = next(p for p in presets["configurePresets"] if p.get("name") == "macos")
        self.assertEqual(cache_value(macos, "SPARKLE_APPCAST_URL"), FEED_PREFIX + "updates_$(ARCHS)_v2.xml")
        self.assertEqual(cache_value(macos, "SPARKLE_PUBLIC_KEY"), OBS_RUST_SPARKLE_KEY)

    def test_macos_sources_use_obs_rust_feed(self) -> None:
        for rel in ("frontend/utility/OBSSparkle.mm.hpp", "frontend/utility/MacUpdateThread.cpp.hpp"):
            text = (ROOT / rel).read_text(encoding="utf-8")
            with self.subTest(file=rel):
                self.assertNotRegex(text, r'"https://[^"]*obsproject\.com')
                self.assertIn(FEED_PREFIX, text)

    def test_update_metadata_key_is_obs_rust(self) -> None:
        pem = (ROOT / "frontend/data/OBSPublicRSAKey.pem").read_text(encoding="utf-8")
        self.assertNotIn(UPSTREAM_RSA_KEY_LINE, pem)
        self.assertIn(OBS_RUST_RSA_KEY_LINE, pem)

    def test_publish_workflow_writes_obs_rust_feed(self) -> None:
        text = (ROOT / ".github/workflows/publish.yaml").read_text(encoding="utf-8")
        self.assertNotRegex(text, r"(urlPrefix|customLink):\s*'?https://[^\s']*obsproject\.com")
        for needle in (
            "secrets.OBS_RUST_SPARKLE_PRIVATE_KEY",
            "secrets.OBS_RUST_UPDATE_RSA_PRIVATE_KEY",
            "gh release upload update-feed",
            "https://github.com/obs-rust/obs-studio/releases/download/",
        ):
            with self.subTest(needle=needle):
                self.assertIn(needle, text)

    def test_packaged_bundle_check_requires_obs_rust_feed(self) -> None:
        # release-rust.yaml's macos_smoke runs `app_identity_check.py bundle` on the
        # app copied out of the released DMG; it must reject upstream's feed and key.
        spec = importlib.util.spec_from_file_location("app_identity_check", ROOT / "ci/app_identity_check.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)

        def bundle(scratch: str, feed: str, key: str) -> Path:
            app = Path(scratch) / "OBS-Studio-Rust.app"
            (app / "Contents/MacOS").mkdir(parents=True)
            (app / "Contents/MacOS/OBS-Studio-Rust").write_bytes(b"")
            (app / "Contents/Frameworks/Sparkle.framework").mkdir(parents=True)
            info = {
                "CFBundleIdentifier": module.APP_ID,
                "CFBundleExecutable": "OBS-Studio-Rust",
                "CFBundleDisplayName": "OBS-Studio-Rust",
                "SUFeedURL": feed,
                "SUPublicEDKey": key,
            }
            with (app / "Contents/Info.plist").open("wb") as stream:
                plistlib.dump(info, stream)
            return app

        with tempfile.TemporaryDirectory() as scratch:
            module.check_bundle(bundle(scratch, FEED_PREFIX + "updates_arm64_v2.xml", OBS_RUST_SPARKLE_KEY))
        for feed, key in (
            ("https://obsproject.com/osx_update/updates_arm64_v2.xml", UPSTREAM_KEY),
            (FEED_PREFIX + "updates_arm64_v2.xml", UPSTREAM_KEY),
            (FEED_PREFIX + "updates_$(ARCHS)_v2.xml", OBS_RUST_SPARKLE_KEY),
            ("", ""),
        ):
            with self.subTest(feed=feed, key=key), tempfile.TemporaryDirectory() as scratch:
                with self.assertRaises(RuntimeError):
                    module.check_bundle(bundle(scratch, feed, key))

    def test_windows_updater_is_off_while_it_points_at_upstream(self) -> None:
        sources = [
            ROOT / "frontend/utility/AutoUpdateThread.cpp.hpp",
            ROOT / "frontend/updater/updater.cpp.hpp",
        ]
        upstream = any(
            re.search(r'"https://[^"]*obsproject\.com', p.read_text(encoding="utf-8")) for p in sources
        )
        if not upstream:
            return
        cmake = (ROOT / "frontend/cmake/os-windows.cmake").read_text(encoding="utf-8")
        option = re.search(r"option\(\s*ENABLE_WIN_UPDATER\b[^)]*\)", cmake)
        self.assertIsNotNone(option, "os-windows.cmake must declare option(ENABLE_WIN_UPDATER ... OFF)")
        self.assertRegex(option.group(0), r"\bOFF\s*\)$")
        gate = cmake.find("if(ENABLE_WIN_UPDATER)")
        self.assertNotEqual(gate, -1, "the updater must be built only under if(ENABLE_WIN_UPDATER)")
        for needle in ("add_subdirectory(updater)", "frontend.windows-autoupdatethread+.hpp.cpp"):
            with self.subTest(needle=needle):
                self.assertNotIn(needle, cmake[:gate], f"{needle} is built before the ENABLE_WIN_UPDATER gate")
        # Shared code starts the updater only under ENABLE_WIN_UPDATER, not on every Windows build.
        for rel in ("frontend/widgets/OBSBasic_Updater.cpp.hpp", "frontend/widgets/OBSBasic_MainControls.cpp.hpp"):
            text = (ROOT / rel).read_text(encoding="utf-8")
            for use in re.finditer(r"new AutoUpdateThread\(", text):
                guards = re.findall(r"^\s*#\s*(?:if|ifdef|elif)\b.*$", text[: use.start()], re.MULTILINE)
                with self.subTest(file=rel, offset=use.start()):
                    self.assertTrue(guards, "AutoUpdateThread is started without a guard")
                    self.assertIn("ENABLE_WIN_UPDATER", guards[-1])

if __name__ == "__main__":
    unittest.main()
