#!/usr/bin/env -S uv run --script
"""Fail a Tier 2 ctest log in which a test that must run was Skipped (#776).

    check_tier2_skips.py CTEST_LOG

The Linux harness image once had no PipeWire daemon, dbus-daemon or X server,
so every PipeWire test exited 77 and ctest reported it as Skipped, which the
lane counted as a pass. Tests matching MUST_RUN may not skip.
"""

from __future__ import annotations

import re
import sys

# Tests the harness must really run: they need the PipeWire/D-Bus/X services
# the Tier 2 image provides.
MUST_RUN = re.compile(r"^(test_pipewire_|test_vlc_|test_screencast_portal_)")
SKIPPED = re.compile(r"Test\s+#\d+:\s+(\S+)\s+\.+\s*(?:\*\*\*)?Skipped")


def skipped_must_run(log: str) -> list[str]:
    return sorted({m.group(1) for m in SKIPPED.finditer(log) if MUST_RUN.search(m.group(1))})


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("usage: check_tier2_skips.py CTEST_LOG", file=sys.stderr)
        return 2
    with open(argv[1], encoding="utf-8", errors="replace") as f:
        bad = skipped_must_run(f.read())
    if bad:
        print("FAIL: Tier 2 skipped tests that must run (obs-rust/obs-studio#776):")
        for name in bad:
            print(f"  {name}")
        print("The harness image must provide pipewire, dbus-daemon, xvfb and xauth, and ctest must run under xvfb-run.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
