"""Prepared-environment measurements require immutable, complete evidence (#1592)."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location('setup_probe', ROOT / 'ci/tier2_setup_probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class ProbeTest(unittest.TestCase):
    def test_corrupt_archive_is_rejected_before_loading_an_image(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'image.tar.zst').write_bytes(b'corrupt')
            (root / 'bundle.json').write_text(json.dumps({
                'archive_sha256': hashlib.sha256(b'original').hexdigest(),
                'image_id': 'sha256:' + 'a' * 64,
                'inventory': 'example\\t1\\n',
            }))
            with patch.object(probe, 'command') as command:
                with self.assertRaisesRegex(ValueError, 'checksum'):
                    probe.verify_bundle(root)
                command.assert_not_called()

    def test_image_identity_is_not_an_arbitrary_docker_argument(self):
        with self.assertRaises(ValueError):
            probe.image_id('--privileged')
        self.assertEqual(probe.image_id('sha256:' + 'a' * 64), 'sha256:' + 'a' * 64)

    def test_package_inventory_comparison_is_order_independent_but_exact(self):
        self.assertEqual(probe.inventory('a:amd64\t1\nb\t2\n'), probe.inventory('b\t2\na:amd64\t1\n'))
        self.assertNotEqual(probe.inventory('a\t1\n'), probe.inventory('a\t2\n'))
        for invalid in ('a\t1\na\t2\n', 'not an inventory', ''):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                probe.inventory(invalid)

    def test_only_exact_package_versions_can_enter_the_experiment(self):
        self.assertEqual(probe.package_specs('libc6:amd64=2.42-0ubuntu3\n'), ['libc6:amd64=2.42-0ubuntu3'])
        for text in ('--allow-unauthenticated\n', 'libc6\n', 'a=1; touch x\n', ''):
            with self.subTest(text=text), self.assertRaises(ValueError):
                probe.package_specs(text)


if __name__ == '__main__':
    unittest.main()
