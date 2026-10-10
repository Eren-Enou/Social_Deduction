# Experiment 008 validation

Completed on Windows x86_64, 2026-10-09. Scientific portability to Linux remains unverified.

## Executed gates

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed, including the post-freeze regression target.
- `cargo test --locked --all-features --release`: complete pre-freeze gate passed 124 tests (6 unit, 118 integration), zero failures/ignored tests. Experiment 008 contributed 29 integration tests. All historical 001-007 source/evidence replay contracts passed; 007 regenerated its complete primary evaluation. The 008 grid test regenerated 608 series/1824 incidents, compared complete reports/replay sections, checked pairing, counts and bootstraps. Logs: ignored local `target/experiment008-final-gate.log`.
- After freeze, `cargo test --locked --release --test experiment008_frozen`: one additional regression passed (85.20s observed test duration), bringing individually validated tests to 125 (6 unit, 119 integration). It checks primary raw hash, regenerates the complete 608-series evaluation, requires exact evaluation bytes, compares all Windows manifest sections, and regenerates both full cost-control reports and their raw hashes. The complete pre-freeze gate and additional post-freeze test were separate commands; no single post-freeze 125-test full-suite run is claimed.
- Actual CLI primary freeze: `cargo run --locked --release --bin experiment008 -- evaluate evidence/experiment-008.json`, successful create-new output after gates and committed source. A separate actual `target/release/experiment008.exe replay evidence/experiment-008-windows.json` regenerated the entire evaluation and canonical manifest on Windows. Both commands exited 0.
- CLI scenario/run smoke checks passed. Attempting to rewrite an existing full-report path failed with exit 1/AlreadyExists and its SHA-256 remained unchanged. No frozen output was overwritten.
- `git diff --check` and staged diff checks passed before each commit. Only 008 implementation/tests/evidence/design documentation plus the module export and README were changed. Historical Rust engines and tests were preserved; dependencies/toolchain/lockfile were not modified.

The early focused test pass initially found a real implementation issue: explicit finding disclosures inherited unrelated inference lineage and could be rejected for excessive depth. Direct acquisition/exact received-parent ancestry resolved that issue before primary freeze. Two test expectations were also corrected to match preregistered rules: unknown U3 still exceeds last-token C2; CostSensitive need not spend conserved effort in later confident conflicts, while ConflictDriven demonstrates a later inspection. Ordinary 005 inference-depth rejection was preserved as a measured legal resolver failure, not disabled. Final gates pass without suppressing Clippy diagnostics.

## Tested invariant coverage

1. Latent evidence initialization/policy-independent matched worlds and fixed generator replay.
2. Secret actor, content and root changes leave equal investigation inputs/actions unchanged.
3. No content in pre-access catalogs; no findings in first-round inputs or uninvestigated states.
4. Scoped acquisitions and private findings leave undelivered observers' inputs/actions unchanged.
5. Persistent effort conservation, exhaustion, voluntary decline/conservation and paid missing results.
6. Duplicate opportunity, illegal stage/ID, unavailable source, empty balance and malformed config rejection.
7. Rejecting custom investigation policies get no delivery/resource and no retry; custom provenance is unversioned.
8. Accurate/false evidence changes support and can create wrong/unresolved beliefs while truth stays fixed.
9. Maximum-weight artifact grouping, exact relays, known-parent merge, contested sources and hidden dependence.
10. Explicit report revisions retain environmental branch identity; report contribution does not inherit low speaker credit.
11. Forged/unreceived finding payloads, altered source/ownership, invalid/duplicate recipients and illegal communication phase rejection.
12. Selected/injected same-information controls with distinct initiative/resource origins.
13. Own episode acquisitions, memory identity/credit isolation, legal ballots, majority/tie rules and termination.
14. Executed local histories change investigation; scarcity and known uncertainty change declared selection.
15. Immutable past testimony/decisions, explicit environmental ancestry and full deterministic replay.
16. Environment reports do not create independent witness reputation; legally delivered participation briefing preserves original-judgment/anchor semantics.
17. Acquisition changes selected communications and executed votes in identical worlds.
18. Complete canonical sections cover resources, findings, cognition, history and actions; floating points are rejected.
19. Comparator rejects two same-OS manifests and changed evidence; synthetic fixtures are not actual OS evidence.
20. All 001-007 byte/evaluation regressions remain unchanged and passing; new primary/secondary frozen regression passes.

## Execution identity and hashes

Pinned `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0 (5f94df478 2026-08-27)`, architecture `x86_64`, OS `windows`. Actual frozen Windows execution records source revision `c2b4ee278cc2d52250b2dcde53ef84c7b6b4bee8`. Platform metadata is outside canonical simulation data.

| Artifact | Bytes | Raw SHA-256 |
|---|---:|---|
| Cargo.lock | unchanged | 360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92 |
| experiment-001.json | 2725107 | 5d3b9ed1b7ae71029bdfa2251fc702e1455c38db0e22550ec21282a2821e7356 |
| experiment-002.json | 11841127 | 08fec1b1685f8e9d5c064374bdec25c4ae6f67da08ae6a88c013a9e16fbb1d32 |
| experiment-003.json | 25296793 | 758c8afb75fd8b76450ee2b9470bdaf3d90a7849c4e05166002cc2674d6330bd |
| experiment-004.json | 46286603 | f8dee33c8282cc1caafde7d6b5c00cca68e001e96e70ee9b7641c62850a71fad |
| experiment-005.json | 72225029 | 3b30bdaec3496d5f4018f42a0d110cc7d67415ee3e0bf73b990ebc7d158e47e6 |
| experiment-006.json | 405837038 | 34e27342c14c1b09fe44cd268e9d9f5dc38d7335797bbdbdb32de4098fc5a360 |
| experiment-007.json | 117084630 | 1ce7952585ab3197243daf33b8ecc68dc54b071f7a6c508953d830ed61e21864 |
| experiment-007-windows.json | 892966 | 44cb24f848a90554056b4222752dfd4b15caa85e8f152e33eefe46cd1f5e7d3b |
| experiment-008.json | 61244605 | fa852e921f549dbb87344c7d3329a913985608b19a930cd456f0cc1c667a09cc |
| experiment-008-windows.json | 322181 | f0aef8d5b460cd2fe5ac68d1fb4429ae64a316c93b39b2c0621edbf9e3d1f381 |
| experiment-008-cost-control.json | 1923910 | 9d7447dd3367d9a9c2e3e66cc8e6c7759642db1aa7a244951f34cf17be7d7876 |

Canonical evaluation digest: `c0a2ac14657fff06b87e497285eca956310ba92a36494f0fd829b9bd5d9e254e`.
Canonical metric digest: `78b6338f453be6e91777a11403c7ec347452f20c96d401b21b3e36c529c46f33`.

Primary serialization uses canonical 008 prefix with unchanged 007 sorted-key float-free encoding; CLI evaluation bytes and all report sections replay exactly. Secondary controls preserve full reports and raw-report digests; the frozen regression independently regenerates them. Git attributes keep frozen JSON binary/no line-ending normalization. Output is create-once.

## Linux attempt and CI

`wsl --list --quiet` reported Windows Subsystem for Linux is not installed. `Get-Command docker -ErrorAction SilentlyContinue` found no Docker. No existing authorized execution environment was available for Linux; none was installed. No push was authorized or performed, and the committed GitHub workflow has not executed. Actual Linux test execution and Windows/Linux agreement are unverified.

The separate `.github/workflows/experiment008-replay.yml` preserves 007's workflow and pins the research toolchain/actions, runs formatting/Clippy/full release tests on Windows/Ubuntu, regenerates manifests independently, then compares actual OS/toolchain/revision/lock identity and complete canonical sections. It provides future infrastructure, not current scientific evidence. To compare real runs, use `cargo run --locked --release --bin experiment008 -- compare windows-manifest.json linux-manifest.json`; both must come from the same committed checkout/toolchain/lock. The frozen Windows manifest records the pre-data source/test commit; a later checkout needs fresh manifests on both platforms.

## Git record

Branch: `codex/experiment-008`. Commits: `eea133f` preregistration; `6b56a42` implementation/design decisions; `c2b4ee2` invariant tests and CI; `8fc3855` frozen primary/secondary/Windows evidence and regression. The final documentation commit hash is supplied in the completion response because a commit cannot contain its own final hash. No historical amend/reset or remote push. Final status is checked after that commit. No 009 work.
