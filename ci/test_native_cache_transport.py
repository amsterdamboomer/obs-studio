#!/usr/bin/env -S uv run --script
"""Cheap archive-boundary checks; no network, compiler or additional CI lane."""

import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location('native_cache_transport', Path(__file__).with_name('native_cache_transport.py'))
TRANSPORT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TRANSPORT)
IDENTITY = {'platform': 'windows', 'architecture': 'x64', 'configuration': 'RelWithDebInfo',
            'compiler': 'cl-version-fixture', 'sdk': 'sdk-fixture', 'cache_format': 'ccache-4.14.1'}
SHA = 'a' * 40


class NativeCacheTransportTest(unittest.TestCase):
    def fixture(self, root):
        source = root / 'source'
        (source / 'nested').mkdir(parents=True)
        (source / 'nested/object').write_bytes(b'compiler cache object\x00')
        archive = root / 'seed.tar.gz'
        metadata = TRANSPORT.pack(source, archive, IDENTITY, SHA)
        return source, archive, metadata

    def test_round_trip_preserves_cache_and_producer_identity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, archive, metadata = self.fixture(root)
            self.assertEqual(metadata['source_sha'], SHA)
            self.assertEqual(metadata['identity'], IDENTITY)
            self.assertEqual(metadata['bytes'], archive.stat().st_size)
            self.assertEqual(metadata['sha256'], hashlib.sha256(archive.read_bytes()).hexdigest())
            json.dumps(metadata)
            TRANSPORT.restore(archive, metadata, root / 'restored', IDENTITY)
            self.assertEqual((root / 'restored/nested/object').read_bytes(), (source / 'nested/object').read_bytes())

    def test_identity_changes_reject_seed_without_touching_destination(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, archive, metadata = self.fixture(root)
            for key in IDENTITY:
                changed = dict(IDENTITY, **{key: 'different'})
                with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'identity'):
                    TRANSPORT.restore(archive, metadata, root / 'restored', changed)
                self.assertFalse((root / 'restored').exists())

    def test_corrupt_payload_rejected_before_restore(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, archive, metadata = self.fixture(root)
            data = bytearray(archive.read_bytes())
            data[len(data) // 2] ^= 1
            archive.write_bytes(data)
            with self.assertRaisesRegex(ValueError, 'digest'):
                TRANSPORT.restore(archive, metadata, root / 'restored', IDENTITY)
            self.assertFalse((root / 'restored').exists())

    def test_existing_destination_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, archive, metadata = self.fixture(root)
            dest = root / 'restored'
            dest.mkdir()
            (dest / 'sentinel').write_text('keep')
            with self.assertRaisesRegex(ValueError, 'destination'):
                TRANSPORT.restore(archive, metadata, dest, IDENTITY)
            self.assertEqual((dest / 'sentinel').read_text(), 'keep')

    def test_verified_archive_still_rejects_path_escape_and_special_files(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, archive, metadata = self.fixture(root)
            for name, kind in (('../escape', tarfile.REGTYPE), ('/absolute', tarfile.REGTYPE),
                               ('link', tarfile.SYMTYPE), ('fifo', tarfile.FIFOTYPE)):
                member = tarfile.TarInfo(name)
                member.type = kind
                member.linkname = '../../escape'
                with tarfile.open(archive, 'w:gz') as stream:
                    stream.addfile(member, io.BytesIO(b''))
                modified = copy.deepcopy(metadata)
                modified['bytes'] = archive.stat().st_size
                modified['sha256'] = hashlib.sha256(archive.read_bytes()).hexdigest()
                with self.subTest(name=name), self.assertRaises(ValueError):
                    TRANSPORT.restore(archive, modified, root / 'restored', IDENTITY)
                self.assertFalse((root / 'restored').exists())
                self.assertFalse((root / 'escape').exists())

    def test_pack_rejects_links_and_preserves_existing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / 'source'
            source.mkdir()
            (source / 'link').symlink_to(root / 'external')
            output = root / 'seed.tar.gz'
            with self.assertRaisesRegex(ValueError, 'link'):
                TRANSPORT.pack(source, output, IDENTITY, SHA)
            self.assertFalse(output.exists())
            (root / 'other').mkdir()
            output.write_bytes(b'keep')
            with self.assertRaisesRegex(ValueError, 'output'):
                TRANSPORT.pack(root / 'other', output, IDENTITY, SHA)
            self.assertEqual(output.read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
