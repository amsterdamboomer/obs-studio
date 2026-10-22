#!/usr/bin/env -S uv run --script
"""Bounded native compiler-cache archive transport (#1269)."""

from pathlib import Path


def pack(source: Path, output: Path, identity: dict, source_sha: str) -> dict:
    raise NotImplementedError('native cache packaging')


def restore(archive: Path, metadata: dict, destination: Path, identity: dict) -> None:
    raise NotImplementedError('native cache verified restore')
