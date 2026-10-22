"""The opt-in cache experiment must measure real rebuilds without mutating its seed."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from ci.native_cache_probe import run_probe


ROOT = Path(__file__).resolve().parent.parent


class NativeCacheProbeTest(unittest.TestCase):
    def test_isolated_counterbalanced_rebuilds_preserve_the_original_cache(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            seed = root / "seed"
            seed.mkdir()
            (seed / "object").write_bytes(b"original compiler object")
            calls = []

            def execute(command, **kwargs):
                env = kwargs["env"]
                cache = Path(env["CCACHE_DIR"])
                self.assertNotEqual(cache, seed)
                if command[0] == "cmake":
                    self.assertEqual((cache / "object").read_bytes(), b"original compiler object")
                    (cache / "object").write_bytes(b"rebuild changed private copy")
                calls.append((command, env["CCACHE_MAXSIZE"], str(cache)))
                return subprocess.CompletedProcess(command, 0, stdout="cache statistics\n")

            report = run_probe(seed, root / "probe", "arm64", "RelWithDebInfo", execute=execute)
            builds = [(command, limit, cache) for command, limit, cache in calls if command[0] == "cmake"]
            self.assertEqual([limit for _, limit, _ in builds], ["2G", "350M", "350M", "2G"])
            self.assertEqual(len({cache for _, _, cache in builds}), 4)
            for command, _, _ in builds:
                self.assertEqual(command, ["cmake", "--build", "--preset", "windows-arm64",
                                          "--config", "RelWithDebInfo", "--clean-first", "--parallel",
                                          "--", "/consoleLoggerParameters:Summary", "/noLogo"])
            self.assertEqual((seed / "object").read_bytes(), b"original compiler object")
            self.assertEqual(report["status"], "success")
            self.assertEqual(len(report["observations"]), 4)

    def test_failed_rebuild_is_reported_and_not_treated_as_a_timing_sample(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            seed = root / "seed"
            seed.mkdir()

            def fail(command, **kwargs):
                if command[0] == "cmake":
                    raise subprocess.CalledProcessError(7, command)
                return subprocess.CompletedProcess(command, 0, stdout="stats")

            with self.assertRaises(subprocess.CalledProcessError):
                run_probe(seed, root / "probe", "x64", "RelWithDebInfo", execute=fail)
            import json
            report = json.loads((root / "probe" / "report.json").read_text())
            self.assertEqual(report["status"], "failure")
            self.assertEqual(report["observations"], [])

    def test_workflow_probe_is_explicit_manual_only_and_defaults_off(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        build = (ROOT / ".github/workflows/build-project.yaml").read_text()
        self.assertIn("native-cache-probe:", workflow)
        self.assertIn("github.event_name == 'workflow_dispatch' && inputs.native-cache-probe", workflow)
        header = build.split("\nenv:", 1)[0]
        probe = header.split("      native-cache-probe:", 1)[1]
        self.assertIn("default: false", probe)
        step = build.split("- name: Measure Windows Cache Budgets", 1)[1].split("\n      - ", 1)[0]
        self.assertIn("github.event_name == 'workflow_dispatch' && inputs.native-cache-probe", step)
        self.assertIn("uv run ci/native_cache_probe.py", step)
        self.assertNotIn("continue-on-error", step)
        # Ordinary compilation happens first and is not conditional on the probe.
        windows = build.split("  windows-build:", 1)[1].split("  rust-tests-linux:", 1)[0]
        self.assertLess(windows.index("- name: Build OBS Studio"), windows.index("- name: Measure Windows"))


if __name__ == "__main__":
    unittest.main()
