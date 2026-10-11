# Retrieving frozen evidence

Six original JSON files exceed GitHub's ordinary Git file limit. They are preserved unchanged as assets of [storage release 409465057](https://github.com/Eren-Enou/Social_Deduction/releases/tag/frozen-evidence-006-012-f13bb09). The other thirteen JSON artifacts remain in Git.

Before running the complete Rust tests or frozen regressions, use Python 3.10 or newer from the checkout:

```powershell
python scripts/fetch_evidence.py
python scripts/fetch_evidence.py --verify-only
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --release
```

The deterministic manifest is [manifest.json](manifest.json). It records all 19 artifacts' exact byte lengths and SHA-256 digests, plus fixed numeric release and asset identifiers for the six downloads. Downloads are requested by asset ID, checked against the recorded release/tag, asset name, size and GitHub digest, streamed to a temporary file, independently hashed, then installed atomically without overwriting an existing file. Correct existing files are verified and retained; missing Git-managed evidence, mismatched files, missing assets or unpublished releases are errors. `--verify-only` never downloads or skips absent files. There is no automatic fallback to regenerated evidence.

| Artifact | Asset ID | Bytes |
| --- | ---: | ---: |
| experiment-006.json | 629967529 | 405837038 |
| experiment-007.json | 629967522 | 117084630 |
| experiment-009.json | 629967530 | 125151900 |
| experiment-010.json | 629967524 | 972371932 |
| experiment-011.json | 629967525 | 233454880 |
| experiment-012.json | 629968390 | 300005474 |

Numeric identifiers pin the uploaded objects; SHA-256 pins the accepted content. GitHub reports this release's `immutable` setting as false: this migration does not claim repository-level immutable-release enforcement. Deleted/replaced assets acquire different IDs or fail retrieval, and changed metadata/content fails verification. Never replace a frozen asset or silently update an accepted digest. Future evidence needs a separate versioned release and manifest.

The tag is a storage anchor on the existing remote main commit `6df52b493c94dd224d123cad8ccad3a0319b1f50`, not the experiment source commit. Original source HEAD is `f13bb0998a3bf6f3e63299e276015becf1452aa5`. [Commit mapping](../publication/commit-map.json) preserves original research provenance after the separate publishing-history migration. Scientific findings and platform manifests retain their original source identifiers.

All replay workflows retrieve and validate required evidence before running the unchanged frozen regressions. Direct test execution without required evidence fails; no regression was weakened or conditionally skipped.
