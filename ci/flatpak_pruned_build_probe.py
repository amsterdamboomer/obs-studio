#!/usr/bin/env -S uv run --script
"""Opt-in isolated rebuild without downloaded sources or compiler cache."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile
import time

MARKER = 'OBS_PRUNED_REBUILD_EXECUTED'
EXCEPTIONS = Path(__file__).resolve().parent.parent / '.github/actions/flatpak-builder-lint/exceptions.json'


def run(command, seconds):
    bounded = ['timeout', '--kill-after=10s', str(seconds) + 's', *map(str, command)]
    print('+ ' + shlex.join(bounded), file=sys.stderr)
    result = subprocess.run(bounded, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    print(result.stdout, end='', file=sys.stderr)
    result.check_returncode()
    return result.stdout


def measure(state, manifest, temp_root, execute=run):
    state = Path(state).resolve(strict=True)
    manifest = Path(manifest).resolve(strict=True)
    scratch = Path(temp_root).resolve(strict=True)
    if scratch == state or state in scratch.parents:
        raise ValueError('temporary state must be outside original state')
    recipe = json.loads(manifest.read_text())
    modules = recipe['modules']
    if not modules or modules[-1].get('name') != 'obs':
        raise ValueError('expected final application module obs')
    binary = recipe['command']
    if not isinstance(binary, str) or not binary or Path(binary).name != binary:
        raise ValueError('expected application command basename')
    dependencies = [module['name'] for module in modules[:-1]]
    modules[-1].setdefault('post-install', []).extend([
        'test -x ' + shlex.quote('/app/bin/' + binary),
        "printf '%s\\n' " + MARKER,
    ])
    # Only the dependency OSTree repository is a transport input. Build/checksum
    # scaffolding is regenerated and may contain transient checkout links.
    retained = [state / 'cache']
    if not (state / 'cache').is_dir():
        raise ValueError('expected populated dependency cache')
    for path in retained:
        if path.is_symlink() or (path.is_dir() and any(child.is_symlink() for child in path.rglob('*'))):
            links = [path] if path.is_symlink() else [child for child in path.rglob('*') if child.is_symlink()]
            relative = links[0].relative_to(state).as_posix()
            raise ValueError('retained dependency cache contains symbolic link: ' + relative[:250])
    # Keeping the temporary manifest beside its source preserves relative paths.
    # Both contexts clean up even when a build or validation fails.
    with tempfile.TemporaryDirectory(prefix='flatpak-pruned-build-', dir=scratch) as directory:
        with tempfile.NamedTemporaryFile(mode='w', prefix='flatpak-pruned-', suffix='.json', dir=manifest.parent) as prepared:
            json.dump(recipe, prepared)
            prepared.flush()
            root = Path(directory)
            isolated = root / 'state'
            isolated.mkdir()
            for source in retained:
                destination = isolated / source.name
                if source.is_dir():
                    shutil.copytree(source, destination)
                else:
                    shutil.copy2(source, destination)
            build, repo = root / 'build', root / 'repo'
            start = time.monotonic()
            output = execute([
                'xvfb-run', '--auto-servernum', 'flatpak-builder',
                '--disable-rofiles-fuse', '--install-deps-from=flathub', '--force-clean',
                '--default-branch=master', '--arch=x86_64', '--ccache',
                '--repo=' + str(repo), '--state-dir', str(isolated), str(build), prepared.name,
            ], 1800)
            elapsed = time.monotonic() - start
            if MARKER not in output.splitlines() or 'Cache hit for obs, skipping build' in output:
                raise ValueError('application rebuild signal missing or application was cached')
            for name in dependencies:
                if 'Cache hit for ' + name + ', skipping build' not in output:
                    raise ValueError('dependency module reuse missing: ' + name)
            for relative in (
                'bin/' + binary,
                'share/app-info/icons/flatpak/128x128/' + recipe['id'] + '.png',
                'share/app-info/xmls/' + recipe['id'] + '.xml.gz',
            ):
                if not (build / 'files' / relative).is_file():
                    raise ValueError('isolated output missing: ' + relative)
            for kind, path in (
                ('appstream', build / 'files/share/metainfo' / (recipe['id'] + '.metainfo.xml')),
                ('builddir', build), ('repo', repo),
            ):
                execute(['flatpak-builder-lint', '--exceptions', '--user-exceptions', str(EXCEPTIONS), kind, str(path)], 120)
    return {
        'status': 'success', 'architecture': 'x86_64', 'application_rebuilt': True,
        'reused_dependency_modules': dependencies, 'isolated_build_seconds': elapsed,
        'retained_state': ['cache'], 'omitted_state': ['downloads', 'ccache', 'transient builder state'],
        'limits': 'One isolated application rebuild and output validation, not runtime/profile validation, '
                  'Actions cache transport savings, representative cost or production adoption. '
                  'Copy/setup and validation excluded from build timer; original state only read (may affect atime).',
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state', type=Path, default=Path('.flatpak-builder'))
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--temp-root', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        result = measure(args.state, args.manifest, args.temp_root)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        args.output.write_text(json.dumps({'status': 'failed', 'error': str(error)}, indent=2) + '\n')
        raise
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
