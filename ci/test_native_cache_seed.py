#!/usr/bin/env -S uv run --script
"""Qualification seeds must not alias same-source builds on different images."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('native_cache_seed', Path(__file__).with_name('native_cache_seed.py'))
SEED = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SEED)
IDENTITY = {'platform': 'windows', 'architecture': 'x64', 'configuration': 'RelWithDebInfo',
            'compiler': 'a' * 64, 'sdk': 'b' * 64, 'cache_format': 'ccache-4.14.1'}


class NativeSeedTest(unittest.TestCase):
    def test_same_source_different_toolchain_cannot_alias(self):
        original = SEED.seed_key(IDENTITY, 'c' * 40)
        for field in ('compiler', 'sdk', 'configuration', 'cache_format', 'architecture'):
            changed = dict(IDENTITY, **{field: {'configuration': 'Release', 'architecture': 'arm64'}.get(field, 'd' * 64)})
            with self.subTest(field=field):
                self.assertNotEqual(original, SEED.seed_key(changed, 'c' * 40))

    def test_deterministic_key_stays_in_owned_family_and_binds_source(self):
        key = SEED.seed_key(IDENTITY, 'c' * 40)
        self.assertEqual(key, SEED.seed_key(dict(reversed(list(IDENTITY.items()))), 'c' * 40))
        self.assertTrue(key.startswith('windows-ccache-x64-RelWithDebInfo-' + 'c' * 40 + '-transport-'))
        self.assertNotEqual(key, SEED.seed_key(IDENTITY, 'd' * 40))
        mac = dict(IDENTITY, platform='macos', architecture='x86_64')
        self.assertTrue(SEED.seed_key(mac, 'c' * 40).startswith('macOS-xcode-x86_64-'))

    def test_output_values_cannot_inject_another_key(self):
        for source in ('', 'c' * 39, 'C' * 40, 'c' * 40 + '\nother=value'):
            with self.subTest(source=source), self.assertRaises(ValueError):
                SEED.seed_key(IDENTITY, source)
        with self.assertRaises(ValueError):
            SEED.seed_key(dict(IDENTITY, configuration='Release\nother=value'), 'c' * 40)


if __name__ == '__main__':
    unittest.main()
