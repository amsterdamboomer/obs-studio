#!/usr/bin/env -S uv run --script
"""Ratchet for platform conditionals in shared C/C++ files (#847, #848).
STUB: the real implementation lands in the GREEN commit.
"""
from __future__ import annotations

import sys


def count_platform_conditionals(text: str) -> list[tuple[int, str]]:
    return []


def is_platform_path(path: str) -> bool:
    return False


def in_scope(path: str) -> bool:
    return False


def compare(counts: dict[str, int], baseline: dict[str, int]) -> tuple[list[str], list[str]]:
    return [], []


def main(argv: list[str]) -> int:
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
