# Experiment 010 validation

The accepted simulation/evaluation source is `0cf42d39bbe5eaeed477a4d4d9ef241ba54022a1`, on `codex/experiment-010`, following preregistration `1f73c50` and implementation `071e74b`. Primary outcomes were not consulted when choosing the generator, priors, thresholds, exploration schedule or comparisons. The baseline is `1ed74ee` (completed 009). No push or Experiment 011.

## Engineering gates

`cargo fmt --all -- --check` and `cargo clippy --locked --all-targets --all-features -- -D warnings` pass. The complete `cargo test --locked --all-features --release` on committed source passes **189 tests**, zero failed/ignored. Earlier development validation passed a complete 188-test suite and, after the final reporting additions, all 33 current 010 tests separately. These are executions, not additional distinct test cases. The development grid uses seeds 0-1 and diagnostic seed 0: 308 series/2,416 incidents, complete per-series replay, 252 paired contrasts and 14,400 predictive strata. It is not the held-out experiment.

Actual tooling is Rust `1.99.0 (b940084d7 2026-09-28)` and Cargo `1.99.0 (5f94df478 2026-08-27)`, Windows x86_64. Lock SHA-256 is `360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92`. Dependencies and pinned toolchain remain unchanged. Separate target directories were used for independent execution; they do not alter simulation inputs.

## Required invariant coverage

All tests below are in `tests/experiment010_{generation,engine,memory,policy,evaluation}.rs`. Historical legality, communication, belief, voting and termination contracts additionally run through the complete 001-009 suite.

| Requested invariant | Executed checks |
|---|---|
| 1. Stable family identity | Generation recurrence and eight-incident learning |
| 2. Distinct incident artifacts | Episode-scoped acquisitions, generated identity checks |
| 3. No quality in identifiers | Hidden-parameter perturbation, local-input field checks, randomized identities |
| 4. Results before choice | Generated configs sealed before policy treatment |
| 5. Same latent worlds | All policy configurations and environment digests |
| 6. Received-only learning | NeverInvestigate, recipient isolation, private histories |
| 7. Missingness without invented accuracy | Missing direct result updates only observed/missing counts |
| 8. Eligible accepted feedback | Accuracy prior and actual 007 receipt acceptance |
| 9. Absent feedback | Missing, undelivered, unauthenticated, pending-before-acquisition cases |
| 10. False authenticated feedback | Corrupted counts and executed clean/false current-input controls |
| 11. Conflicting verification | Retained quarantine, order and immutable earlier snapshots |
| 12. Duplicates | Exact acquisition repetition, repeated tokens, shared visible roots |
| 13. Known dependence | First eligible owner of a visible episode/root, duplicate ineligible |
| 14. Hidden dependence | Hidden roots absent from updater inputs; distinct visible roots retain credit |
| 15. Individual locality | One inspector's memory differs while nonrecipients remain empty |
| 16. Identical histories | Equal local inputs/estimates, complete deterministic replay |
| 17. Different histories | Executed fixed-current histories change estimates and choices |
| 18. Public claims | Advertised quality changes conservative choice with unchanged local latent conditions |
| 19. Hidden quality | Sealed content/process changes cannot change first preacquisition choices |
| 20. Real costs | One-to-three-unit charges, missing results, free injection separation, conservation |
| 21. Legal exploration | Affordable/access filters, empty balance, wrong stage and decline |
| 22. Real regime changes | Changed future process probabilities/artifacts, preserved earlier snapshots |
| 23. Reversible preferences | One success then five accepted failures changes voluntary later target; absent-feedback control does not |
| 24. Bounded memory | FIFO/checkpoints, token/receipt bounds, late/evicted/pending references, malformed IDs |
| 25. Immutable past | Earlier decisions, memory snapshots, claims and outcomes retain their serialized values |
| 26. Complete replay | Exact reports plus seven full/section hashes; independent canonical reference |
| 27. Historical preservation | All thirteen accepted frozen hashes; unchanged 001-009 source/tests/dependencies/toolchain |

Additional checks cover zero-observation priors, maximum u16 arithmetic, JSON method-map round trips, semantic ties under source reordering, unknown method IDs, series/budget limits, conditional-accuracy label exclusion, exact Brier/bin endpoints and zero-denominator cells. Matched training verifies equal delivered effects and local memories across selectors, rather than installing synthetic trust scores. The eligibility projection supplies neither assertions nor a source identity to the unchanged 007 acceptance code.

An independent Python `hashlib` plus compact sorted JSON reference matches the new canonical fixture digest `c9edb8663f835d677f410a22d8e4e151a06eb31c0dc3d1e864f42f69f14bdae7` for `{"a":2,"z":1}`. The canonical prefix is `canonical-010-v1\n`; unchanged 007 encoding rejects floating-point values. Method identities serialize as version/key strings.

## Platform boundary

`wsl --list --quiet` reports that Windows Subsystem for Linux is not installed; Docker is unavailable. Linux execution and Windows/Linux agreement are **unverified**. No OS was installed, no Linux manifest invented and no remote CI triggered. `.github/workflows/experiment010-replay.yml` pins the established toolchain/actions, runs actual Windows/Ubuntu suites and generates new manifests at one common revision before comparing them. This workflow's existence is not evidence that it ran. The comparator rejects two Windows manifests and mismatched revision, toolchain, architecture or lock.

## Development failures and limits

Compile-time field/type/lifetime mismatches in the new reporting code and test scaffolding were corrected before primary evaluation; Clippy style findings were resolved. A concurrent rebuild hit Windows linker LNK1104 because a test executable was already running. The active tests were allowed to finish; an isolated target directory and sequential rebuild resolved the executable lock. No failed assertion was bypassed, accepted history amended, or held-out distribution/policy tuned. This transient build failure is not an experimental outcome.

The boundary is a narrow typed interface for trusted supplied Rust components, not a hostile native-code sandbox or cryptographic verification implementation. Full observer reports intentionally contain secrets. Compact frozen traces retain captured forecasts, update effects, input hashes and complete-report/section digests; exact committed scenario generation and `run` regenerate complete decision-time views for audit. They must be checked against original hashes, rather than explained using a changed policy or reconstructed cognition from an audit archive. An individual's bounded memory never receives that archive.

## Freeze and final validation

Primary evaluation and independently compiled Windows replay completed on the same source revision, `0cf42d39bbe5eaeed477a4d4d9ef241ba54022a1`. Both execute all 3,632 series / 28,864 incidents, with two equal complete reports per series. Full evaluation, metrics and all seven complete/section canonical digests agree. Independent Python sorted compact JSON plus SHA-256 checks agree with both actual manifests, world matching, conditional labels and resource conservation.

| Frozen artifact | Bytes | Raw SHA-256 |
|---|---:|---|
| experiment-010.json | 972371932 | `68d25a4799f79d21aff002da6867132fc1a0d69975542e3a5bc5f7d9df0905b6` |
| experiment-010-windows.json | 2270551 | `81d712affd6dd39346e0af475c8dfdbcf7cce4920e62dbf4f856eadb3dbebed7` |

Canonical evaluation digest: `98d21fe1d543f138b7e414bd5ec915235c2dd5ab3b99be19ba488bce65cf67be`. Canonical metrics digest: `36a411d3e8a5cb2ed842860ed6aeca7ce88280dcd4e787d1687eb5d8faeea2c1`. Evidence was copied once with overwrite refusal after complete source gates and independent agreement. No frozen artifact was rewritten.

The later `6315247` change buffers create-once file writes only; `src/experiment010` is identical to primary source. Its separate CLI regression passes exact serialization bytes and overwrite refusal. Formatting and all-target/all-feature warnings-as-errors Clippy also pass with the added frozen test target. The full-grid frozen regression passes: `cargo test --locked --release --test experiment010_frozen`, one test, zero failed/ignored, 1516.96 seconds. It regenerates all 3,632 series twice, compares complete evaluation values and exact frozen bytes, checks the actual Windows manifest plus all complete/section digests, and rejects a same-OS comparison as proof of Linux agreement.

There are **191 distinct individually validated tests**: the complete 189-test release gate, then the separate one-test output-byte/overwrite check and one-test full-grid frozen regression. This is not a claim that a combined 191-test command was executed. No simulation/scoring change followed the complete gate. Final formatting and warnings-as-errors all-target/all-feature Clippy pass; thirteen historical artifact hashes and historical source/tests/dependencies/toolchain remain unchanged.

Frozen evidence and regression are committed in `50b2db8`; final findings, validation and navigation are committed separately. All work remains local on `codex/experiment-010`, with no push or history rewrite. The primary common-source revision remains `0cf42d3`; the data/documentation commit IDs are not substituted into the historical execution metadata.

### Historical artifact integrity

All thirteen byte counts and SHA-256 values below match the accepted baseline before and after 010 freeze. Earlier source, tests, dependencies and toolchain also match baseline `1ed74ee`.

| Artifact | Bytes | SHA-256 |
|---|---:|---|

| evidence/experiment-001.json | 2725107 | `5d3b9ed1b7ae71029bdfa2251fc702e1455c38db0e22550ec21282a2821e7356` |
| evidence/experiment-002.json | 11841127 | `08fec1b1685f8e9d5c064374bdec25c4ae6f67da08ae6a88c013a9e16fbb1d32` |
| evidence/experiment-003.json | 25296793 | `758c8afb75fd8b76450ee2b9470bdaf3d90a7849c4e05166002cc2674d6330bd` |
| evidence/experiment-004.json | 46286603 | `f8dee33c8282cc1caafde7d6b5c00cca68e001e96e70ee9b7641c62850a71fad` |
| evidence/experiment-005.json | 72225029 | `3b30bdaec3496d5f4018f42a0d110cc7d67415ee3e0bf73b990ebc7d158e47e6` |
| evidence/experiment-006.json | 405837038 | `34e27342c14c1b09fe44cd268e9d9f5dc38d7335797bbdbdb32de4098fc5a360` |
| evidence/experiment-007-windows.json | 892966 | `44cb24f848a90554056b4222752dfd4b15caa85e8f152e33eefe46cd1f5e7d3b` |
| evidence/experiment-007.json | 117084630 | `1ce7952585ab3197243daf33b8ecc68dc54b071f7a6c508953d830ed61e21864` |
| evidence/experiment-008-cost-control.json | 1923910 | `9d7447dd3367d9a9c2e3e66cc8e6c7759642db1aa7a244951f34cf17be7d7876` |
| evidence/experiment-008-windows.json | 322181 | `f0aef8d5b460cd2fe5ac68d1fb4429ae64a316c93b39b2c0621edbf9e3d1f381` |
| evidence/experiment-008.json | 61244605 | `fa852e921f549dbb87344c7d3329a913985608b19a930cd456f0cc1c667a09cc` |
| evidence/experiment-009-windows.json | 679786 | `2c38de80a86ba76db54d0b0c918e00f4710730a7e7fc81cc08b3c2b4045d4169` |
| evidence/experiment-009.json | 125151900 | `157019ee5c078b609f34dee295ce4e6415e9411367f13ea77c29aa48183b1872` |
