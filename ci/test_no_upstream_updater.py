#!/usr/bin/env -S uv run --script
"""The macOS build must not use upstream OBS Studio's Sparkle update feed.

Usage:
    uv run python -m unittest ci/test_no_upstream_updater.py

The presets set SPARKLE_APPCAST_URL and SPARKLE_PUBLIC_KEY, and
frontend/cmake/feature-sparkle.cmake turns the updater on when both are set.
Upstream's feed and key made OBS-Studio-Rust offer, and accept, upstream OBS
Studio as an update. obs-rust publishes no appcast of its own, so the updater
stays off until it does, with its own feed and key.
"""

import json
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


if __name__ == "__main__":
    unittest.main()
