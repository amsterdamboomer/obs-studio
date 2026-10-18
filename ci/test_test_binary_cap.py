#!/usr/bin/env -S uv run --script
"""Unit tests for ci/test_binary_cap.py (#968)."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import test_binary_cap as cap  # noqa: E402


def write(path: Path, text: str = "") -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)


class CategoryBudgetTest(unittest.TestCase):
    def test_metadata_counts_unit_and_custom_feature_gated_tests(self):
        targets = [
            {"name": "library", "kind": ["lib"], "test": True},
            {"name": "disabled", "kind": ["bin"], "test": False},
            {"name": "custom", "kind": ["test"], "test": False, "required-features": ["gpu"]},
            {"name": "example", "kind": ["example"], "test": True},
            {"name": "bench", "kind": ["bench"], "test": True},
        ]
        metadata = {"workspace_members": ["member"], "packages": [
            {"id": "member", "name": "component", "targets": targets},
            {"id": "dependency", "name": "dependency", "targets": targets},
        ]}
        self.assertEqual(cap.cargo_test_targets(metadata), [
            ("component", "custom", "test"), ("component", "library", "lib"),
        ])

    def test_selected_categories_pass(self):
        targets = list(cap.CATEGORY_OWNERS)
        self.assertEqual(cap.category_violations(targets), [])

    def test_extra_unit_harness_fails_even_below_eight(self):
        self.assertTrue(cap.category_violations([("core", "core", "lib")]))

    def test_disabling_categories_cannot_satisfy_budget(self):
        errors = cap.category_violations([])
        self.assertEqual(len(errors), 8)
        self.assertTrue(all("missing category" in error for error in errors))

    def test_duplicate_category_and_total_budget_fail(self):
        targets = [(f"suite-{i}", "obs-tests-core", "test") for i in range(9)]
        errors = cap.category_violations(targets)
        self.assertTrue(any("duplicate" in error for error in errors))
        self.assertTrue(any("9 test binaries" in error for error in errors))


class CountTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        write(self.root / "Cargo.toml", '[workspace]\nmembers = ["rust/*"]\nexclude = ["rust/tools"]\n')
        write(self.root / "rust/tools/Cargo.toml", '[package]\nname = "tools"\n')
        write(self.root / "rust/tools/tests/a.rs")

    def tearDown(self):
        self.tmp.cleanup()

    def integration_counts(self):
        counts = {}
        for package, _, kind in cap.workspace_test_targets(self.root):
            if kind == "test":
                counts[package] = counts.get(package, 0) + 1
        return counts

    def crate(self, name: str, manifest: str = "") -> Path:
        crate = self.root / "rust" / name
        write(crate / "Cargo.toml", f'[package]\nname = "{name}"\n{manifest}')
        return crate

    def test_counts_files_category_dirs_and_explicit_targets(self):
        crate = self.crate("a", '\n[[test]]\nname = "extra"\npath = "other/extra.rs"\n')
        write(crate / "tests/one.rs")
        write(crate / "tests/two.rs")
        write(crate / "tests/cat/main.rs")
        write(crate / "tests/cat/one_mod.rs")
        write(crate / "tests/common/mod.rs")
        self.assertEqual(self.integration_counts(), {"a": 4})

    def test_explicit_target_for_auto_file_counts_once(self):
        crate = self.crate("a", '\n[[test]]\nname = "one"\npath = "tests/one.rs"\nharness = false\n')
        write(crate / "tests/one.rs")
        self.assertEqual(self.integration_counts(), {"a": 1})

    def test_autotests_false_counts_only_explicit(self):
        crate = self.crate("a", 'autotests = false\n\n[[test]]\nname = "x"\npath = "tests/x.rs"\n')
        write(crate / "tests/x.rs")
        write(crate / "tests/y.rs")
        self.assertEqual(self.integration_counts(), {"a": 1})

    def test_excluded_member_is_skipped(self):
        self.assertNotIn("tools", self.integration_counts())

    def test_inventory_finds_empty_and_automatic_unit_harnesses(self):
        crate = self.crate("a")
        write(crate / "src/lib.rs")
        write(crate / "src/main.rs")
        write(crate / "src/bin/worker.rs")
        write(crate / "src/bin/nested/main.rs")
        write(crate / "tests/suite/main.rs")
        self.assertEqual(cap.workspace_test_targets(self.root), [
            ("a", "a", "bin"), ("a", "a", "lib"),
            ("a", "nested", "bin"), ("a", "suite", "test"),
            ("a", "worker", "bin"),
        ])

    def test_disabled_units_and_discovery_leave_only_explicit_custom_test(self):
        crate = self.crate("a", '''autotests = false
autobins = false
[lib]
test = false
[[bin]]
name = "cli"
path = "tools/cli.rs"
test = false
[[test]]
name = "suite"
path = "checks/suite.rs"
harness = false
required-features = ["gpu"]
''')
        write(crate / "src/lib.rs")
        write(crate / "src/bin/worker.rs")
        write(crate / "tests/accidental.rs")
        self.assertEqual(cap.workspace_test_targets(self.root), [("a", "suite", "test")])



if __name__ == "__main__":
    unittest.main()
