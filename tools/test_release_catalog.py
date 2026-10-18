import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from tools.verify_release_catalog import compare_files


class CatalogFiles(unittest.TestCase):
    def test_verification_builds_generator_with_an_empty_cargo_target(self):
        root = Path(__file__).resolve().parents[1]
        parent = root / 'target'
        parent.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent) as directory:
            build = Path(directory) / 'build'
            env = dict(os.environ, CARGO_TARGET_DIR=str(build))
            env.pop('GITHUB_EVENT_PATH', None)
            result = subprocess.run(
                [sys.executable, '-B', str(root / 'tools/verify_release_catalog.py')],
                cwd=root, env=env, capture_output=True, encoding='utf-8', timeout=300,
            )
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('previous release identities verified', result.stdout)
            program = 'xtask.exe' if os.name == 'nt' else 'xtask'
            self.assertTrue((build / 'debug' / program).is_file())

    def test_tampering_missing_and_extra_files_are_rejected(self):
        parent = Path(__file__).resolve().parents[1] / 'target'
        parent.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=parent) as directory:
            root, generated = Path(directory) / 'source', Path(directory) / 'generated'
            for base in (root, generated):
                (base / 'bucket').mkdir(parents=True)
                (base / 'bucket/pactrun-preview.json').write_text('{"version":"1.0.0-alpha.1"}\n', encoding='utf-8')
            compare_files(root, generated)
            file = root / 'bucket/pactrun-preview.json'
            original = file.read_bytes()
            file.write_text('{"version":"changed"}\n', encoding='utf-8')
            with self.assertRaises(ValueError): compare_files(root, generated)
            file.unlink()
            with self.assertRaises(ValueError): compare_files(root, generated)
            file.write_bytes(original)
            (root / 'bucket/unreviewed.json').write_text('{}', encoding='utf-8')
            with self.assertRaises(ValueError): compare_files(root, generated)


if __name__ == '__main__':
    unittest.main()
