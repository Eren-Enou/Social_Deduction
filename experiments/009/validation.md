# Experiment 009 engineering validation

Executed 2026-10-10 on Windows x86_64, pinned rustc 1.99.0(b940084d7 2026-09-28), cargo 1.99.0(5f94df478 2026-08-27). Cargo.lock SHA-256 360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92. No dependencies/toolchain/lock changes. Scientific findings are in results.md; this is engineering validation, not independent review.

## Completed gates

- `cargo fmt --all -- --check`: passed, including the added frozen target.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: passed, including the added frozen target.
- `cargo test --locked --all-features --release`: complete pre-freeze gate passed 155 tests (six unit/149 integration), zero failures/ignored tests. Historical 001-008 contracts all passed; 009 contributes 30 integration tests. Actual 009 harness duration 119.28s. Ignored local log:target/experiment009-final-gate.log.
- Primary grid: 960 series/2880 incidents replayed with complete report equality and six canonical digests each, environmental pairing, totals, world-level intervals and legal decisions. Separate selected/forced/injected controls and observer alternatives bring canonical coverage to 1284 series/3828 incidents, all replayed.
- Actual create-once freeze after successful gates and committed source:`target/release/experiment009.exe evaluate evidence/experiment-009.json`, exit 0. Separate `target/release/experiment009.exe replay evidence/experiment-009-windows.json`, exit 0, independently recomputed the complete evaluation on Windows.
- Post-freeze `cargo test --locked --all-features --release --test experiment009_frozen`: one added regression passed, actual test duration 110.15s. It compares raw hashes, entire evaluation values/bytes, Windows source/lock provenance, all canonical sections, counts 1284/3828 and manifest consistency. Individually validated total 156 tests (six unit/150 integration); the complete 155-test gate and one added test were separate commands. No single post-freeze 156-test full-suite run is claimed.
- Independent Python integer/key-sorted/UTF-8 reference matches Windows evaluation and metric digests. Full Rust report replays establish the section comparisons; Python did not execute a Linux simulation.
- CLI scenario/config/full-report regeneration matches library output and refuses overwrites while preserving prior bytes. Invalid configs and unaffordable actions fail without fallback. No frozen file was overwritten.
- Historical 001-008 engine/test sources, Cargo.toml/Cargo.lock/toolchain have zero diff against 6dfef69. Only historical Rust change is the 009 module export. All 11 historical artifact hashes below match the predevelopment baseline and frozen regressions. Git diff/check and staged review passed; only authorized 009 implementation/tests/evidence/design/navigation changed.

## Coverage and development failures

Tests establish pre-treatment latent generation and equal worlds across policies; hidden content/actor/ancestry invariance for all seven policies; absent uninspected/nonrecipient findings; legitimate descriptors/quality/cost/relevance/known-overlap changes; at least two legal targets and voluntary decline; random target selection distinct from random whether-to-investigate; own stream replay; source-order/ID/agent permutations; semantic ties; paid prices 1/2/3; missing/false costs; all-unaffordable/empty catalogs; no cheap fallback/retry; stage/use/access/identity guards; malformed/duplicate IDs/keys/coverage/access, price 0/overflow and exhausted budgets; resource conservation/reservation across episodes; duplicate artifact max-weight and known/hidden ancestry distinctions; identical selected/forced/injected finding updates with distinct costs/origins; explicit independent-acquisition disclosure; immutable snapshots/namespaces; ballot execution/four-vote majority/ties/abstention/termination; full canonical section integrity; observer-only alternatives; CLI create-once; unchanged historical contracts and new frozen replay.

Resolved failures are retained: initial Clippy found a collapsible conditional. An early conservation assertion wrongly expected the InformationValue policy to decline every unopposed belief. Its preregistered possible-change formula may value a rival-induced tie; the assertion was corrected to test the separately declared BudgetAware reservation rule, without changing thresholds or held-out worlds. Focused 23 then28 checks passed. Pre-primary review found unequal optional provenance in EqualSources/CostDifference and a physical source labeled Derived. These fixture metadata corrections were documented/committed before held-out execution, and an explicit matching/generation test was added. The earlier complete-suite attempt was deliberately interrupted during historical 007 replay; it is not a completed gate. The subsequent full 155-test gate passed. No empirical adverse/null result was removed or caused tuning after the held-out run.

Built-ins produce zero investigation rejections but1669 primary communication rejections under inherited depth-three rules. Those are measured legal resolution failures consuming a turn, not failed engineering assertions. Custom policies remain caller-supplied-unversioned and cannot support a hostile-host sandbox claim. Output consistency checks do not prove arbitrary custom cognition semantics. Public separate acquisition metadata is not a statistical-independence guarantee.

## Canonical and platform evidence

Actual Windows source revision 5102065b2e0c8cb855f8af0d53c6d33f620b97f3; execution metadata is outside simulation hashes. Manifest series 1284, incidents 3828, section maps 1284, version replay-manifest-009-v1. Evaluation digest 699e05319d60c5658f319e06ceed982addb3acb74619b6a87afa45e870d49ebe; metric digest f519ce37f2113ec7aeef0ac2cb1c3889e32dbace821a7f95bc9542688afd625b. Canonical 009 reuses007 sorted-key float-free encoding with a new prefix, preserving historical serialization.

Linux **unverified**: `wsl --list --quiet` reports WSL not installed; `Get-Command docker -ErrorAction SilentlyContinue` finds no executable. No authorized Linux environment was available, none was installed, and no push/CI trigger occurred. `.github/workflows/experiment009-replay.yml` preserves007/008 workflows, pins official action revisions/Rust/locked dependencies, runs Windows/Ubuntu full gates, regenerates independent manifests and compares actual OS/revision/toolchain/lock plus canonical fields. It has not executed. Close this gate by actually running both platforms on one common committed revision and comparing their fresh manifests; the frozen Windows provenance belongs to its earlier source revision. Do not pair it with a different-HEAD Linux execution or label a synthetic/reference test portability evidence.

## Artifact integrity

| Artifact | Bytes | SHA-256 |
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
| experiment-009.json | 125151900 | `157019ee5c078b609f34dee295ce4e6415e9411367f13ea77c29aa48183b1872` |
| experiment-009-windows.json | 679786 | `2c38de80a86ba76db54d0b0c918e00f4710730a7e7fc81cc08b3c2b4045d4169` |

Git attributes retain binary/no-newline-translation frozen JSON. Compact009 evidence125151900 bytes plus 679786-byte Windows manifest; full detailed reports regenerate via scenario/run commands. Report/local-memory sizes and exposure/possible-outcome counts are storage/processing proxies, not wall-clock/heap benchmarks.

## Git record and reproduction

Branch codex/experiment-009.621e668 preregistration; 6feb695 implementation/design; 6f4ee9a metadata correction/invariant tests/CI; 5102065 final fixture assertions; ec98fa2 frozen data/Windows/new regression. The final findings/navigation commit hash is in the completion response and git log, avoiding self-reference. Reviewed scoped staging and diff checks precede every commit. No failed Git operation, historical amend/reset, remote mutation or push; 010 not started.

Each primary row regenerates with `cargo run --locked --release --bin experiment009 -- scenario diagnostic Family seed Policy new-config.json` or `scenario generated selected|injected seed Policy new-config.json`, then `run new-config.json new-full-report.json`. `controls [new-output]` and `counterfactuals [new-output]` regenerate their compact audits. Selected/forced/injected control records include complete configs. To regenerate an observer alternative, generate its diagnostic config with NeverInvestigate, set incident0.forced to `[[5,{"Inspect":source}]]` (or `[[5,"Decline"]]`); remaining policies stay Never. To regenerate a counterfactual ordinary selector, set policy[5] to its recorded policy and later incident.forced to `[[5,"Decline"]]`, leaving the first choice voluntary. Alternative keys are family/seed/source and selector keys family/seed/policy. Run the edited config to obtain the entire observer report and check its canonical full_series hash via the library. Frozen full hashes include local memory and communication consequences, not only final counts.
