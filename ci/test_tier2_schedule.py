#!/usr/bin/env -S uv run --script
"""Tier 2 Linux starts the slowest tests first and runs 8 at a time (#931).

    uv run python -m unittest ci/test_tier2_schedule.py

Each OFF/ON ctest run took about 285 s at -j4 for 660 s of test work: every
CI container started without CTestCostData.txt, so ctest did not know that
test_deinterlace_2x_field_order (83 s) and the other real-time media tests
are slow and could start them last. validate.sh now carries the cost data
per mode in the ccache directory, which CI already restores.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VALIDATE = ROOT / "rust" / "tools" / "linux-validate" / "validate.sh"
WORKFLOW = ROOT / ".github" / "workflows" / "build-project.yaml"


class CostDataTest(unittest.TestCase):
    def setUp(self):
        self.text = VALIDATE.read_text(encoding="utf-8")
        start = self.text.index('echo "== [$mode] ctest"')
        self.around_ctest = self.text[start - 1500 : start + 1500]

    def test_cost_data_is_restored_before_ctest(self):
        restore = self.text.find("CTestCostData.txt")
        ctest = self.text.find("xvfb-run -a ctest")
        self.assertNotEqual(restore, -1, "validate.sh never mentions CTestCostData.txt")
        self.assertLess(restore, ctest, "cost data must be in place before ctest runs")

    def test_cost_data_lives_in_the_ccache_dir_per_mode(self):
        self.assertRegex(self.around_ctest, r"\$\{?CCACHE_DIR\}?/[^\s\"]*\$\{?mode\}?")

    def test_cost_data_is_saved_after_ctest(self):
        ctest = self.text.find("xvfb-run -a ctest")
        self.assertNotEqual(self.text.find("CTestCostData.txt", ctest), -1,
                            "cost data must be written back after ctest")


class JobsTest(unittest.TestCase):
    def test_ci_runs_tier2_eight_at_a_time(self):
        m = re.search(r"OBS_CTEST_JOBS:\s*(\d+)", WORKFLOW.read_text(encoding="utf-8"))
        self.assertIsNotNone(m)
        self.assertGreaterEqual(int(m.group(1)), 8)


if __name__ == "__main__":
    unittest.main()
