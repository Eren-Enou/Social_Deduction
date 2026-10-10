# Experiment 005 engineering validation

Validation performed 2026-10-09 on Windows with Rust/Cargo 1.99.0 and unchanged locked dependencies. This is an engineering report, not independent review.

Final gate commands:

- `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test`: 59 tests (4 unit, 55 integration), including 17 new integration and 2 new unit tests for 005; all passed, zero failures.
- Primary evaluation: 576/576 serialized replays; 288 listener pairs and 384 mechanism comparisons; zero rejected selected actions.
- Full byte-level frozen regression checks for all five experiments. Historical 001–004 artifacts were neither regenerated nor overwritten and their hashes remain unchanged.
- CLI example matches library execution and fixture config; output refuses existing files without changing them.

Before freezing, formatting, warnings-denied Clippy and all 57 then-present tests passed, including historical frozen contracts and in-memory primary replay. The CLI created evidence/experiment-005.json once using create-new. The frozen 005 regression assertion and two engineering-only tests (adverse misleading correction and explicit majority ballots) were added afterward. No policy, listener, fixture, weights or primary evidence changed afterward. Final checks include these additions.

Coverage includes own-target ownership, previous sent-message possession, unavailable original targets, exact received forwarded replacements, foreign and forward/cyclic references, scoped recipients, malformed original provenance, unverified claims, branch competition and sequential links, retraction, first ordinary assertion behavior, no extra group contribution for revisions, stale relay conflict and unavailable forwarded source messages, changed private knowledge without public disclosure, cost-selected withholding/tie silence, numeric limits, sincere repairs versus still-wrong revisions and intent-supported contradictions, faithful false forwarding, immutable observations/messages/decisions, chronological delivery and evidence-parent references, controller observation/injection distinction, actual ballot execution, abstention/four-vote threshold, invalid actions without substitution, config/resource limits, fixed 18-decision termination and deterministic replay.

Evidence chains of depth three pass; depth four is rejected. Primary evidence depth reaches one. Individual communication remains 12 scheduled opportunities; controller injection and later-observation lists each cap at 12. All historical test source files remain unchanged. Listener inputs and messages expose no actor truth, sender intent/belief, audit roots or correctness flag. Injection-time resolver inputs are preserved separately in the observer report.

Early checks found a Rust mutable-borrow error in copying a configured listener and Clippy style issues in tests; these were fixed. Two early assertions incorrectly expected RelayIntermediate to become uncertain. Under the preregistered rule its forwarded source correction was unavailable, so the listener ignored the revision and retained wrong support. Tests were corrected to that negative result; the algorithm was not changed to improve it. No primary behavior was tuned away.

| Frozen artifact | SHA-256 |
|---|---|
| experiment-001.json | `5D3B9ED1B7AE71029BDFA2251FC702E1455C38DB0E22550EC21282A2821E7356` |
| experiment-002.json | `08FEC1B1685F8E9D5C064374BDEC25C4AE6F67DA08AE6A88C013A9E16FBB1D32` |
| experiment-003.json | `758C8AFB75FD8B76450EE2B9470BDAF3D90A7849C4E05166002CC2674D6330BD` |
| experiment-004.json | `F8DEE33C8282CC1CAAFDE7D6B5C00CCA68E001E96E70EE9B7641C62850A71FAD` |
| experiment-005.json | `3B30BDAEC3496D5F4018F42A0D110CC7D67415EE3E0BF73B990EBC7D158E47E6` |

Limits: typed interfaces prevent accidental audit access but do not sandbox malicious Rust host-side side channels. Custom implementations are labeled caller-supplied-unversioned; consistency validation is not a proof of arbitrary listener semantics. Cross-platform replay, runtime/heap benchmarks and broad generality remain untested. The checkout has no committed tracked baseline; source review and frozen replay contracts provide available regression evidence. No commit or push was performed. See results.md for scientific failures, uncertainty and limited recovery claims.
