# Publication migration validation

Validated implementation commit `f61f7fe5069862040de06c180e9e6c7c0e8fc155` is on the separate `codex/publish-experiment-012` branch. The original accepted `codex/experiment-012` remains clean at `f13bb0998a3bf6f3e63299e276015becf1452aa5`. No original branch, accepted commit, frozen artifact, engine, policy, Rust test, lockfile or toolchain was rewritten. No Experiment 013.

## Actual gates

The migrated checkout completed `cargo test --locked --all-features --release`: **267 passed, zero failed, zero ignored**, including every historical frozen regression and the complete Experiment 012 frozen regression. This is a combined 267-test execution, unlike the original 266-plus-separate-one completion record. Both records remain factually distinct. Formatting and locked all-target/all-feature Clippy with warnings denied passed on the migrated checkout. All 12 Python retrieval tests passed.

The retrieval tests verify exact installation, corrupt/truncated/overlong downloads, missing required evidence, changed release/asset identities, path traversal, malformed manifests, duplicate paths and refusal to overwrite an existing or concurrently created file. Existing Rust regressions still demand their exact original raw data; there is no conditional skip or regenerated-evidence fallback.

```powershell
python scripts/fetch_evidence.py
python scripts/fetch_evidence.py --verify-only
python scripts/verify_publication.py
python -m unittest discover -s scripts/tests -v
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --release
```

Actual environment: Windows x86_64, Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo `1.99.0 (5f94df478 2026-08-27)`, Python 3.12.6. Unchanged Cargo.lock SHA-256 `360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92`.

## Preservation, backup and history

Before migration, an all-ref Git bundle was created and verified, restored to a mirror, and checked with `git fsck --full`. All 19 frozen JSON artifacts were separately copied and independently SHA-256 verified. The complete backup remains at `C:/Users/Aaron/CoderVibe/SocialDeduction-publication-backup-f13bb09-20261010`. Bundle SHA-256 `499d70c32bff2553fe171ba48495a10da10452bd41664fe91368e118efb0fc3f`. [Backup manifest](backup-manifest.json) records exact original files, sizes, hashes and the original oversized-object inventory.

The original scan used `git cat-file --batch-all-objects`, covering historical and unreachable objects, rather than only files in HEAD. Exactly six blobs exceed 100 MiB; each is an original frozen JSON artifact, and each was uploaded unchanged. No historical oversized version was omitted. Sanitization occurred only in an isolated repository. All 35 source-history commits map in [commit-map.json](commit-map.json); every corresponding tree was checked excluding only those six paths, with authors, committers, timestamps and messages preserved. The current migrated source/test/experiment/configuration audit separately verifies **142 identical original Git blobs**.

The first isolated import failed on a data-record separator after removing a blob. Its diagnostic directory was retained and a fresh import corrected the separator. Accepted original history and evidence were unaffected. Earlier ordinary publication attempts failed with HTTP timeouts and then GitHub's oversized-file rejection; no original branch was published or force-pushed.

The all-object audit on the sanitized repository finds **no blob above 100 MiB**. The largest remaining ordinary Git blob is 72,225,029 bytes. The original main commit is retained as an ancestor, permitting a PR without rewriting main. The PR includes accumulated Experiments 006–012 because main predates them, plus the publication migration.

## Release storage and deterministic retrieval

Release ID **409465057**, tag `frozen-evidence-006-012-f13bb09`, is [published release storage](https://github.com/Eren-Enou/Social_Deduction/releases/tag/frozen-evidence-006-012-f13bb09). Its tag is anchored to existing remote main `6df52b493c94dd224d123cad8ccad3a0319b1f50`, not to a falsely claimed experiment-source commit. The six original files were uploaded, API sizes/digests checked, then independently downloaded into an initially artifact-free migrated checkout using the retrieval script. All 19 resulting files matched original lengths and SHA-256 values. The thirteen smaller artifacts remain in Git.

Numeric release/asset identifiers are fixed references; SHA-256 pins content. The GitHub release reports `immutable: false`: repository-level immutable-release enforcement is not claimed or newly enabled. Deleted/replaced assets, changed metadata or mismatched content fail retrieval. Storage arrangement changes require separate approval. Consumers install only verified bytes atomically, never overwrite mismatched files and never silently skip required evidence.

| Release artifact | Asset ID | Bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `evidence/experiment-006.json` | 629967529 | 405837038 | `34e27342c14c1b09fe44cd268e9d9f5dc38d7335797bbdbdb32de4098fc5a360` |
| `evidence/experiment-007.json` | 629967522 | 117084630 | `1ce7952585ab3197243daf33b8ecc68dc54b071f7a6c508953d830ed61e21864` |
| `evidence/experiment-009.json` | 629967530 | 125151900 | `157019ee5c078b609f34dee295ce4e6415e9411367f13ea77c29aa48183b1872` |
| `evidence/experiment-010.json` | 629967524 | 972371932 | `68d25a4799f79d21aff002da6867132fc1a0d69975542e3a5bc5f7d9df0905b6` |
| `evidence/experiment-011.json` | 629967525 | 233454880 | `001bfe314e3c65e7c43f886a75836e9aceff4153021595fe4ac7992ca5ea262f` |
| `evidence/experiment-012.json` | 629968390 | 300005474 | `f22a61dbeca029dd29977228805ab9c932f5e834433c61c8edaca1da0157b4f2` |

Committed Git-blob SHA-256 for [evidence manifest](../evidence/manifest.json): `151ebac480e0386a76847b9d12d7ca47cc3df55ca12ddf4ed4d549c4e9fa85f9`. Committed Git-blob SHA-256 for [complete commit mapping](commit-map.json): `52e32d0bfebc50ff028a3bcae0b0bc327ab854298e0e6ba868a74543cb2b3475`. These metadata hashes refer to committed Git bytes, independent of checkout text-line conversion.

## Independent deterministic replay

A separate actual Windows execution on migrated commit `f61f7fe5069862040de06c180e9e6c7c0e8fc155` regenerated the complete Experiment 012 population, running every series twice. Its entire canonical manifest matches the accepted original: 696 series / 5,568 incidents, all evaluation/metric and eight section hashes per series. The [migration replay record](migrated-windows-replay.json) differs from the original Windows JSON only by the truthful execution revision. OS, architecture, toolchain, lock hash and all canonical evidence remain identical.

Migration replay bytes: 466173; raw SHA-256 `ba48dc0256d621641399b3891e4afb27d985037b45e76c381b5040cbb475620d`. Evaluation digest `7f59152e77d500b07b038d4551aa4f0fc69bf241a2edbfb72d5c1d614c905e9b`; metric digest `323b83810219245c3aad5d33a0f21befcd74a931454f59dd36380a4010e44a6d`. Frozen original platform evidence was never edited to substitute a new revision.

## CI and publication status

All six existing Windows/Ubuntu replay workflows retrieve/verify required evidence, audit ordinary Git blob size and run retrieval tests before their unchanged formatting, Clippy, complete Rust suite and independently generated replay comparison. No workflow silently skips a missing frozen regression. Local Linux execution remains unverified; prepared CI is not proof of Linux agreement. Actual CI status must be read from the resulting PR.

The publication branch may be pushed and its PR opened only after these local gates and preservation audits succeed. No force-push, original-history rewrite, automatic merge or Experiment 013 is authorized. The final documentation commit and remote PR state are reported by their actual Git/GitHub results; no self-referential final commit hash is invented here.
