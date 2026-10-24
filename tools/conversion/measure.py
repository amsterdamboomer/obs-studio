#!/usr/bin/env -S uv run --script
"""Measure how much of OBS the hybrid build takes from Rust (#2242).

Counting lines in the tree is misleading: the original C stays for the
libobs-c-reference build and the Tier 3 oracle, so C line counts go up as
ports land. The hybrid build is the honest measure:

- libobs: lines of the implementation files (.c.h, .m.h, ...) that the unity
  groups in libobs/build/source-sets.json compile, against the lines of the
  members of the ported groups, which the hybrid build leaves out for Rust.
  The ported set is rust/tools/ported_sources.py's, the list Tier 2 checks.
- Whole app: the same ported lines over libobs plus our C, C++ and
  Objective-C implementation files in frontend/, plugins/ and shared/.

Excluded from every count (EXCLUDED below): tests, the Tier 3 oracle copies,
vendored third-party code, submodules other than obs-websocket, and the
build/ unity group files (they only include their members). obs-websocket
stays C++ by policy (AGENTS.md) and is reported apart as kept by design.

    uv run tools/conversion/measure.py [--json out.json] [--svg out.svg]
    uv run tools/conversion/measure.py --check conversion.json
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HOSTS = ("linux", "windows", "macos")

# A unity group's member: `#include "libobs/<file>.c.h"` (or .m.h, .cpp.hpp).
MEMBER = re.compile(r'^\s*#include\s+"libobs/([^"\n]+\.(?:c|m|cpp|mm)\.h(?:pp)?)"\s*$', re.MULTILINE)

# Implementation files outside libobs: the build-role names of BUILD.md plus
# plain sources that are not in a unity group yet.
IMPLEMENTATION = (".c.h", ".m.h", ".cpp.hpp", ".mm.hpp", ".c", ".m", ".cpp", ".mm", ".cc")

APP_DIRS = ("frontend", "plugins", "shared")
KEPT_BY_DESIGN = ("plugins/obs-websocket",)
EXCLUDED = (
    "plugins/obs-browser",  # submodule (CEF browser source)
    "deps",  # vendored: blake2, glad, json11, libcaption, libdshowcapture, w32-pthreads
    "rust/obs-c-oracle/legacy",  # Tier 3 oracle copies of ported C
    "test",
)
# Path parts that mark test, generated or vendored code anywhere in APP_DIRS.
EXCLUDED_PARTS = {"tests", "test", "build", "deps", "external", "third_party", "thirdparty", "vendor"}


def _ported_sources():
    path = ROOT / "rust" / "tools" / "ported_sources.py"
    spec = importlib.util.spec_from_file_location("ported_sources", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _count(path: Path) -> int:
    with path.open("rb") as f:
        return sum(1 for _ in f)


def _percent(ported: int, total: int) -> float:
    return round(100 * ported / total, 1) if total else 0.0


def _ratio(total: int, ported: int) -> dict:
    return {"total_lines": total, "ported_lines": ported, "percent": _percent(ported, total)}


def _sets(root: Path) -> dict:
    return json.loads((root / "libobs/build/source-sets.json").read_text())


def _group_members(root: Path, group: str) -> list[str]:
    return MEMBER.findall((root / "libobs" / group).read_text())


def host_members(root: Path, host: str) -> list[str]:
    """Every libobs implementation member the C reference build compiles on `host`."""
    sets = _sets(root)
    names = [n for n in sets if n.startswith("LIBOBS_") and n.endswith("_SOURCES")]
    others = {f"LIBOBS_{h.upper()}" for h in HOSTS if h != host}
    members = set()
    for name in names:
        if any(name.startswith(o + "_") for o in others):
            continue
        # Linux-only options, as ported_sources.py reads them.
        if host != "linux" and any(k in name for k in ("PULSE", "NIX_GIO", "WAYLAND")):
            continue
        for group in sets[name]:
            if "+" in Path(group).name:
                members.update(_group_members(root, group))
    return sorted(members)


def ported_members(root: Path, host: str) -> list[str]:
    """The members the hybrid build leaves out on `host` (Tier 2's list)."""
    objects = _ported_sources().ported_objects(root, host)
    return sorted(o[: -len(".o")] for o in objects)


def _area(member: str) -> str:
    top = member.split("/", 1)[0]
    return top if top in ("util", "media-io", "graphics") else "core"


def _excluded(rel: str) -> bool:
    return any(rel == e or rel.startswith(e + "/") for e in EXCLUDED + KEPT_BY_DESIGN)


def _app_files(root: Path) -> list[Path]:
    files = []
    for top in APP_DIRS:
        base = root / top
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file() or not path.name.endswith(IMPLEMENTATION):
                continue
            rel = path.relative_to(root).as_posix()
            if _excluded(rel) or EXCLUDED_PARTS.intersection(path.relative_to(root).parts[1:-1]):
                continue
            files.append(path)
    return sorted(files)


def _rust_lines(root: Path) -> int:
    total = 0
    for path in sorted((root / "rust").rglob("*.rs")) if (root / "rust").is_dir() else []:
        rel = path.relative_to(root).as_posix()
        parts = path.relative_to(root).parts
        if rel.startswith("rust/obs-c-oracle/") or "tests" in parts or "benches" in parts:
            continue
        total += _count(path)
    return total


def measure(root: Path = ROOT) -> dict:
    lines: dict[str, int] = {}

    def member_lines(member: str) -> int:
        if member not in lines:
            lines[member] = _count(root / "libobs" / member)
        return lines[member]

    hosts = {}
    every, ported = set(), set()
    for host in HOSTS:
        all_m = set(host_members(root, host))
        ported_m = set(ported_members(root, host))
        every |= all_m
        ported |= ported_m
        hosts[host] = _ratio(sum(map(member_lines, all_m)), sum(map(member_lines, ported_m & all_m)))

    areas: dict[str, list[int]] = {}
    for member in sorted(every):
        entry = areas.setdefault(_area(member), [0, 0])
        entry[0] += member_lines(member)
        if member in ported:
            entry[1] += member_lines(member)

    libobs_total = sum(map(member_lines, every))
    libobs_ported = sum(map(member_lines, ported))
    app_total = libobs_total + sum(_count(p) for p in _app_files(root))

    # None: the submodule is not checked out, so its size is unknown.
    kept = {}
    for rel in KEPT_BY_DESIGN:
        base = root / rel
        files = sorted(p for p in base.rglob("*") if p.is_file() and p.name.endswith(IMPLEMENTATION))
        kept[rel] = sum(map(_count, files)) if files else None

    return {
        "libobs": {
            **_ratio(libobs_total, libobs_ported),
            "hosts": hosts,
            "areas": {name: _ratio(*v) for name, v in sorted(areas.items())},
        },
        "app": _ratio(app_total, libobs_ported),
        "kept_by_design": kept,
        "rust_lines": _rust_lines(root),
    }


# ---------------------------------------------------------------------------
# SVG: the flat badge style of docs/images/merge-queue.svg and discussions.svg
# (grey #555 label, coloured value, Verdana 11 with a 1 px text shadow).

# Approximate Verdana 11 px advance widths, as shields.io measures them.
_WIDE = {"%": 11.9, "m": 10.7, "w": 8.9, "M": 9.6, "W": 11.0, " ": 3.9, ".": 4.0, "·": 4.0, "-": 4.6}
_NARROW = set("ijlrtf")


def _text_width(text: str) -> float:
    width = 0.0
    for ch in text:
        if ch in _WIDE:
            width += _WIDE[ch]
        elif ch in _NARROW:
            width += 3.9
        elif ch.isdigit():
            width += 7.0
        elif ch.isupper():
            width += 7.5
        else:
            width += 6.8
    return width


RUST_ORANGE = "#dea584"  # GitHub's colour for Rust
STAMP_GREY = "#9f9f9f"


def render_svg(result: dict, commit: str, date: str) -> str:
    segments = [
        ("rust conversion", "#555", "#fff"),
        (f"libobs {result['libobs']['percent']:.1f}%", RUST_ORANGE, "#333"),
        (f"app {result['app']['percent']:.1f}%", RUST_ORANGE, "#333"),
        (f"{commit[:7]} · {date}", STAMP_GREY, "#fff"),
    ]
    widths = [round(_text_width(text)) + 10 for text, _, _ in segments]
    total = sum(widths)
    label = (
        f"rust conversion: libobs {result['libobs']['percent']:.1f}%, "
        f"app {result['app']['percent']:.1f}% ({commit[:7]}, {date})"
    )

    rects, texts, x = [], [], 0
    for (text, fill, ink), width in zip(segments, widths):
        rects.append(f'    <rect x="{x}" width="{width}" height="20" fill="{fill}"/>')
        mid = x + width / 2
        if ink == "#fff":
            texts.append(f'    <text x="{mid:g}" y="15" fill="#010101" fill-opacity=".3">{text}</text>')
        texts.append(f'    <text x="{mid:g}" y="14" fill="{ink}">{text}</text>')
        x += width
    # Thin dividers between the two orange segments and the stamp.
    return "\n".join(
        [
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{total}" height="20" role="img" aria-label="{label}">',
            f"  <title>{label}</title>",
            '  <linearGradient id="s" x2="0" y2="100%">',
            '    <stop offset="0" stop-color="#bbb" stop-opacity=".1"/>',
            '    <stop offset="1" stop-opacity=".1"/>',
            "  </linearGradient>",
            '  <clipPath id="r">',
            f'    <rect width="{total}" height="20" rx="3" fill="#fff"/>',
            "  </clipPath>",
            '  <g clip-path="url(#r)">',
            *rects,
            f'    <rect x="{widths[0] + widths[1]}" width="1" height="20" fill="#000" fill-opacity=".15"/>',
            f'    <rect width="{total}" height="20" fill="url(#s)"/>',
            "  </g>",
            '  <g text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" '
            'text-rendering="geometricPrecision" font-size="11">',
            *texts,
            "  </g>",
            "</svg>",
            "",
        ]
    )


def _head(root: Path) -> tuple[str, str]:
    commit = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True, check=True
    ).stdout.strip()
    stamp = subprocess.run(
        ["git", "-C", str(root), "show", "-s", "--format=%ct", "HEAD"], capture_output=True, text=True, check=True
    ).stdout.strip()
    date = datetime.fromtimestamp(int(stamp), timezone.utc).strftime("%Y-%m-%d")
    return commit, date


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--json", type=Path, help="write the numbers here")
    parser.add_argument("--svg", type=Path, help="write the badge here")
    parser.add_argument("--check", type=Path, help="fail if this JSON's numbers differ from the tree's")
    args = parser.parse_args()

    result = measure(args.root)
    commit, date = _head(args.root)
    document = {"commit": commit, "date": date, **result}

    if args.check:
        saved = json.loads(args.check.read_text())
        saved = {k: v for k, v in saved.items() if k not in ("commit", "date")}
        if saved != result:
            print(f"measure: {args.check} is stale; rerun tools/conversion/measure.py", file=sys.stderr)
            return 1
        return 0
    if args.json:
        args.json.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n")
    if args.svg:
        args.svg.write_text(render_svg(result, commit, date))
    if not args.json and not args.svg:
        print(json.dumps(document, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
