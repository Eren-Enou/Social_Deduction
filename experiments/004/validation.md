# Experiment 004 engineering validation

Final gates passed on 2026-10-09 with rustc and Cargo 1.99.0 on this Windows workspace:

- `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test`: 40 tests passed, zero failed: 2 new unit tests and 38 integration tests (6 historical 001, 9 historical 002, 11 historical 003, 12 new 004). No additional doc tests.
- Primary evaluation: 416/416 byte-identical replays, 208 matched listener comparisons, 32 equal-visible-input hidden-origin controls, and 32 attribution-only comparisons.
- Full frozen 001, 002 and 003 regressions passed. Their evidence hashes are unchanged. No historical evidence was regenerated or overwritten.
- The new full evaluation matches frozen 004 evidence exactly. CLI run output matches library execution and refuses existing files without changing them.

Before freezing 004 evidence, formatting, warnings-denied Clippy and all 38 integration tests passed. The primary CLI then wrote the new artifact using create-new semantics. Two unit checks and the frozen 004 regression assertion were added afterward; they changed no algorithm, weights, fixtures or primary evidence. The final gates above include these checks. Early Clippy checks found an unused import and collapsible conditional, which were fixed before freezing. No failed behavioral result was removed or tuned away.

New checks cover local typed inputs and private-state isolation, unavailable relay parents, exact proposition copies, recipient permissions, malformed/future/cyclic/mismatched references, missing parents without archive lookup, actual and visible depth bounds, separate assigned acquisitions and common roots, wrong and opaque attribution, own-source echoes, first-speaker and first-group semantics, competing chains, deferred genuine corrections, voluntary silence/disagreement, sincere false relays versus original intentional contradictions, injected versus selected origins, immutable decision-time snapshots, chronological decision/delivery/update effects, ballot execution, legal majority/abstention, config validation, rejection without substituted actions, finite termination, and deterministic JSON.

Actual chains of depth three pass; depth four is rejected. Primary chains reach depth two. Fixed rules allow at most 12 selected discussion messages, 12 separately logged injections and six ballots. Listener-visible references cannot fetch inaccessible payloads or actual roots. Origin verification never follows merely from a claimed attribution.

| Frozen artifact | SHA-256 |
|---|---|
| experiment-001.json | `5D3B9ED1B7AE71029BDFA2251FC702E1455C38DB0E22550EC21282A2821E7356` |
| experiment-002.json | `08FEC1B1685F8E9D5C064374BDEC25C4AE6F67DA08AE6A88C013A9E16FBB1D32` |
| experiment-003.json | `758C8AFB75FD8B76450EE2B9470BDAF3D90A7849C4E05166002CC2674D6330BD` |
| experiment-004.json | `F8DEE33C8282CC1CAAFDE7D6B5C00CCA68E001E96E70EE9B7641C62850A71FAD` |

Limitations: this is an engineering evaluation, not independent review. Custom Rust implementations are caller-supplied and unversioned; interfaces do not sandbox malicious host-side channels. Assessment consistency checks are not a proof of arbitrary custom listener semantics. Cross-platform replay, adversarial performance, RAM use and broader behavioral generality remain untested. The repository has no committed baseline, so Git cannot provide a tracked-file diff; source review and unchanged historical replay contracts provide the available regression evidence. No commit or push was performed.
