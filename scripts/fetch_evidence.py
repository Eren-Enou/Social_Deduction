#!/usr/bin/env python3
"""Retrieve exact frozen bytes. Missing/corrupt evidence is an error, never a skip."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile
import urllib.request

REPOSITORY = "Eren-Enou/Social_Deduction"
CHUNK = 1024 * 1024


def validate_manifest(manifest):
    if manifest.get("version") != "frozen-evidence-release-v1":
        raise ValueError("unsupported evidence manifest version")
    if manifest.get("repository") != REPOSITORY:
        raise ValueError("unexpected repository")
    entries = manifest.get("artifacts")
    if not isinstance(entries, list) or not entries:
        raise ValueError("empty artifact manifest")
    paths, assets = set(), set()
    for entry in entries:
        path = entry.get("path", "")
        if not re.fullmatch(r"evidence/experiment-[0-9]{3}(?:-windows|-cost-control)?\.json", path):
            raise ValueError("unsafe evidence path")
        if path in paths:
            raise ValueError("duplicate evidence path")
        paths.add(path)
        if type(entry.get("bytes")) is not int or entry["bytes"] <= 0:
            raise ValueError("invalid byte length")
        if not re.fullmatch(r"[0-9a-f]{64}", entry.get("sha256", "")):
            raise ValueError("invalid SHA-256")
        if entry.get("storage") == "git":
            continue
        if entry.get("storage") != "github-release":
            raise ValueError("unknown evidence storage")
        for key in ("release_id", "asset_id"):
            if type(entry.get(key)) is not int or entry[key] <= 0:
                raise ValueError("invalid release/asset identity")
        if entry["asset_id"] in assets:
            raise ValueError("duplicate release asset")
        assets.add(entry["asset_id"])
        if not re.fullmatch(r"[A-Za-z0-9._-]+", entry.get("tag", "")):
            raise ValueError("invalid release tag")
    return entries


def checked_path(root, entry):
    root = Path(root).resolve()
    path = root / entry["path"]
    if not path.resolve().is_relative_to(root):
        raise ValueError("evidence path escapes checkout")
    return path


def check_file(path, entry):
    if not path.is_file():
        raise ValueError(f"missing required evidence: {path}; run python scripts/fetch_evidence.py")
    h, size = hashlib.sha256(), 0
    with path.open("rb") as stream:
        while chunk := stream.read(CHUNK):
            h.update(chunk)
            size += len(chunk)
    if size != entry["bytes"] or h.hexdigest() != entry["sha256"]:
        raise ValueError(f"evidence mismatch; refusing to replace {path}")


def request(url, accept):
    headers = {"Accept": accept, "User-Agent": "SocialDeduction-frozen-evidence-v1"}
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        headers["Authorization"] = "Bearer " + token
    return urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=60)


def release_assets(entry):
    url = f"https://api.github.com/repos/{REPOSITORY}/releases/{entry['release_id']}"
    with request(url, "application/vnd.github+json") as response:
        release = json.load(response)
    if release["id"] != entry["release_id"] or release["tag_name"] != entry["tag"] or release["draft"]:
        raise ValueError("release identity changed or release is unpublished")
    return {asset["id"]: asset for asset in release["assets"]}


def asset_stream(entry, assets):
    asset = assets.get(entry["asset_id"])
    if not asset or asset["name"] != Path(entry["path"]).name or asset["size"] != entry["bytes"]:
        raise ValueError("recorded release asset is absent or changed")
    if asset.get("digest") != "sha256:" + entry["sha256"] or asset["state"] != "uploaded":
        raise ValueError("release asset digest/state mismatch")
    url = f"https://api.github.com/repos/{REPOSITORY}/releases/assets/{entry['asset_id']}"
    return request(url, "application/octet-stream")


def install_file(path, entry, stream):
    """Verify a temporary file, then link atomically without overwriting a target."""
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".frozen-", delete=False) as out:
            temporary = Path(out.name)
            size = 0
            while chunk := stream.read(CHUNK):
                size += len(chunk)
                if size > entry["bytes"]:
                    raise ValueError("download exceeds declared evidence size")
                out.write(chunk)
        check_file(temporary, entry)
        # Same-directory hard link provides create-once semantics on Windows/Linux.
        os.link(temporary, path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def retrieve(manifest, root, verify_only=False):
    entries = validate_manifest(manifest)
    releases = {}
    for entry in entries:
        path = checked_path(root, entry)
        if path.exists() or verify_only or entry["storage"] == "git":
            check_file(path, entry)
        else:
            key = entry["release_id"]
            if key not in releases:
                releases[key] = release_assets(entry)
            with asset_stream(entry, releases[key]) as stream:
                install_file(path, entry, stream)
            check_file(path, entry)
        print(f"verified {entry['path']} {entry['bytes']} {entry['sha256']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-only", action="store_true")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    manifest = json.loads((args.root / "evidence/manifest.json").read_text(encoding="utf-8"))
    retrieve(manifest, args.root, args.verify_only)


if __name__ == "__main__":
    main()
