"""Which platforms a path belongs to (#897). Stub for the RED commit."""

from __future__ import annotations

LINUX, MACOS, WINDOWS = "linux", "macos", "windows"


def platforms(path: str) -> frozenset[str] | None:
    raise NotImplementedError
