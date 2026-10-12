#!/usr/bin/env python3
"""Contract and validation tests for discovery eval tasks (01-05) and held-out seal (06).

Ensures:
1. Seam 1: Task Contract & Directory Structure (prompt.txt, manifest.json, fixtures, reference)
2. Seam 2: Linguistic Audit of Zero-Knowledge Prompts (no command or internal schema leaks)
3. Seam 3: Reference Solutions Execution (dynamic render and programmatic verify with 0 errors)
4. Seam 4: Held-Out Task 06 Seal & Integrity (held-out.sha256 format and checksums)
"""

import json
import os
import re
import shutil
import subprocess
import unittest
from pathlib import Path

DISCOVERY_DIR = Path(__file__).resolve().parent
REPO_ROOT = DISCOVERY_DIR.parent.parent
TASKS_DIR = DISCOVERY_DIR / "tasks"
REGISTER_FILE = DISCOVERY_DIR / "register.json"

OPEN_TASK_IDS = [
    "discovery-01-assembly-inspect",
    "discovery-02-font-licensing",
    "discovery-03-chroma-key",
    "discovery-04-timeline-shift",
    "discovery-05-scene-transition",
]

# Strict regex patterns for forbidden Montagent verbs, commands, and schema terms
FORBIDDEN_PROMPT_PATTERNS = [
    r"\bmontagent\b",
    r"\bvalidate\b",
    r"\brender\b",
    r"\bverify\b",
    r"\bcreate-project\b",
    r"\bcreate_project\b",
    r"\bshift\b",
    r"\bpreview\b",
    r"\bmeasure\b",
    r"\bquery\b",
    r"\bvendor\b",
    r"\bacquire\b",
    r"\btransition\b",
    r"\bcrossfade\b",
    r"\bconstant_power\b",
    r"\bconstant-power\b",
    r"\bchroma\b",
    r"\bstagger\b",
    r"\bfontVendor\b",
    r"\btracks\b",
    r"\belements\b",
]

MONTAGENT_BIN = (
    os.environ.get("MONTAGENT")
    or shutil.which("montagent")
    or str(REPO_ROOT / "target" / "debug" / "montagent")
)


class TestDiscoveryTasksContract(unittest.TestCase):
    """Seam 1: Task Contract & Directory Structure."""

    def test_tasks_directory_exists(self):
        self.assertTrue(TASKS_DIR.is_dir(), f"Missing tasks dir at {TASKS_DIR}")

    def test_open_task_directories_exist(self):
        for task_id in OPEN_TASK_IDS:
            task_dir = TASKS_DIR / task_id
            self.assertTrue(task_dir.is_dir(), f"Missing task directory for {task_id}")

    def test_task_files_and_manifest_structure(self):
        with open(REGISTER_FILE, "r") as f:
            register_data = json.load(f)

        features = register_data.get("features", [])
        self.assertTrue(len(features) > 0, "register.json has no features defined")

        # Map task IDs to their expected primary capabilities from register.json
        expected_primary_map = {}
        for feature in features:
            feature_id = feature.get("id")
            for task_id in feature.get("tasks", {}).get("primary_targets", []):
                expected_primary_map.setdefault(task_id, set()).add(feature_id)

        for task_id in OPEN_TASK_IDS:
            task_dir = TASKS_DIR / task_id
            prompt_file = task_dir / "prompt.txt"
            manifest_file = task_dir / "manifest.json"
            fixtures_dir = task_dir / "fixtures"
            reference_dir = task_dir / "reference"

            self.assertTrue(prompt_file.is_file(), f"Missing prompt.txt in {task_id}")
            self.assertTrue(manifest_file.is_file(), f"Missing manifest.json in {task_id}")
            self.assertTrue(fixtures_dir.is_dir(), f"Missing fixtures/ in {task_id}")
            self.assertTrue(reference_dir.is_dir(), f"Missing reference/ in {task_id}")

            with open(manifest_file, "r") as f:
                manifest = json.load(f)

            self.assertEqual(manifest.get("task_id"), task_id)
            self.assertIn("name", manifest)
            self.assertIn("primary_target_capabilities", manifest)
            self.assertIn("incidental_capabilities", manifest)
            self.assertIn("deliverables", manifest)
            self.assertIn("allowable_alternatives", manifest)
            self.assertIn("diagnostic_traps", manifest)

            # Check that manifest primary targets align with register.json
            expected_caps = expected_primary_map.get(task_id, set())
            manifest_caps = set(manifest.get("primary_target_capabilities", []))
            self.assertEqual(
                manifest_caps,
                expected_caps,
                f"Task {task_id} primary capabilities do not match register.json. Diff: {manifest_caps ^ expected_caps}"
            )


class TestZeroKnowledgePrompts(unittest.TestCase):
    """Seam 2: Linguistic Audit of Zero-Knowledge Prompts."""

    def test_prompts_contain_no_internal_names(self):
        for task_id in OPEN_TASK_IDS:
            prompt_file = TASKS_DIR / task_id / "prompt.txt"
            self.assertTrue(prompt_file.exists(), f"Missing prompt file for {task_id}")
            text = prompt_file.read_text()
            for pattern in FORBIDDEN_PROMPT_PATTERNS:
                match = re.search(pattern, text, re.IGNORECASE)
                self.assertIsNone(
                    match,
                    f"Task {task_id} prompt leaks forbidden pattern '{pattern}': '{match.group(0) if match else ''}' in:\n{text}"
                )


class TestHeldOutSeal(unittest.TestCase):
    """Seam 4: Held-Out Task 06 Seal & Integrity."""

    def test_held_out_sha256_exists_and_valid(self):
        seal_file = TASKS_DIR / "held-out.sha256"
        self.assertTrue(seal_file.is_file(), f"Missing held-out.sha256 at {seal_file}")
        raw_lines = seal_file.read_text().strip().splitlines()
        lines = [line.strip() for line in raw_lines if line.strip() and not line.strip().startswith("#")]
        self.assertTrue(len(lines) >= 1, "held-out.sha256 has no hash entries")
        for line in lines:
            parts = line.strip().split()
            self.assertTrue(len(parts) >= 2, f"Invalid format in held-out.sha256: {line}")
            sha, target = parts[0], parts[1]
            self.assertEqual(len(sha), 64, f"Invalid SHA-256 length: {sha}")
            self.assertTrue(re.match(r"^[0-9a-f]{64}$", sha), f"Invalid SHA-256 characters: {sha}")
            self.assertIn("06", target, f"Target filename should reference Task 06: {target}")


class TestReferenceSolutions(unittest.TestCase):
    """Seam 3: Reference Solutions Validation, Render, and Verification."""

    def _run_montagent_json(self, verb: str, target: str, cwd: Path):
        self.assertTrue(
            os.path.exists(MONTAGENT_BIN) or shutil.which(MONTAGENT_BIN),
            f"Montagent binary not found at {MONTAGENT_BIN}"
        )
        res = subprocess.run(
            [MONTAGENT_BIN, verb, target, "--json"],
            capture_output=True,
            text=True,
            cwd=str(cwd)
        )
        self.assertEqual(
            res.returncode,
            0,
            f"montagent {verb} failed for {target} in {cwd}:\nSTDOUT: {res.stdout}\nSTDERR: {res.stderr}"
        )
        data = json.loads(res.stdout)
        self.assertEqual(
            data.get("summary", {}).get("error", 0),
            0,
            f"montagent {verb} reported errors for {target}: {data.get('findings')}"
        )
        return data

    def test_reference_solutions_validate(self):
        for task_id in OPEN_TASK_IDS:
            ref_dir = TASKS_DIR / task_id / "reference"
            self.assertTrue(ref_dir.is_dir(), f"Missing reference directory: {ref_dir}")
            project_files = list(ref_dir.glob("*.json"))
            self.assertTrue(
                len(project_files) >= 1,
                f"No project.json found in {ref_dir}"
            )
            for proj in project_files:
                self._run_montagent_json("validate", f"./{proj.name}", ref_dir)

    def test_reference_solutions_render_and_verify(self):
        for task_id in OPEN_TASK_IDS:
            ref_dir = TASKS_DIR / task_id / "reference"
            self.assertTrue(ref_dir.is_dir(), f"Missing reference directory: {ref_dir}")
            output_file = ref_dir / "output.mp4"

            # Render project dynamically
            self._run_montagent_json("render", "./project.json", ref_dir)
            self.assertTrue(output_file.is_file(), f"render did not produce output.mp4 in {ref_dir}")

            # Verify deliverable with decoder
            self._run_montagent_json("verify", "./project.json", ref_dir)

            # Clean up output.mp4 and partial files to keep git lightweight per RETENTION.md §1 & §2.2
            output_file.unlink(missing_ok=True)
            for p in ref_dir.iterdir():
                if "montagent-partial" in p.name:
                    p.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main()
