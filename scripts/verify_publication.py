#!/usr/bin/env python3
"""Audit every local Git blob, including historical/unreachable objects."""
import subprocess

objects = subprocess.check_output(
    ["git", "cat-file", "--batch-all-objects", "--batch-check=%(objectname) %(objecttype) %(objectsize)"],
    text=True,
).splitlines()
blobs = [(oid, int(size)) for oid, kind, size in (line.split() for line in objects) if kind == "blob"]
oversized = [(oid, size) for oid, size in blobs if size > 100 * 1024 * 1024]
if oversized:
    raise SystemExit("oversized ordinary Git blobs remain: " + repr(oversized))
print(f"verified {len(blobs)} Git blobs; none exceed 100 MiB; largest={max(size for _, size in blobs)} bytes")
