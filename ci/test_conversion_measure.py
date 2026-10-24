#!/usr/bin/env -S uv run --script
"""tools/conversion/measure.py counts how much of OBS the hybrid build takes
from Rust (#2242).

    uv run python -m unittest ci/test_conversion_measure.py
"""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MEASURE = ROOT / "tools" / "conversion" / "measure.py"
PORTED_SOURCES = ROOT / "rust" / "tools" / "ported_sources.py"


def load(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def lines(n: int) -> str:
    return "".join(f"line {i}\n" for i in range(n))


SETS = {
    "LIBOBS_SOURCES": ["build/libobs.core+.h.c"],
    "LIBOBS_PORTED_SOURCES": ["build/libobs.ported-util+.h.c", "build/libobs.ported-media+.h.c"],
    "LIBOBS_RUST_SOURCES": [],
    "LIBOBS_WINDOWS_SOURCES": [],
    "LIBOBS_WINDOWS_PORTED_SOURCES": [],
    "LIBOBS_MACOS_SOURCES": [],
    "LIBOBS_MACOS_PORTED_SOURCES": [],
    "LIBOBS_LINUX_SOURCES": ["build/libobs.linux+.h.c"],
    "LIBOBS_LINUX_PORTED_SOURCES": [],
    "LIBOBS_HEVC_PORTED_SOURCES": [],
    "LIBOBS_PULSE_PORTED_SOURCES": [],
    "LIBOBS_NIX_GIO_PORTED_SOURCES": [],
    "LIBOBS_WAYLAND_PORTED_SOURCES": [],
    "LIBOBS_WAYLAND_GIO_PORTED_SOURCES": [],
}

# path -> contents. Group files list their members; everything else is a
# counted (or excluded) implementation file of the given length.
FILES = {
    "libobs/build/libobs.core+.h.c": '#include "libobs/build/libobs.pch.h"\n\n#include "libobs/obs.c.h"\n',
    "libobs/build/libobs.linux+.h.c": '#include "libobs/build/libobs.pch.h"\n\n#include "libobs/obs-nix.c.h"\n',
    "libobs/build/libobs.ported-util+.h.c": (
        '#include "libobs/build/libobs-util.pch.h"\n\n#include "libobs/util/dstr.c.h"\n'
    ),
    "libobs/build/libobs.ported-media+.h.c": (
        '#include "libobs/build/libobs-media.pch.h"\n\n#include "libobs/media-io/audio-io.c.h"\n'
    ),
    "libobs/obs.c.h": lines(100),
    "libobs/obs-nix.c.h": lines(20),
    "libobs/util/dstr.c.h": lines(50),
    "libobs/media-io/audio-io.c.h": lines(30),
    # Not in any group: a header-only file is not implementation.
    "libobs/obs.h": lines(999),
    "frontend/widgets/OBSBasic.cpp.hpp": lines(200),
    "frontend/build/frontend.widgets+.hpp.cpp": '#include "frontend/widgets/OBSBasic.cpp.hpp"\n',
    "plugins/obs-ffmpeg/obs-ffmpeg.c.h": lines(40),
    "plugins/mac-capture/mac-capture.m.h": lines(10),
    "shared/ipc-util/ipc-util/pipe.c.h": lines(5),
    # Kept by design: listed, never counted.
    "plugins/obs-websocket/src/WebSocketServer.cpp": lines(300),
    # Excluded from every count.
    "rust/obs-c-oracle/legacy/util/dstr.c": lines(777),
    "test/cmocka/test_dstr.c": lines(888),
    "plugins/obs-ffmpeg/tests/test_mux.c": lines(66),
    "deps/glad/src/glad.c": lines(555),
    "plugins/obs-browser/browser-client.cpp": lines(444),
    "rust/obs-util/tests/strings.rs": lines(333),
    # Rust lines (secondary number).
    "rust/obs-util/src/dstr.rs": lines(70),
    "rust/obs-util/src/lib.rs": lines(5),
}


def make_tree(root: Path) -> None:
    (root / "libobs/build").mkdir(parents=True)
    (root / "libobs/build/source-sets.json").write_text(json.dumps(SETS))
    for rel, text in FILES.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)


class FixtureCountsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.measure = load(MEASURE, "conversion_measure")
        cls.tmp = tempfile.TemporaryDirectory()
        cls.root = Path(cls.tmp.name)
        make_tree(cls.root)
        cls.result = cls.measure.measure(cls.root)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_libobs_counts_group_members_only(self):
        libobs = self.result["libobs"]
        # obs.c.h 100 + obs-nix.c.h 20 + dstr 50 + audio-io 30; obs.h is not a member.
        self.assertEqual(libobs["total_lines"], 200)
        self.assertEqual(libobs["ported_lines"], 80)
        self.assertEqual(libobs["percent"], 40.0)

    def test_libobs_per_host_and_area(self):
        linux = self.result["libobs"]["hosts"]["linux"]
        self.assertEqual((linux["total_lines"], linux["ported_lines"]), (200, 80))
        windows = self.result["libobs"]["hosts"]["windows"]
        self.assertEqual((windows["total_lines"], windows["ported_lines"]), (180, 80))
        areas = self.result["libobs"]["areas"]
        self.assertEqual(areas["util"], {"total_lines": 50, "ported_lines": 50, "percent": 100.0})
        self.assertEqual(areas["media-io"]["ported_lines"], 30)
        self.assertEqual(areas["core"], {"total_lines": 120, "ported_lines": 0, "percent": 0.0})

    def test_whole_app_excludes_tests_oracle_vendored_and_websocket(self):
        app = self.result["app"]
        # libobs 200 + frontend 200 + obs-ffmpeg 40 + mac-capture 10 + ipc-util 5.
        self.assertEqual(app["total_lines"], 455)
        self.assertEqual(app["ported_lines"], 80)
        self.assertEqual(app["percent"], round(100 * 80 / 455, 1))

    def test_websocket_is_kept_by_design(self):
        self.assertEqual(self.result["kept_by_design"], {"plugins/obs-websocket": 300})

    def test_rust_lines_skip_tests_and_oracle(self):
        self.assertEqual(self.result["rust_lines"], 75)

    def test_same_tree_same_numbers(self):
        self.assertEqual(self.measure.measure(self.root), self.result)

    def test_svg_is_text_in_the_badge_style(self):
        svg = self.measure.render_svg(self.result, commit="0123456789abcdef", date="2026-10-24")
        self.assertTrue(svg.startswith("<svg "))
        self.assertNotIn("<image", svg)
        self.assertIn("<text", svg)
        self.assertIn("libobs 40.0%", svg)
        self.assertIn("app 17.6%", svg)
        self.assertIn("0123456", svg)
        self.assertIn("2026-10-24", svg)
        # The README badges' flat style: grey label, Verdana 11.
        self.assertIn('fill="#555"', svg)
        self.assertIn("Verdana,Geneva,DejaVu Sans,sans-serif", svg)


class RealTreeTest(unittest.TestCase):
    def test_readme_and_leaderboard_publish_the_badge(self):
        readme = (ROOT / "README.md").read_text()
        self.assertIn("raw.githubusercontent.com/obs-rust/obs-studio/leaderboard/conversion.svg", readme)
        workflow = (ROOT / ".github/workflows/leaderboard.yml").read_text()
        self.assertIn("tools/conversion/measure.py --json data/conversion.json --svg data/conversion.svg", workflow)

    def test_libobs_ported_set_matches_tier2_list(self):
        measure = load(MEASURE, "conversion_measure")
        ported_sources = load(PORTED_SOURCES, "ported_sources")
        for host in ("linux", "windows", "macos"):
            expected = sorted(o[: -len(".o")] for o in ported_sources.ported_objects(ROOT, host))
            self.assertEqual(measure.ported_members(ROOT, host), expected, host)


if __name__ == "__main__":
    unittest.main()
