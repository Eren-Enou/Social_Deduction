# Experiment 001 findings

Executed 2026-10-09 with Rust/Cargo 1.99.0. Rules incident-001-v1, baseline-v1, SplitMix64-v1. Frozen full reports: `evidence/experiment-001.json`; dependency versions: Cargo.lock. Configuration and complete decision snapshots are included in every report.

| Fixture | Runs | Claims | Verdicts | Correct verdicts |
|---|---:|---:|---:|---:|
| Silent | 32 | 0 | 0 | 0 |
| Truthful | 32 | 96 | 0 | 0 |
| Random | 32 | 182 | 2 | 0 |
| Mixed | 32 | 123 | 0 | 0 |

All 128 byte-level serialized replays matched. All 2,304 baseline decisions were legal; zero rejected decisions. The silent negative control and incomplete-information truthful control matched their specified predictions. Incorrect random verdicts and unresolved judgments are retained. Truthful means faithful to local belief: one of each truthful run's three claims was factually false because its observation intervention was mistaken. This is not evidence of lying.

Validation: formatting check passed; Clippy all targets with warnings denied passed; six integration tests passed. Tests exercise bounded termination, phase order, local inputs, absence of private fields in dialogue, immutable claim snapshots, deterministic replay, response references, invalid targets/phase/budget, tie/abstention/majority, resolver correctness, and rejected-action effects. Deliberately illegal custom policies terminate without fabricated valid actions. A forced-ballot test establishes successful verdict resolution independently of baseline outcomes.

Demonstrated: a finite reproducible environment, explicit observation interventions, separate observer and individual records, optional communication/voting, and auditable resolver effects. Baseline silence and truth-reporting are coded rules, not emergent findings. Discussion can affect the information available to later decisions, but these baselines do not infer belief from claims.

Failed outcomes: neither random verdict was correct; most runs reached no verdict. No implementation validation failed in the final gate. Untested: intentional lying, strategic concealment, incentives, belief revision, trust, learning, bounded memory, generality, malicious external policy side channels, and cross-platform execution. Custom policies are labeled unversioned and cannot support reproducibility claims unless callers preserve their implementation separately. No Experiment 002 work performed.
