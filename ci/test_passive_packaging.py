"""Exercise the packaging guard's explicit acquisition exception at its CLI seam."""
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest


class PassivePackaging(unittest.TestCase):
    def check(self, crates, extra_source=None):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            (root / "ci").mkdir()
            script = root / "ci/assert_passive_packaging.py"
            shutil.copyfile(pathlib.Path(__file__).with_name(script.name), script)
            (root / "Cargo.lock").write_text(
                "\n".join(f'[[package]]\nname = "{name}"' for name in crates)
            )
            acquisition = root / "crates/montagent-core/src/verbs/fonts_acquire.rs"
            acquisition.parent.mkdir(parents=True)
            acquisition.write_text("let client = reqwest::blocking::Client::new();")
            if extra_source:
                (acquisition.parent / "render.rs").write_text(extra_source)
            return subprocess.run(
                [sys.executable, str(script)], capture_output=True, text=True, check=False
            )

    def test_explicit_acquisition_stack_is_allowed(self):
        result = self.check(["reqwest", "hyper", "hyper-util", "hyper-rustls"])
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_telemetry_and_other_http_clients_remain_refused(self):
        for crate in ["sentry", "self_update", "ureq", "curl", "ffmpeg-sys-next"]:
            with self.subTest(crate=crate):
                result = self.check(["reqwest", crate])
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(crate, result.stderr)

    def test_render_cannot_use_the_acquisition_client(self):
        for source in [
            "let client = reqwest::blocking::Client::new();",
            "use reqwest as client;",
            "extern crate reqwest as client;",
        ]:
            with self.subTest(source=source):
                result = self.check(["reqwest"], source)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("outside explicit font acquisition", result.stderr)


if __name__ == "__main__":
    unittest.main()
