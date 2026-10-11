# Experiment 012 validation

Preregistration commit `596e8a6` preceded implementation. The simulation, test fixtures, rules and pre-primary clarifications were committed as `9cdbf593dc7361108939e0e6954a024b6b6a0e9d` before any primary world ran. Accepted baseline `b7bc55521b2837a74921713802265115bebd8324` completed Experiment 011. All work is on `codex/experiment-012`; no push, accepted-history rewrite or Experiment 013.

## Engineering gates

The complete locked all-feature release suite on committed source passed **266 tests**, zero failed or ignored. After exclusive create-once freeze, the new complete frozen 012 regression passed separately in 283.82 seconds: **267 distinct validated tests**. No combined 267-test execution is claimed. Formatting and all-target/all-feature warnings-denied Clippy passed again with the new regression included.

The unchanged historical baseline completed all 225 tests with zero failures or ignored tests. The latest pre-primary focused gate passed all 40 Experiment 012 mechanism tests and its create-once CLI test. Formatting and locked all-target/all-feature Clippy with warnings denied passed before primary execution. The combined current-source gate ran separately; primary execution overlapped it, but freezing waited for its success, the independent Windows replay and integrity checks. No combined gate success was assumed while it was running.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --release
cargo test --locked --all-features --release --test experiment012_frozen
```

The isolated build directory was `CARGO_TARGET_DIR=target/experiment012-validation`; build paths do not enter simulation state. Rust `1.99.0 (b940084d7 2026-09-28)`, Cargo `1.99.0 (5f94df478 2026-08-27)`, actual Windows x86_64. No dependency, lock or toolchain change. Cargo.lock SHA-256: `360931d88271e853ace662b91ff45c6df0537a94c029f7318b7f230812099c92`.

## Mandatory invariant coverage

The 40 tests in [experiment012.rs](../../tests/experiment012.rs) cover the 29 requirements below. The unchanged historical suite additionally tests original event resolution, legal actions, voting, tie handling, termination, feedback, memory and resources. The CLI test checks exact configuration/report output and refused overwrites. The frozen regression regenerates the complete population rather than checking selected summary values alone.

| Requirement | Executed checks |
| --- | --- |
| 1. Typed requests | `typed_request_and_response_versions_and_bounds` |
| 2. Recipient private memory isolation | `requester_decisions_ignore_hidden_truth_and_other_private_memory`; changed responder history with identical requester history |
| 3. No uninspected environmental evidence | `request_views_are_local_and_catalogs_have_no_sealed_results` |
| 4. Request capacity | `legal_queries_and_answers_consume_distinct_outbound_actions` |
| 5. Received request reference | Foreign, future, unreceived and duplicate references reject |
| 6. Addressed response only | Scoped delivery and foreign-recipient validation |
| 7. Response capacity | Legal action accounting and rejected submissions consume available capacity |
| 8. Outbound maximum | `total_outbound_capacity_conserves_across_all_windows`; no free multiple answers |
| 9. Displaced testimony | `response_can_exhaust_capacity_and_block_final_testimony`; selective silence executes retained final opportunity |
| 10. Silence/refusal | No-experience silence, selective silence, legal refusal and refusal excluded from social evidence |
| 11. No unsent answer | `absent_response_creates_no_wire_or_social_evidence` |
| 12. Insufficient information | `no_experience_allows_insufficiency_and_silence` |
| 13. Retained sincere information | `sincere_response_counts_equal_retained_local_counts`; basis clarification uses retained earlier report |
| 14. Hidden intent | Wire/input field exclusions and equal interpretation despite changed private intent |
| 15. No query personal samples | `questions_and_answers_add_no_verified_personal_samples` |
| 16. No answer personal samples | Same test; availability supplies no conditional accuracy support |
| 17. No verification token transfer | Asserted verification leaves accepted personal counts unchanged; historical token validation unchanged |
| 18. Known duplicates | `known_duplicate_origins_group_and_hidden_ancestry_is_not_available` |
| 19. Hidden common origins | Same test and private intent/input boundary tests |
| 20. Immutable corrections | `response_corrections_preserve_immutable_old_wires` |
| 21. Late response | `late_answers_cannot_rewrite_completed_investigation` |
| 22. Different recipients | Scoped delivery and explicit wider control with different legitimate histories |
| 23. Equal local histories | Requester hidden-truth/private-history substitution and listener hidden-intent substitution |
| 24. Sources/resources unchanged | `effort_accounting_and_finding_weights_are_unchanged` and historical invariants |
| 25. Investigation influence | `legally_different_answers_change_same_incident_investigation`; all early delivery precedes selection |
| 26. No-query capacity | `ordinary_no_query_baseline_retains_two_speech_opportunities` |
| 27. Selected/injected origins | `selected_and_injected_answers_share_visible_effects_distinct_origin` |
| 28. Complete replay | Full reports, all diagnostic/development rows, eight canonical sections, legal ballots and termination |
| 29. Historical preservation | Complete historical gates, 17 raw integrity checks, 98 unchanged prior implementation/configuration files and retained document prefixes |

Additional negatives cover invalid recipients, unknown families, duplicate requests, inaccessible priors, malformed reply payloads, foreign revisions, inappropriate timing, wrong windows and exhausted capacity without retry. Independent seeded random requests can decline; recipient selection is sensitive to legitimate local uncertainty/cost changes. FIFO evictions are logged, archive-only history is not restored, and questions/refusals do not become method evidence.

## Primary execution and independent audit

The committed evaluator completed 696 series / 5,568 incidents with 696 exact complete-report replay matches. It includes 64 independent matched held-out worlds, 66 paired comparisons and 1,728 prediction groups. Every row executed twice. A separate actual Windows process independently executed the entire evaluator again, also running every row twice, and wrote its execution manifest. Both processes exited successfully.

The independent Python audit reproduced sorted, integer-only canonical JSON and SHA-256; matched the complete evaluation/metric digests and every one of eight section hashes for all 696 rows against the Windows manifest; recomputed all rational Brier groups; and checked matching physical worlds/resources, request/response routing, capacity and effort ledgers. It independently recomputed the recorded before/after forecast effects. Eight separately regenerated complete seed-zero diagnostic reports also matched all eight population section hashes. These independent checks are evidence analysis, not new policy training.

| Artifact | Raw bytes | SHA-256 |
| --- | ---: | --- |
| [Primary evidence](../../evidence/experiment-012.json) | 300005474 | `f22a61dbeca029dd29977228805ab9c932f5e834433c61c8edaca1da0157b4f2` |
| [Actual Windows replay](../../evidence/experiment-012-windows.json) | 466173 | `de9aebfcf050b92e3a518c31c8139cc54ec5da894d8f65b546c6254a26d7e1c2` |

Canonical namespace `canonical-012-v1`; complete evaluation digest `7f59152e77d500b07b038d4551aa4f0fc69bf241a2edbfb72d5c1d614c905e9b`; metric digest `323b83810219245c3aad5d33a0f21befcd74a931454f59dd36380a4010e44a6d`. Frozen files were copied from successful primary outputs using exclusive create semantics, then checked for exact bytes and hashes. Existing evidence was never overwritten. The frozen regression independently repeats the full evaluation and exact serialization, verifies the platform metadata/manifests, and rejects two Windows manifests as proof of Linux agreement.

## Development failures and measurement limitations

Before primary execution, focused development exposed two mistaken assertions: a forced training choice was compared as if voluntary, and the initial accurate fixture failed to create a genuine A-to-B choice change. Assertions were corrected to distinguish the controller intervention; the preregistered conditioned fixture was clarified to create mixed own A experience. Review also found a copied bootstrap seed (11012), corrected to the required 12012 before primary execution; a SelectiveReply refusal that spent rather than conserved capacity, corrected to silence for the strong incident-candidate branch; and a foregone-speech recorder storing an exhausted actual view beside a proposal computed with hypothetical capacity, corrected to retain the exact counterfactual input separately. Regression checks cover these changes.

Temporary test edits included an invalid field reference and a misplaced assertion; these failed development checks and were corrected before the final 40-test gate. An initial Clippy duplicate branch warning was resolved before committed validation. Earlier overlapping full-gate attempts were stopped when pre-primary changes made them stale; only the completed committed-source gate counts. No failed gate was bypassed, and no policy/threshold was retuned on the primary outcomes.

Primary failures are retained: 352 misleading reply proposals fail family recognition; 1,328 Targeted requests lack a same-incident reply; capacity blocks 727 final ordinary proposals; R/S reversal fixtures produce no retractions; targeted default utility is adverse versus the main baselines. Submitted reply-kind counters can include rejected proposals. Reply-stage eviction counters omit query-stage evictions, which are separately recorded and reported. Two inherited encoded-ratio disagreement counters are not treated as validated truth measures; the findings use independently cross-multiplied process-direction counts. The DirectOnly versus ProvenanceAware comparison includes inherited advertised-fallback differences and is not a pure answer-content contrast. Personal Beta forecast scores are estimator scores, not always the actual DirectOnly choice expectation.

## Historical raw integrity

All 17 artifacts retain their original sizes and hashes. Existing source, tests, workflows, toolchain and dependencies were compared with the accepted baseline; only an additive 012 export changes the old lib entry point. Existing document contents remain prefixes; new decisions are appended.

| Historical artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `evidence/experiment-001.json` | 2725107 | `5d3b9ed1b7ae71029bdfa2251fc702e1455c38db0e22550ec21282a2821e7356` |
| `evidence/experiment-002.json` | 11841127 | `08fec1b1685f8e9d5c064374bdec25c4ae6f67da08ae6a88c013a9e16fbb1d32` |
| `evidence/experiment-003.json` | 25296793 | `758c8afb75fd8b76450ee2b9470bdaf3d90a7849c4e05166002cc2674d6330bd` |
| `evidence/experiment-004.json` | 46286603 | `f8dee33c8282cc1caafde7d6b5c00cca68e001e96e70ee9b7641c62850a71fad` |
| `evidence/experiment-005.json` | 72225029 | `3b30bdaec3496d5f4018f42a0d110cc7d67415ee3e0bf73b990ebc7d158e47e6` |
| `evidence/experiment-006.json` | 405837038 | `34e27342c14c1b09fe44cd268e9d9f5dc38d7335797bbdbdb32de4098fc5a360` |
| `evidence/experiment-007-windows.json` | 892966 | `44cb24f848a90554056b4222752dfd4b15caa85e8f152e33eefe46cd1f5e7d3b` |
| `evidence/experiment-007.json` | 117084630 | `1ce7952585ab3197243daf33b8ecc68dc54b071f7a6c508953d830ed61e21864` |
| `evidence/experiment-008-cost-control.json` | 1923910 | `9d7447dd3367d9a9c2e3e66cc8e6c7759642db1aa7a244951f34cf17be7d7876` |
| `evidence/experiment-008-windows.json` | 322181 | `f0aef8d5b460cd2fe5ac68d1fb4429ae64a316c93b39b2c0621edbf9e3d1f381` |
| `evidence/experiment-008.json` | 61244605 | `fa852e921f549dbb87344c7d3329a913985608b19a930cd456f0cc1c667a09cc` |
| `evidence/experiment-009-windows.json` | 679786 | `2c38de80a86ba76db54d0b0c918e00f4710730a7e7fc81cc08b3c2b4045d4169` |
| `evidence/experiment-009.json` | 125151900 | `157019ee5c078b609f34dee295ce4e6415e9411367f13ea77c29aa48183b1872` |
| `evidence/experiment-010-windows.json` | 2270551 | `81d712affd6dd39346e0af475c8dfdbcf7cce4920e62dbf4f856eadb3dbebed7` |
| `evidence/experiment-010.json` | 972371932 | `68d25a4799f79d21aff002da6867132fc1a0d69975542e3a5bc5f7d9df0905b6` |
| `evidence/experiment-011-windows.json` | 555565 | `12e85f955e09c2faad15f846e091e2ace26fc5265f5a2ef3c4025fcadab5e6a9` |
| `evidence/experiment-011.json` | 233454880 | `001bfe314e3c65e7c43f886a75836e9aceff4153021595fe4ac7992ca5ea262f` |

## Cross-platform status and commits

Windows replay succeeded. Actual Linux execution remains **unverified**: `wsl --list --quiet` reports WSL is not installed, and no Docker command is available. Existing pinned Windows/Ubuntu workflows were reviewed; the new [012 workflow](../../.github/workflows/experiment012-replay.yml) uses the same pinned toolchain, unchanged lock, committed configurations and canonical comparisons. It was not triggered, and no Linux metadata was invented. Closing this limitation requires a real Linux x86_64 runner with Rust 1.99.0 or explicit authorization to push/execute that CI workflow.

The preregistration and implementation commit IDs above precede execution. Data commit `44d044e5a76a512050b2594986f4894e8b097d0a` freezes the validated reports and regression. The final documentation commit records findings, this validation and append-only navigation/research status; its actual ID is available in Git and the delivery message. No self-referential document commit hash is invented.
