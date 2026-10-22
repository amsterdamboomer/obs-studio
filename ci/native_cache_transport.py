#!/usr/bin/env -S uv run --script
"""Bounded native compiler-cache archive transport (#1269).

No network or publication policy lives here. Native cache stores must contain
ordinary files/directories; unsupported entries fail rather than disappearing.
"""

from __future__ import annotations

import hashlib
import os
from pathlib import Path
import re
import shutil
import stat
import tarfile
import tempfile

# GitHub release assets must be strictly below 2 GiB. Expanded/native-store
# and entry limits are separate, explicit safeguards, not storage reservations.
MAX_ARCHIVE_BYTES = (2 << 30) - 1
MAX_EXPANDED_BYTES = 4 << 30
MAX_ENTRIES = 250_000
IDENTITY_FIELDS = {'platform', 'architecture', 'configuration', 'compiler', 'sdk', 'cache_format'}


def digest(stream) -> str:
    value = hashlib.sha256()
    for chunk in iter(lambda: stream.read(1 << 20), b''):
        value.update(chunk)
    return value.hexdigest()


def validate_identity(identity: dict) -> None:
    if not isinstance(identity, dict) or set(identity) != IDENTITY_FIELDS:
        raise ValueError('cache identity fields do not match schema')
    if any(not isinstance(value, str) or not value or len(value) > 2048 for value in identity.values()):
        raise ValueError('cache identity values must be bounded nonempty strings')
    if (identity['platform'], identity['architecture']) not in {
        ('windows', 'x64'), ('windows', 'arm64'), ('macos', 'x86_64'), ('macos', 'arm64'),
    }:
        raise ValueError('unsupported native cache identity')


def safe_name(name: str) -> None:
    # Backslashes, drive prefixes and Windows-special names must not acquire
    # a different meaning when the same archive is restored on Windows.
    parts = name.split('/')
    reserved = re.compile(r'(?:CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(?:\..*)?', re.IGNORECASE)
    if (not name or any(part in ('', '.', '..') or part.endswith(('.', ' ')) or reserved.fullmatch(part) for part in parts)
            or any(char in name for char in '\\:<>"|?*')):
        raise ValueError('unsupported archive path: ' + name[:200])


def reject_case_collision(name: str, aliases: dict) -> None:
    parts = name.split('/')
    for count in range(1, len(parts) + 1):
        prefix = '/'.join(parts[:count])
        key = prefix.casefold()
        if key in aliases and aliases[key] != prefix:
            raise ValueError('case-insensitive archive path collision')
        aliases[key] = prefix


def publish_directory(staged: Path, destination: Path) -> None:
    # mkdir is the portable exclusive ownership operation: POSIX rename can
    # replace an empty directory created by another owner after an exists check.
    try:
        destination.mkdir()
    except FileExistsError as error:
        raise ValueError('destination already exists') from error
    complete = False
    try:
        # Link verified payloads on the same filesystem without copying bytes.
        # The caller may consume the directory only after successful return.
        shutil.copytree(staged, destination, copy_function=os.link, dirs_exist_ok=True)
        complete = True
    finally:
        if not complete:
            shutil.rmtree(destination)


def pack(source: Path, output: Path, identity: dict, source_sha: str) -> dict:
    validate_identity(identity)
    if not isinstance(source_sha, str) or not re.fullmatch('[0-9a-f]{40}', source_sha):
        raise ValueError('producer source_sha must be a complete commit identity')
    source, output = Path(source), Path(output)
    if output.exists() or output.is_symlink():
        raise ValueError('output already exists')
    if source.is_symlink():
        raise ValueError('source link is not supported')
    source = source.resolve(strict=True)
    if not source.is_dir():
        raise ValueError('source must be a directory')
    output = output.absolute()
    if source == output.resolve() or source in output.resolve().parents:
        raise ValueError('output must be outside source')
    entries, expanded, aliases = [], 0, {}
    for directory, dirs, files in os.walk(source, followlinks=False):
        dirs.sort()
        for name in sorted(dirs + files):
            path = Path(directory) / name
            info = path.lstat()
            if stat.S_ISLNK(info.st_mode):
                raise ValueError('source link is not supported: ' + str(path.relative_to(source)))
            if not (stat.S_ISDIR(info.st_mode) or stat.S_ISREG(info.st_mode)):
                raise ValueError('source special file is not supported')
            relative = path.relative_to(source).as_posix()
            safe_name(relative)
            reject_case_collision(relative, aliases)
            expanded += info.st_size if stat.S_ISREG(info.st_mode) else 0
            entries.append((path, relative, info))
            if expanded > MAX_EXPANDED_BYTES or len(entries) > MAX_ENTRIES:
                raise ValueError('native cache expanded size/entry budget exceeded')
    # A private temporary directory ensures failures never publish partial data.
    # Hard-link publication is atomic, same-filesystem, and never clobbers output.
    with tempfile.TemporaryDirectory(prefix='native-cache-pack-', dir=output.parent) as directory:
        staged = Path(directory) / 'seed.tar.gz'
        with tarfile.open(staged, 'w:gz', compresslevel=1, dereference=True) as archive:
            for path, relative, status in entries:
                info = archive.gettarinfo(str(path), arcname=relative)
                if not (info.isdir() or info.isfile()):
                    raise ValueError('source changed type during packaging')
                if info.isfile() and info.size != status.st_size:
                    raise ValueError('source changed size during packaging')
                info.uid = info.gid = 0
                info.uname = info.gname = ''
                info.mode &= 0o777
                if info.isfile():
                    with path.open('rb') as data:
                        archive.addfile(info, data)
                else:
                    archive.addfile(info)
        size = staged.stat().st_size
        if size > MAX_ARCHIVE_BYTES:
            raise ValueError('archive asset size budget exceeded')
        with staged.open('rb') as stream:
            checksum = digest(stream)
        os.link(staged, output)
    return {'schema': 1, 'format': 'tar.gz', 'identity': dict(identity), 'source_sha': source_sha,
            'bytes': size, 'expanded_bytes': expanded, 'entries': len(entries), 'sha256': checksum}


def restore(archive: Path, metadata: dict, destination: Path, identity: dict) -> None:
    validate_identity(identity)
    if not isinstance(metadata, dict) or metadata.get('schema') != 1 or metadata.get('format') != 'tar.gz':
        raise ValueError('unsupported cache metadata schema/format')
    if metadata.get('identity') != identity:
        raise ValueError('cache identity mismatch')
    if not re.fullmatch('[0-9a-f]{40}', str(metadata.get('source_sha', ''))):
        raise ValueError('invalid producer identity')
    for field, limit in (('bytes', MAX_ARCHIVE_BYTES), ('expanded_bytes', MAX_EXPANDED_BYTES), ('entries', MAX_ENTRIES)):
        value = metadata.get(field)
        if type(value) is not int or not 0 <= value <= limit:
            raise ValueError('cache metadata budget exceeded: ' + field)
    if not re.fullmatch('[0-9a-f]{64}', str(metadata.get('sha256', ''))):
        raise ValueError('invalid archive digest')
    destination = Path(destination).absolute()
    if destination.exists() or destination.is_symlink():
        raise ValueError('destination already exists')
    with Path(archive).open('rb') as stream:
        if os.fstat(stream.fileno()).st_size != metadata['bytes']:
            raise ValueError('archive size mismatch')
        if digest(stream) != metadata['sha256']:
            raise ValueError('archive digest mismatch')
        stream.seek(0)
        with tempfile.TemporaryDirectory(prefix='native-cache-restore-', dir=destination.parent) as directory:
            staged = Path(directory) / 'payload'
            staged.mkdir()
            seen, expanded, aliases = set(), 0, {}
            try:
                with tarfile.open(fileobj=stream, mode='r|gz') as payload:
                    for member in payload:
                        safe_name(member.name)
                        reject_case_collision(member.name, aliases)
                        if not (member.isfile() or member.isdir()) or member.name in seen:
                            raise ValueError('unsupported or duplicate archive entry')
                        seen.add(member.name)
                        expanded += member.size if member.isfile() else 0
                        if len(seen) > MAX_ENTRIES or expanded > MAX_EXPANDED_BYTES:
                            raise ValueError('archive expanded size/entry budget exceeded')
                        payload.extract(member, staged, filter='data')
            except (tarfile.TarError, EOFError) as error:
                raise ValueError('invalid native cache archive') from error
            if expanded != metadata['expanded_bytes'] or len(seen) != metadata['entries']:
                raise ValueError('archive contents do not match metadata')
            if destination.exists() or destination.is_symlink():
                raise ValueError('destination already exists')
            publish_directory(staged, destination)


def publish(archive: Path, metadata: dict, store, context: dict) -> dict:
    raise NotImplementedError('trusted native cache publication')


def consume(identity: dict, destination: Path, store) -> dict:
    raise NotImplementedError('read-only native cache consumption')
