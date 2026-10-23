#!/usr/bin/env -S uv run --script
"""The FHS AppImage lanes check that OBS loads the host's libnvidia-ml.so.1 (#1970).

The NixOS lane (#1028) builds build-aux/appimage/fake-nvml.c into
/run/opengl-driver/lib. On the FHS distributions NVML is a bare
libnvidia-ml.so.1 in the host library directory, so the build job builds the
same stand-in once (in the glibc 2.28 image, like the testkit), ships it with
the testkit artifact, and test-appimage.sh's first-start scenario installs it
there and fails unless OBS's GPU usage sampler loaded that copy.
"""

from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = ROOT / ".github/workflows/appimage.yaml"
SCRIPT = ROOT / "build-aux/appimage/test-appimage.sh"
STANDIN = "obs-appimage-fake-nvml.so"


def step(workflow: str, name: str) -> str:
    return workflow.split(f"      - name: {name}", 1)[1].split("\n      - ", 1)[0]


def first_start(script: str) -> str:
    return script.split('if [ "$SCENARIO" = first-start ]; then', 1)[1].split(
        "\nfi\n", 1
    )[0]


class FhsNvmlTest(unittest.TestCase):
    def test_build_job_builds_the_stand_in(self):
        helper = step(WORKFLOW.read_text(), "Build the test helper 🧰")
        self.assertRegex(
            helper,
            rf"gcc [^\n]*-shared[^\n]*-o {re.escape(STANDIN)} build-aux/appimage/fake-nvml\.c",
        )

    def test_testkit_artifact_ships_the_stand_in(self):
        workflow = WORKFLOW.read_text()
        upload = workflow.split("name: obs-appimage-testkit\n", 1)[1].split(
            "if-no-files-found", 1
        )[0]
        self.assertIn(STANDIN, upload)

    def test_first_start_installs_it_as_the_hosts_nvml(self):
        body = first_start(SCRIPT.read_text())
        self.assertIn(STANDIN, body)
        self.assertIn("libnvidia-ml.so.1", body)
        self.assertIn("ldconfig", body)
        self.assertIn("OBS_TEST_NVML_MARKER", body)

    def test_first_start_fails_unless_obs_loaded_the_hosts_copy(self):
        body = first_start(SCRIPT.read_text())
        self.assertRegex(body, r'fail "[^"]*NVML')


if __name__ == "__main__":
    unittest.main()
