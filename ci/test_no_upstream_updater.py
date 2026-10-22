#!/usr/bin/env -S uv run --script
"""OBS-Studio-Rust must not take updates from upstream OBS Studio.

Usage:
    uv run python -m unittest ci/test_no_upstream_updater.py

The presets set SPARKLE_APPCAST_URL and SPARKLE_PUBLIC_KEY, and
frontend/cmake/feature-sparkle.cmake turns the updater on when both are set.
Upstream's feed and key made OBS-Studio-Rust offer, and accept, upstream OBS
Studio as an update. obs-rust publishes no appcast of its own, so the updater
stays off until it does, with its own feed and key.

Windows (#1729): frontend/utility/AutoUpdateThread.cpp.hpp and
frontend/updater/ fetch and install upstream's signed manifests from
obsproject.com. frontend/cmake/os-windows.cmake builds them only when the
ENABLE_WIN_UPDATER option is on, and it defaults to OFF; shared frontend code
starts update checks on ENABLE_WIN_UPDATER, not on _WIN32.
"""

import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UPSTREAM_KEY = "HQ5/Ba9VHOuEWaM0jtVjZzgHKFJX9YTl+HNVpgNF0iM="


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
