"""Restored Ubuntu app compiler caches must be consumed before sizing saves."""
from pathlib import Path
import re
import unittest

ROOT=Path(__file__).resolve().parents[1]

def ubuntu_job():
    text=(ROOT/'.github/workflows/build-project.yaml').read_text()
    return re.search(r'^  ubuntu-build:\n(.*?)(?=^  [a-z0-9-]+:\n)',text,re.M|re.S).group(1)

class UbuntuCachePreserveTest(unittest.TestCase):
    def test_restored_cache_is_not_deleted_before_build(self):
        job=ubuntu_job()
        before=job[:job.index('- name: Build OBS Studio')]
        self.assertNotIn('rm -rf "$dir"',before)
        self.assertNotIn('starting cold',before)

    def test_oversize_snapshot_skips_save_after_compiler_consumption(self):
        job=ubuntu_job()
        self.assertLess(job.index('- name: Build OBS Studio'),job.index('id: cache-size'))
        self.assertIn('8192',job)
        self.assertIn("steps.cache-size.outputs.save == 'true'",job)
        self.assertIn('save=false',job)
        self.assertIn('save=true',job)

if __name__ == '__main__':
    unittest.main()
