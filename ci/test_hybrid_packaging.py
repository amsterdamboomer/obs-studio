"""Hybrid packages keep offline inputs pinned and publish only after approval."""

import json
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


class HybridPackagingTest(unittest.TestCase):
    def test_flatpak_has_the_pinned_compiler_and_offline_cargo_inputs(self):
        manifest = json.loads((ROOT / "build-aux/com.obsproject.Studio.json").read_text())
        version = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        compiler = next(module for module in manifest["modules"] if module["name"] == "obs-rust-toolchain")
        self.assertIn(f"rust-{version}-x86_64-unknown-linux-gnu.tar.xz", compiler["sources"][0]["url"])
        self.assertRegex(compiler["sources"][0]["sha256"], r"^[a-f0-9]{64}$")
        obs = next(module for module in manifest["modules"] if module["name"] == "obs")
        self.assertEqual(obs["build-options"]["env"]["CARGO_NET_OFFLINE"], "true")
        self.assertIn("-DOBS_LIBOBS_BACKEND=HYBRID", obs["config-opts"])
        self.assertTrue(any(source.get("dest") == ".flatpak-rust/corrosion" and source.get("sha256") for source in obs["sources"]))
        workflow = (ROOT / ".github/workflows/build-project.yaml").read_text()
        self.assertLess(workflow.index("run: build-aux/prepare-flatpak-rust.sh"), workflow.index("- name: Build Flatpak Manifest"))

    def test_every_publication_path_checks_hybrid_release_approval(self):
        for name, publish in (("release-rust.yaml", "- name: Create or repair release"),
                              ("appimage.yaml", "- name: Attach to the tag's release")):
            source = (ROOT / ".github/workflows" / name).read_text().split("\n  release:", 1)[1]
            with self.subTest(workflow=name):
                self.assertIn("ci/hybrid_release_gate.py", source)
                self.assertLess(source.index("ci/hybrid_release_gate.py"), source.index(publish))
