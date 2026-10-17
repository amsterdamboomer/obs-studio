#!/usr/bin/env -S uv run --script
"""Read the canonical native source inventory for Tier 2 graph validation."""
from pathlib import Path


def ported_objects(root: Path, host: str, hevc: bool = True) -> list[str]:
    return []
