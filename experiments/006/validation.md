# Experiment 006 engineering validation

Validated 2026-10-09 on Windows, Rust/Cargo 1.99.0, unchanged Cargo.lock and dependencies. This is engineering validation, not independent review.

Before freezing: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` passed. The full suite contained 75 tests: four historical unit tests and 71 integration tests, including 16 new006 integration tests. The primary test executed 352 complete serialized series replays, 176 listener pairs and 96 broader controls. All 001–005 frozen byte-regression contracts passed. The CLI then created `evidence/experiment-006.json` once. A full-byte006 frozen assertion was added afterward and passed against a fresh primary evaluation; no implementation, weights, fixtures or seeds changed after freeze.

Final gates: formatting check; all-targets Clippy with warnings denied; full `cargo test`; six frozen byte-regression contracts; historical SHA-256 integrity checks; `git diff --check`; staged review and clean final working tree. Seventy-five tests passed with no failures or ignored tests. There are no doctests. Example generation, JSON-config series execution and refusal to overwrite existing output are also tested.

Coverage: stable identities; fresh current observations/beliefs/messages/votes/policies; episode-namespaced received memories; exact callback credit inputs matching corresponding engine updates; local delivery eligibility; no historical report/actor/intent in cognition views; undelivered and unverified feedback; wrong collective verdicts; received authenticated outcomes; separate original/final accuracy and correction counts; delayed verification; repeated verification idempotence and conflicting/unauthorized issuer rejection; local observer disagreement; prior errors suppressing correct testimony; previously trusted false testimony; reversible score paths; visible relay deduplication and unknown/misattributed responsibility; unchanged neutral005 support and interpretation; revision/retraction/dispute standing; private anchor/own-source echo behavior; legal voluntary silence/abstention; eight-incident/FIFO/config route bounds; declared retained consequences and eviction; equal local experience despite hidden actor/private-state/intent changes; selected/injected experience equivalence; identity/order controls; updater Retain control; complete primary replay and chronological feedback records. Historical tests continue to cover action ownership, phase/budget legality, four-vote majority and bounded incident termination.

The prefreeze Clippy check found one needless indexed range loop; it was replaced with an enumerated iterator. No invariant assertion failed, and no empirical negative/null outcome caused policy tuning. Constitution introductory coverage now includes006; its ten numbered principles are unchanged. `.gitattributes` disables newline translation and textual diff for frozen JSON so Windows checkout cannot silently change comparison bytes; old artifact contents were not rewritten.

| Frozen artifact | SHA-256 |
|---|---|
| experiment-001.json | `5D3B9ED1B7AE71029BDFA2251FC702E1455C38DB0E22550EC21282A2821E7356` |
| experiment-002.json | `08FEC1B1685F8E9D5C064374BDEC25C4AE6F67DA08AE6A88C013A9E16FBB1D32` |
| experiment-003.json | `758C8AFB75FD8B76450EE2B9470BDAF3D90A7849C4E05166002CC2674D6330BD` |
| experiment-004.json | `F8DEE33C8282CC1CAAFDE7D6B5C00CCA68E001E96E70EE9B7641C62850A71FAD` |
| experiment-005.json | `3B30BDAEC3496D5F4018F42A0D110CC7D67415EE3E0BF73B990EBC7D158E47E6` |
| experiment-006.json | `34E27342C14C1B09FE44CD268E9D9F5DC38D7335797BBDBDB32DE4098FC5A360` |

Historical 001–005 engines/test sources/artifacts have no diff from initial commit `6df52b4`. Only the new module export is appended to src/lib.rs. Historical experiment documents/evidence remain preserved; current architecture/decisions/roadmap record006 separately. Research claims and limits appear in results.md. Typed interfaces and output validation do not sandbox malicious host-side channels or prove arbitrary custom policy correctness. Custom updaters carry caller-supplied-unversioned implementation provenance.

Git branch: `codex/experiment-006`. Preregistration `0124577`; implementation/config/invariants/architecture `a63fa3b`; frozen data/byte regression `33afd2a`; findings/validation/navigation in the subsequent documentation commit. Each commit used scoped staging, diff check and staged review. Local commits were retained; no push, amend, history rewrite or remote mutation occurred. The final documentation hash is available from `git log` and the development completion report, avoiding a self-referential commit hash here.
