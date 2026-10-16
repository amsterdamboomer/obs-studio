#!/usr/bin/env -S uv run --script
"""Report a merge-queue run's wall time and rebuilds in its job summary (#930 E1)."""

from __future__ import annotations


def attempt(runs: list[dict], current: dict) -> tuple[int, list[int]]:
    raise NotImplementedError


def summary(current: dict, jobs: list[dict], att: tuple[int, list[int]], now: str) -> str:
    raise NotImplementedError
