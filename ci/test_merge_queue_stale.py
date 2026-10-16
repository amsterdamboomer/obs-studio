#!/usr/bin/env -S uv run --script
"""Decision logic of ci/merge_queue_stale.py (#974).

    uv run python -m unittest ci/test_merge_queue_stale.py
"""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("merge_queue_stale", ROOT / "ci" / "merge_queue_stale.py")
mqs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mqs)


def resp(*oids):
    nodes = [{"headCommit": {"oid": o}} for o in oids]
    return {"data": {"repository": {"mergeQueue": {"entries": {"nodes": nodes}}}}}


class StaleTest(unittest.TestCase):
    def test_head_in_queue_is_current(self):
        self.assertFalse(mqs.is_stale("merge_group", "a", mqs.queue_heads(resp("a", "b"))))

    def test_head_missing_is_stale(self):
        self.assertTrue(mqs.is_stale("merge_group", "c", mqs.queue_heads(resp("a", "b"))))

    def test_empty_queue_is_stale(self):
        self.assertTrue(mqs.is_stale("merge_group", "a", mqs.queue_heads(resp())))

    def test_other_events_never_stale(self):
        for ev in ("pull_request", "push", "workflow_dispatch", "schedule"):
            self.assertFalse(mqs.is_stale(ev, "c", {"a"}))

    def test_unknown_answers_never_cancel(self):
        self.assertFalse(mqs.is_stale("merge_group", "c", None))
        self.assertFalse(mqs.is_stale("merge_group", None, {"a"}))
        self.assertIsNone(mqs.queue_heads({"data": {"repository": {"mergeQueue": None}}}))
        self.assertIsNone(mqs.queue_heads({"errors": []}))


class WorkflowWiringTest(unittest.TestCase):
    def test_guard_steps_gated_on_merge_group(self):
        text = (ROOT / ".github" / "workflows" / "build-project.yaml").read_text()
        self.assertGreaterEqual(text.count("ci/merge_queue_stale.py"), 4)
        self.assertEqual(text.count("ci/merge_queue_stale.py"), text.count("if: github.event_name == 'merge_group'\n        run: uv run ci/merge_queue_stale.py"))


if __name__ == "__main__":
    unittest.main()
