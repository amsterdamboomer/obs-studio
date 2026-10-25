"""The local gate runs Dylint through soldr's native `soldr dylint` (#2293).

`soldr cargo dylint` defers to whatever cargo-dylint is on PATH ("deferring
to cargo-dylint on PATH"), so it skips soldr's managed driver and the verdict
ledger (soldr#3894) that replays a Dylint pass whose inputs already passed.

    uv run python -m unittest ci/test_gate_dylint.py
"""

import importlib
import re
import sys
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "ci"))
GATE = ROOT / "ci" / "local_gate.py"


class GateDylintTest(unittest.TestCase):
    def test_no_cargo_dylint_call(self):
        text = GATE.read_text(encoding="utf-8")
        self.assertIsNone(re.search(r'soldr_cargo\(\s*\[\s*"dylint"', text),
                          "run Dylint with soldr_dylint(), not soldr_cargo([\"dylint\", ...])")

    def test_soldr_dylint_runs_the_native_entry_point(self):
        gate = importlib.import_module("local_gate")
        with mock.patch.object(gate, "run") as run, mock.patch.dict("os.environ", {}, clear=False):
            gate.soldr_dylint(["--all", "--", "--workspace", "--lib", "--bins"])
        argv = run.call_args.args[0]
        self.assertEqual(argv[:2], ["soldr", "dylint"])
        self.assertEqual(argv[2:], ["--all", "--", "--workspace", "--lib", "--bins"])
        # Same cache policy as soldr_cargo (zackees/zccache#1943).
        self.assertEqual(run.call_args.kwargs["env"].get("SOLDR_RUSTC_WRAPPER"), "none")


if __name__ == "__main__":
    unittest.main()
