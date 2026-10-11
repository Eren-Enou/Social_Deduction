import hashlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("fetch_evidence", Path(__file__).parents[1] / "fetch_evidence.py")
fetch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fetch)


class FrozenRetrieval(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.body = b'{"immutable":true}\n'
        self.entry = {"path": "evidence/experiment-012.json", "bytes": len(self.body),
                      "sha256": hashlib.sha256(self.body).hexdigest(), "storage": "github-release",
                      "release_id": 123, "asset_id": 456, "tag": "frozen-v1"}
        self.manifest = {"version": "frozen-evidence-release-v1", "repository": fetch.REPOSITORY,
                         "artifacts": [self.entry]}
        self.path = self.root / self.entry["path"]

    def install(self, body):
        fetch.install_file(self.path, self.entry, io.BytesIO(body))

    def test_exact_bytes_install_and_validate(self):
        self.install(self.body)
        self.assertEqual(self.path.read_bytes(), self.body)
        fetch.retrieve(self.manifest, self.root, verify_only=True)

    def test_wrong_digest_rejected_without_target_or_temporary(self):
        with self.assertRaises(ValueError):
            self.install(b'x' * len(self.body))
        self.assertFalse(self.path.exists())
        self.assertEqual(list(self.path.parent.iterdir()), [])

    def test_truncated_download_is_not_installed(self):
        with self.assertRaises(ValueError):
            self.install(self.body[:-1])
        self.assertFalse(self.path.exists())

    def test_overlong_download_is_not_installed(self):
        with self.assertRaises(ValueError):
            self.install(self.body + b'x')
        self.assertFalse(self.path.exists())

    def test_existing_corrupt_file_refuses_replacement(self):
        self.path.parent.mkdir()
        self.path.write_bytes(b'corrupt')
        with patch.object(fetch, "request", side_effect=AssertionError("network must not run")):
            with self.assertRaises(ValueError):
                fetch.retrieve(self.manifest, self.root)
        self.assertEqual(self.path.read_bytes(), b'corrupt')

    def test_existing_valid_file_needs_no_network(self):
        self.install(self.body)
        with patch.object(fetch, "request", side_effect=AssertionError("network must not run")):
            fetch.retrieve(self.manifest, self.root)

    def test_missing_git_file_is_required(self):
        self.entry["storage"] = "git"
        with self.assertRaisesRegex(ValueError, "missing required"):
            fetch.retrieve(self.manifest, self.root)

    def test_verify_only_never_silently_skips_missing_release_file(self):
        with self.assertRaisesRegex(ValueError, "missing required"):
            fetch.retrieve(self.manifest, self.root, verify_only=True)

    def test_concurrent_target_never_overwritten(self):
        self.path.parent.mkdir()
        self.path.write_bytes(b'keep')
        with self.assertRaises(FileExistsError):
            self.install(self.body)
        self.assertEqual(self.path.read_bytes(), b'keep')
        self.assertEqual(list(self.path.parent.iterdir()), [self.path])

    def test_manifest_rejects_path_escape_duplicates_and_invalid_identity(self):
        for change in ({"path": "../outside.json"}, {"path": "evidence/../outside.json"},
                       {"bytes": True}, {"sha256": "invalid"}, {"asset_id": False},
                       {"tag": "../foreign"}, {"storage": "optional"}):
            altered = dict(self.entry, **change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                fetch.validate_manifest(dict(self.manifest, artifacts=[altered]))
        with self.assertRaises(ValueError):
            fetch.validate_manifest(dict(self.manifest, artifacts=[self.entry, self.entry]))

    def test_asset_replacement_id_digest_and_size_are_rejected(self):
        valid = {"name": self.path.name, "size": self.entry["bytes"],
                 "digest": "sha256:" + self.entry["sha256"], "state": "uploaded"}
        for assets in ({999: valid}, {456: dict(valid, size=1)},
                       {456: dict(valid, digest="sha256:" + "0" * 64)}):
            with self.subTest(assets=assets), self.assertRaises(ValueError):
                fetch.asset_stream(self.entry, assets)

    def test_wrong_release_id_tag_or_draft_rejected(self):
        for change in ({"id": 124}, {"tag_name": "other"}, {"draft": True}):
            data = {"id": 123, "tag_name": "frozen-v1", "draft": False, "assets": []}
            data.update(change)
            with patch.object(fetch, "request", return_value=io.BytesIO(json.dumps(data).encode())):
                with self.subTest(change=change), self.assertRaises(ValueError):
                    fetch.release_assets(self.entry)


if __name__ == "__main__":
    unittest.main()
