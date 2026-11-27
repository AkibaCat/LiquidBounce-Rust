#!/usr/bin/env python3
"""Exercise layered patch recovery without touching an existing checkout."""
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("stage_horizon", Path(__file__).with_name("stage-horizon-build.py"))
stage_horizon = importlib.util.module_from_spec(spec)
spec.loader.exec_module(stage_horizon)


def diff(relative, before, after):
    return (f"diff --git a/{relative} b/{relative}\n" + "".join(difflib.unified_diff(
        before.splitlines(True), after.splitlines(True), fromfile=f"a/{relative}", tofile=f"b/{relative}"
    ))).encode()


class LayeredRecovery(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="liquidbounce-patch-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.previous_project, self.previous_staging = stage_horizon.PROJECT, stage_horizon.STAGING_ROOT
        self.addCleanup(self.restore_globals)
        stage_horizon.PROJECT = self.root
        stage_horizon.STAGING_ROOT = self.root / "build/staging"
        stage_horizon.STAGING_ROOT.mkdir(parents=True)
        self.stage = stage_horizon.STAGING_ROOT / "fixture"
        self.stage.mkdir()
        self.relative = "src/gfx.rs"
        self.baseline = "fn camera(){let value=1;}\n\nfn overlay(){let value=2;}\n"
        self.gpu = self.baseline.replace("value=2", "value=20")
        self.behavior = self.gpu.replace("value=1", "value=10")
        baseline = self.root / "upstream/Horizon" / self.relative
        baseline.parent.mkdir(parents=True)
        baseline.write_text(self.baseline, encoding="utf-8", newline="\n")
        (self.stage / "src").mkdir()
        self.gfx = self.stage / self.relative
        self.gfx.write_text(self.behavior, encoding="utf-8", newline="\n")
        for stem, before, after in [
            (".liquidbounce-gpu-ui-patch", self.baseline, self.gpu),
            (".liquidbounce-behavior-patch", self.gpu, self.behavior),
        ]:
            patch = diff(self.relative, before, after)
            (self.stage / (stem + ".patch")).write_bytes(patch)
            (self.stage / (stem + ".json")).write_text(json.dumps({"sha256": hashlib.sha256(patch).hexdigest()}))
        self.applied = self.stage / ".liquidbounce-behavior-patch.patch"

    def restore_globals(self):
        stage_horizon.PROJECT, stage_horizon.STAGING_ROOT = self.previous_project, self.previous_staging

    def format(self, source):
        return subprocess.run(["rustfmt", "+stable", "--edition", "2024", "--config", "skip_children=true", "--emit", "stdout"],
                              input=source.encode(), capture_output=True, check=True).stdout

    def test_recovers_both_changes_in_the_same_formatted_file(self):
        self.gfx.write_bytes(self.format(self.behavior))
        stage_horizon.restore_formatted_patch(self.stage, self.applied)
        self.assertEqual(self.gfx.read_text(), self.behavior)
        subprocess.run(["git", "apply", "--reverse", str(self.applied)], cwd=self.stage, check=True)
        self.assertEqual(self.gfx.read_text(), self.gpu)

    def test_rejects_unrecorded_changes_without_mutation(self):
        self.gfx.write_bytes(self.format(self.behavior.replace("value=10", "value=99")))
        before = self.gfx.read_bytes()
        with self.assertRaisesRegex(ValueError, "differs beyond rustfmt"):
            stage_horizon.restore_formatted_patch(self.stage, self.applied)
        self.assertEqual(self.gfx.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
