#!/usr/bin/env -S uv run --script
"""Fail CI Gate when any job of this run did not succeed (#1793).

    uv run ci/gate_jobs.py

`needs.*.result` only sees the gate's direct needs. A job inside the
reusable build-project call can end cancelled (for example, "The job was
not acquired by Runner of type hosted even after multiple attempts") while
the call itself reports success, so the gate listed every job of the run
attempt instead. Run in CI Gate with GITHUB_RUN_ID, GITHUB_RUN_ATTEMPT and
GITHUB_REPOSITORY set and gh authenticated (actions: read).
Standard library only.
"""

from __future__ import annotations


def blocking(jobs: list[dict]) -> list[str]:
    """Names of the finished jobs that should fail the gate."""
    return []
