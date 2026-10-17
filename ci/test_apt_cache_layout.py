"""Container-relative apt archives must be relocated on hosted runners."""
import unittest
from pathlib import Path
from unittest.mock import patch
from apt_cache_layout import restored_archive_path, normalize_archives

class AptCacheLayoutTest(unittest.TestCase):
    def test_container_and_host_workspace_depths_differ(self):
        self.assertEqual(restored_archive_path(Path('/__w/repo/repo')), Path('/var/cache/apt/archives'))
        self.assertEqual(restored_archive_path(Path('/home/runner/work/repo/repo')), Path('/home/runner/var/cache/apt/archives'))

    def test_copy_debs_from_container_relative_restore_location(self):
        source=Path('/home/runner/var/cache/apt/archives/example.deb')
        with patch.object(Path,'is_dir',return_value=True), patch.object(Path,'glob',return_value=[source]) as glob, patch.object(Path,'exists',return_value=False), patch.object(Path,'mkdir'), patch('apt_cache_layout.shutil.copy2') as copy:
            self.assertEqual(normalize_archives(Path('/home/runner/work/repo/repo'),Path('/var/cache/apt/archives')),1)
            glob.assert_called_once_with('*.deb')
            copy.assert_called_once_with(source,Path('/var/cache/apt/archives/example.deb'))

    def test_container_already_has_correct_layout(self):
        with patch('apt_cache_layout.shutil.copy2') as copy:
            self.assertEqual(normalize_archives(Path('/__w/repo/repo'),Path('/var/cache/apt/archives')),0)
            copy.assert_not_called()

if __name__ == '__main__':
    unittest.main()
