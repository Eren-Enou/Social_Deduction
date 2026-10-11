# Experiment 011 validation

The preregistration was committed as `b689d92` before implementation. The validated simulation, fixtures and evaluation source was committed as `ba439e8a06a9bb2ad90a94aff0e0fbd9ab54ff61` before primary execution. Baseline `99b8a1e` completed Experiment 010. No held-out policy, fixture or threshold tuning, push or Experiment 012.

## Engineering gates

The complete locked all-feature release suite on committed source passed **224 tests**, zero failed or ignored. After create-once freeze, the newly added complete frozen 011 regression passed separately: **225 distinct validated tests**. No combined 225-test execution is claimed. Formatting and all-target/all-feature warnings-denied Clippy passed again with that regression included. The pre-primary focused release gate passed all 32 Experiment 011 integration tests and the create-once CLI test, including the complete 104-series development grid and matched controls. Formatting and locked all-target/all-feature Clippy with warnings denied passed on the committed implementation.

Required commands:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --release
```

Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo `1.99.0 (5f94df478 2026-08-27)`, Windows x86_64. The unchanged lock SHA-256 is `360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92`. The complete current suite uses `CARGO_TARGET_DIR=target/experiment011-validation` to avoid rebuilding running historical Windows executables. Target paths do not enter simulation state.

## Required invariant coverage

The 32 tests in [experiment011.rs](../../tests/experiment011.rs) cover the following requirements; historical tests additionally enforce original legal actions, transitions, voting, termination, feedback, memories and resources.

| Requirement | Executed check |
|---|---|
| 1. Typed bounded reports | Version, bounded counts, recipients, references and family checks |
| 2. No unacquired speaker experience | Empty local memory produces no sincere personal report; narrow sharing inputs |
| 3. Retained sincere information | Report counts equal the captured own summary, not world truth |
| 4. Hidden misleading intent | Wire field exclusions and changed private intent with identical recipient decisions |
| 5. No invented personal samples | Received social records leave own graded counters empty |
| 6. Verification assertion is not feedback | Asserted verification transfers no token or accepted personal grade |
| 7. Legal recipient scope | Declared recipients only; duplicate/foreign recipients rejected |
| 8. Undelivered no effect | Same nonrecipient method assessments and voluntary actions |
| 9. No repeated independent support | Same-speaker latest branch, exact duplicate ingestion, returned source echo |
| 10. Visible relay relationship | Retained parents and one recipient-visible origin group |
| 11. Hidden origins inaccessible | Identical wires under changed hidden intent; hidden copies remain separate visible groups |
| 12. Unknown origin not verified | Plausible claimed origin accepted as an unverified association |
| 13. Immutable correction history | Original wire unchanged after correction and retraction |
| 14. Own revision only | Foreign, future, missing and wrong-family revision targets rejected |
| 15. Scoped correction | Only declared recipients change current standing |
| 16. Different social histories | Executed clean/false sender feedback with equal present recipient observations/catalog/balance |
| 17. Equal local histories | Equal interpretation and full recipient decisions despite changed private selected intent |
| 18. Future influence | Received favorable report changes a later affordable source choice |
| 19. No retroactive choice | After-investigation report leaves the earlier resolved choice unchanged |
| 20. Real resource costs | Delivery charges, old finding weights, exhausted balance, finite legal schedule |
| 21. Separate credibility | Own 006 scores weight advice without adding 010 personal counts |
| 22. Sincere corruption | False authenticated feedback produces locally sincere false method assessments |
| 23. Conflict uncertainty | Contested revisions/groups add zero; cap and strong own evidence preserved |
| 24. Bounds and eviction | FIFO 12/8 bounds, absent original, depth four rejected, future IDs, exhausted speech |
| 25. Distinct controller origin | Same selected/injected payload and effects retain different audit origins |
| 26. Complete replay | All diagnostics, development population, captured snapshots and seven canonical sections |
| 27. Historical preservation | Complete historical suite, fifteen original raw hashes and unchanged historical tracked code/test/dependency/toolchain bytes |

[CLI test](../../tests/experiment011_cli.rs) verifies exact JSON configuration output, runnable headless reports and refusal to overwrite an existing file. Additional checks exercise two controlled speaker-role/order permutations, actual relay depth rather than truth-based origin validation, missing labels excluded from conditional accuracy, and same serialized visible message with different private basis/intent. These are trusted Rust component boundaries, not a hostile native-code sandbox.

## Replay and independent reference

Primary `evaluate` and a separate Windows `replay` process run the full committed population. Each series internally executes twice and compares complete reports before retaining compact traces. The replay separately regenerates the entire evaluation; both processes use the same committed release executable and pinned environment. This is independent execution, not independent compiler or operating-system evidence.

An independent Python `hashlib` plus sorted compact UTF-8 JSON reference rejects floating-point inputs and checks the complete evaluation, comparison/prediction metric digest, all 952 sets of seven section hashes, matching world environments, ballot/verdict denominators and actual paid-resource totals. It also independently reconstructs all seven sections of a complete `AccurateFirsthand/ProvenanceAwareSocial` report and compares them with its frozen row. A post hoc routing audit regenerated the 37 RelayProvenance series with falsely separate support and independently matched all original full-series hashes before inspecting captured local inputs; all 74 affected pairs involved parent messages never addressed to those recipients. The prefix is `canonical-011-v1\n`; the unchanged 007 float-free encoding is reused.

Compact evidence contains the original captured actions, effects, personal counts, social interpretations, input hashes and full/section hashes. Complete views can be regenerated from the exact committed `scenario`/`run` code and must match the original hashes. Regeneration is an observer audit operation, never a reconstruction of evicted cognition for a policy or an explanation produced by a later policy version.

## Platform limitation

`wsl --list --quiet` reports Windows Subsystem for Linux is not installed. Docker is unavailable. No authorized Linux execution host is exposed. Linux replay and Windows/Linux agreement remain **unverified**. The new [workflow](../../.github/workflows/experiment011-replay.yml) follows the established workflow, pins actions/toolchain, executes actual Windows/Ubuntu suites and compares fresh manifests at one common revision; it has not run. A second Windows manifest cannot satisfy the comparator.

Closing this limitation requires an available Linux x86_64 host with the pinned toolchain, unchanged lockfile and the same committed source, or explicit authorization to push/dispatch the repository workflow with an available CI runner. No OS installation, invented Linux manifest or unauthorized push was attempted.

## Development failures

Initial reporting scaffolding had export/type/field mismatches, corrected before primary execution. Three development assertions initially failed: a nonrecipient comparison included a forced training choice, a verification test incorrectly required an empty legitimate pending-token journal, and a credibility fixture privately learned new evidence then publicly retracted its original assertion. The first two assertions were corrected to test the intended invariant; the fixture's second-round revision was routed privately before primary evaluation so its audience's earlier ordinary claim could be legitimately graded. These were not held-out improvements or weakened historical checks. Clippy findings were fixed in new files only. One attempted overlapping log redirection failed because another process held that file; the retry used a new log path. No test was reported passed from that failed invocation.

## Freeze and final checks

The initial historical baseline finished with 191 passing tests. The complete current 224-test gate returned exit 0 after regenerating all historical frozen artifacts and the new development population. Formatting, pre-freeze Clippy, independent canonical checks and historical byte checks passed before the create-once copies. The new frozen regression then returned exit 0: it regenerated all 952 series, compared the complete evaluation and exact serialized bytes, every section/metric manifest and recorded Windows provenance. Postfreeze formatting and warnings-denied all-target/all-feature Clippy include the new test. No kernel/policy/generator/threshold change followed consultation of held-out outcomes.

| Frozen artifact | Bytes | Raw SHA-256 |
|---|---:|---|
| experiment-011.json | 233454880 | `001bfe314e3c65e7c43f886a75836e9aceff4153021595fe4ac7992ca5ea262f` |
| experiment-011-windows.json | 555565 | `12e85f955e09c2faad15f846e091e2ace26fc5265f5a2ef3c4025fcadab5e6a9` |

Complete evaluation canonical SHA-256: `cb9a03a2104d2e4a2f94beb2ae575d5ba9fd7aeaef7ef86f41bd3aeabc459766`. Comparisons/prediction-strata canonical SHA-256: `54c2a5587923d2e38f9ae239cfa8c8f719a7c3025e84a28a01c50505085472bb`. All 6,664 complete/section digests match the independent Windows manifest and Python reference. Windows execution metadata records exactly the committed source revision `ba439e8a06a9bb2ad90a94aff0e0fbd9ab54ff61`; metadata is outside simulation canonicalization.

Historical raw hashes below were captured before edits, verified during development, before freeze and after the new regression. Historical engine/test/dependency/toolchain tracked bytes match baseline `99b8a1e`; the sole historical-library addition exports the new module. New source/tests, appended documentation and two new frozen files are the task changes. Historical documentary prefixes were preserved rather than re-encoded.

| Historical artifact | Bytes | Unchanged raw SHA-256 |
|---|---:|---|
| experiment-001.json | 2725107 | `5d3b9ed1b7ae71029bdfa2251fc702e1455c38db0e22550ec21282a2821e7356` |
| experiment-002.json | 11841127 | `08fec1b1685f8e9d5c064374bdec25c4ae6f67da08ae6a88c013a9e16fbb1d32` |
| experiment-003.json | 25296793 | `758c8afb75fd8b76450ee2b9470bdaf3d90a7849c4e05166002cc2674d6330bd` |
| experiment-004.json | 46286603 | `f8dee33c8282cc1caafde7d6b5c00cca68e001e96e70ee9b7641c62850a71fad` |
| experiment-005.json | 72225029 | `3b30bdaec3496d5f4018f42a0d110cc7d67415ee3e0bf73b990ebc7d158e47e6` |
| experiment-006.json | 405837038 | `34e27342c14c1b09fe44cd268e9d9f5dc38d7335797bbdbdb32de4098fc5a360` |
| experiment-007-windows.json | 892966 | `44cb24f848a90554056b4222752dfd4b15caa85e8f152e33eefe46cd1f5e7d3b` |
| experiment-007.json | 117084630 | `1ce7952585ab3197243daf33b8ecc68dc54b071f7a6c508953d830ed61e21864` |
| experiment-008-cost-control.json | 1923910 | `9d7447dd3367d9a9c2e3e66cc8e6c7759642db1aa7a244951f34cf17be7d7876` |
| experiment-008-windows.json | 322181 | `f0aef8d5b460cd2fe5ac68d1fb4429ae64a316c93b39b2c0621edbf9e3d1f381` |
| experiment-008.json | 61244605 | `fa852e921f549dbb87344c7d3329a913985608b19a930cd456f0cc1c667a09cc` |
| experiment-009-windows.json | 679786 | `2c38de80a86ba76db54d0b0c918e00f4710730a7e7fc81cc08b3c2b4045d4169` |
| experiment-009.json | 125151900 | `157019ee5c078b609f34dee295ce4e6415e9411367f13ea77c29aa48183b1872` |
| experiment-010-windows.json | 2270551 | `81d712affd6dd39346e0af475c8dfdbcf7cce4920e62dbf4f856eadb3dbebed7` |
| experiment-010.json | 972371932 | `68d25a4799f79d21aff002da6867132fc1a0d69975542e3a5bc5f7d9df0905b6` |

## Confirmed Git work

Dedicated branch `codex/experiment-011`, initially clean. Reviewed local commits, oldest first:

- `b689d92e86402c1126d394f83e943d226830567c` - `docs(experiment-011): preregister social evidence transmission hypotheses`
- `ba439e8a06a9bb2ad90a94aff0e0fbd9ab54ff61` - `feat(experiment-011): add voluntary method testimony and bounded local interpretation`
- `1333d30873589c4338a30eeb3fb2f2f9962545ef` - `data(experiment-011): freeze validated social transmission evidence`

The final findings/validation/README/architecture/decisions/protocol/roadmap documentation is delivered by the subsequent `docs(experiment-011): report social transmission findings and limitations` commit. Its confirmed hash appears in the final delivery and Git log rather than a self-referential field inside that commit. Final staged diffs and working-tree status are checked at delivery. No accepted commit was amended or rewritten, no frozen artifact overwritten, no push made, and no Experiment 012 started.
