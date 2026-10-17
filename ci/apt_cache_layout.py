#!/usr/bin/env -S uv run --script
"""Normalize the existing container-produced apt archive on a host runner.

Actions cache stores paths relative to GITHUB_WORKSPACE. The Tier 2
container uses /__w/<repo>/<repo>, so /var/cache/apt/archives is recorded
as ../../../var/cache/apt/archives. On a hosted VM that lands under
/home/runner/var instead. Copy only package archives into apt's real path.
"""
import os
import shutil
from pathlib import Path


def restored_archive_path(workspace: Path) -> Path:
    return (workspace / '../../../var/cache/apt/archives').resolve()


def normalize_archives(workspace: Path, destination: Path) -> int:
    source = restored_archive_path(workspace)
    if source == destination.resolve() or not source.is_dir():
        return 0
    destination.mkdir(parents=True, exist_ok=True)
    copied = 0
    for package in source.glob('*.deb'):
        target = destination / package.name
        if not target.exists():
            shutil.copy2(package, target)
            copied += 1
    return copied


if __name__ == '__main__':
    destination = Path('/var/cache/apt/archives')
    copied = normalize_archives(Path(os.environ['GITHUB_WORKSPACE']), destination)
    packages = list(destination.glob('*.deb'))
    print(f'apt archive layout: copied={copied} usable={len(packages)} bytes={sum(p.stat().st_size for p in packages)}')
