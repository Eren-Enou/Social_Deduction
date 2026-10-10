# Experiment 003 engineering validation

This summary separates automated engineering checks from the primary scientific comparisons. It is not an independent third-party review.

Final gates passed on 2026-10-09:

- `cargo fmt --check`.
- `cargo clippy --all-targets -- -D warnings`.
- `cargo test`: 26 integration tests passed (6 unchanged 001, 9 unchanged 002, 11 new 003); no failures. Unit/doc-test harnesses contain no additional tests.
- Primary 003 evaluation: 272/272 byte-identical serialized replays; 176 counterfactual pairs and 16 relabel checks preserved.
- Full frozen 001 CLI output and 128 original reports matched; full frozen 002 evaluation matched. No historical artifacts were regenerated or overwritten.
- The new full 003 evaluation matches its frozen artifact. CLI example output matches library execution, and output creation refuses an existing file without changing it.

New tests exercise typed local inputs, perturbation of another individual's private knowledge without observable disclosure, exact hidden-intention invariance, first-source counting, repeated/retracted claims, uncertainty, competing support, own-message suppression, anchored knowledge, ignored/retained testimony, current versus original voting, explicit abstention, selected sincere mistake and selected deliberate contradiction, injected testimony provenance, actual incorrect verdicts, illegal targets/phases/references, config versions, permutation validation, chronological delivery references and immutable prior snapshots, fixed termination and deterministic replay. Engineering-only single-speaker fixtures added after the primary evaluation verify selected communication influence without injections; they do not alter the preregistered fixture family or listener algorithm.

Early checks found unused imports and a Clippy style issue; these were fixed. One trace test incorrectly required a candidate change in every fixture. Conflict legitimately changed scores while retaining an unknown candidate; the test was corrected to check a belief-state change and phase order. No listener weights or behavior were changed to make tests or primary findings favorable. These were engineering failures before the final successful gate.

Frozen artifact SHA-256 values:

| Artifact | SHA-256 |
|---|---|
| experiment-001.json | `5D3B9ED1B7AE71029BDFA2251FC702E1455C38DB0E22550EC21282A2821E7356` |
| experiment-002.json | `08FEC1B1685F8E9D5C064374BDEC25C4AE6F67DA08AE6A88C013A9E16FBB1D32` |
| experiment-003.json | `758C8AFB75FD8B76450EE2B9470BDAF3D90A7849C4E05166002CC2674D6330BD` |

Limitations: local interfaces prevent accidental engine/audit access but do not sandbox malicious externally supplied Rust policies. Custom runs are labeled unversioned; callers must retain their implementations and provenance. Cross-platform reproducibility was not executed. Legal reference tests and chronological audits cover the built-in policies and declared fixtures, not arbitrary hostile assessment metadata.
