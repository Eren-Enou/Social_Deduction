# SocialDeduction

> **Status:** Experiments 001 through 007 implemented and evaluated. Historical baselines are preserved. Experiment 007 tests fallible verification, local quarantine, held-out forecasts and decision reliability. Windows replay is validated; Windows/Linux agreement remains unverified.
>
> **Primary purpose:** Build a controlled, game-like social-deduction environment to study the conditions under which artificial individuals communicate truthfully, make mistakes, withhold information, or intentionally deceive one another.

## Mission

Explore **deception as an optional behavior that can arise from an individual's private information, incentives, memories, beliefs, and expectations of other individuals**. The objective is not to script convincing dialogue or reproduce a particular commercial social-deduction game.

The environment should make it possible to ask *why* an agent made a claim, *what it believed* when speaking, *what its listener could know*, and *what changed as a result*.

A favorable game outcome does not by itself demonstrate sophisticated cognition. A false assertion does not by itself establish a lie.

## Relationship to other projects

SocialDeduction is an independent experiment, conceptually informed by the **World of Individuals** research project in [`Eren-Enou/Eldoria`](https://github.com/Eren-Enou/Eldoria). It inherits the principles of identifiable causality, information locality, individual perspective, persistent consequences, deterministic replay, and cautious experimental claims.

It does **not** inherit Eldoria fiction or canon; its experiments, code, results, and scientific conclusions must remain independent. See [the handoff](docs/ELDORIA_REFERENCE.md).

## Current status and authority

- **Decided objective:** A controlled social-deduction research environment centered on testing lying and related behavior.
- **Implemented starting scenario:** *The Incident*, with partial observations, two discussion rounds, and optional ballots requiring four matching votes.
- **Implementation:** Headless Rust without Bevy ECS; silent, truthful-to-belief, and seeded-random baselines.
- **Experiment 002:** Truthful, silent, explicit deceptive, and incentive-sensitive policies; private self-knowledge and intent-aware classification.
- **Experiment 003:** Replaceable evidence listeners and judgment policies, per-message causal assessments, testimony interventions, and counterfactual influence measurements.
- **Experiment 004:** Scoped delivery, voluntary exact-copy relays, claimed attribution, bounded audit lineage and provenance-aware listener comparisons. Source claims remain unverified; grouping can worsen judgments.
- **Experiment 005:** Later local observations, voluntary cost-sensitive corrections, own-relay revisions and contested branch/group handling. Revision awareness can restore, leave unchanged or worsen judgments; corrections are not verified truth.
- **Experiment 006:** Bounded persistent local experience and credibility, explicitly delivered outcome verification and mixed later judgment effects.
- **Experiment 007:** Fallible verification, retained-conflict quarantine, held-out frequency/ordinal forecasts, coverage-aware decision comparisons and canonical replay manifests.
- **Deferred:** Advanced cognition, global reputation, trained social learning, coalitions and a public UI.
- **Evaluated:** 352 series/1056 incidents in 006 and 1792 series/5200 incidents in 007, plus 128 historical 001 runs, 256 002 runs, 272 003 runs, 416 004 runs and 576 005 runs with matching serialized replays. See [001 findings](docs/FINDINGS_001.md), [002 results](experiments/002/results.md), [003 findings](experiments/003/results.md), [004 findings](experiments/004/results.md), [005 findings](experiments/005/results.md), [006 findings](experiments/006/results.md) and [007 findings](experiments/007/results.md). All behaviors reflect specified policies and heuristics, not evidence of emergent intelligence.

Documents contain explicit **Decided / Proposed / Open / Deferred** statuses. A design proposal must not be reported as an implemented or verified capability.

## Read first

1. [`AGENTS.md`](AGENTS.md) — instructions for coding assistants and contributors.
2. [`docs/SIMULATION_CONSTITUTION.md`](docs/SIMULATION_CONSTITUTION.md) — invariants the research must preserve.
3. [`docs/RESEARCH_OBJECTIVES.md`](docs/RESEARCH_OBJECTIVES.md) — questions, definitions, and experimental claims.
4. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — state and information boundaries.
5. [`docs/EXPERIMENT_001.md`](docs/EXPERIMENT_001.md) — initial game-engine and observation experiment.
6. [`docs/EVALUATION_PROTOCOL.md`](docs/EVALUATION_PROTOCOL.md) — pre-registration, evidence, controls, and replay.
7. [`docs/ROADMAP.md`](docs/ROADMAP.md) — provisional sequence of research stages.
8. [`docs/DECISIONS.md`](docs/DECISIONS.md) — decision statuses and unresolved choices.
9. [`docs/ELDORIA_REFERENCE.md`](docs/ELDORIA_REFERENCE.md) — ideas worth adapting, and what not to import.

## Suggested development workflow

1. Resolve only the decisions needed for the next experiment.
2. Write a testable hypothesis and control conditions before implementing the behavior.
3. Implement the smallest engine/rule surface necessary.
4. Validate legality, information boundaries, termination, and deterministic replay.
5. Run controlled studies, including negative controls and failure cases.
6. Record the actual result before considering adoption or generalization.

**Do not begin with a large language model, a complex role catalog, a visual game client, or a shared cognition library.** First establish the experimental environment and the standards by which we can trust its observations.

## Run Experiment 001

```powershell
cargo run -- 7 truthful
cargo run -- 7 random report.json
cargo run -- evaluate evidence/experiment-001-repeat.json
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Single runs accept a seed and `silent`, `truthful`, or `random`. Output goes to stdout unless a file is supplied. Evaluation runs all four documented fixtures over seeds 0–31 and verifies serialized replay. File output refuses overwrites. Reports contain observer secrets; never supply them as individual policy input. Existing frozen evidence is preserved in `evidence/experiment-001.json`.

The original checkout contained only README.md and AGENTS.md. Referenced design documents were absent; the foundation documents were established from the supplied instructions before source implementation, and chosen mechanics are explicitly recorded in DECISIONS.md.

## Run Experiment 002

```powershell
cargo run --bin experiment002 -- run experiments/002/accuracy.json
cargo run --bin experiment002 -- evaluate evidence/experiment-002-repeat.json
```

The example config specifies a seed, six policy selections, six private incentive vectors, and whether the participation briefing is delivered. Change incentive weights in a copy to make new comparisons with the same rules. Built-in evaluation runs all eight preregistered fixtures and 96 matched comparisons; files refuse overwrites. [Specification](experiments/002/specification.md) describes scoring, classification, controls, and limitations. [Frozen 002 evidence](evidence/experiment-002.json) preserves every decision. Full observer reports contain private knowledge and intent; only public messages enter listener views.

## Run Experiment 003

```powershell
cargo run --bin experiment003 -- run experiments/003/truth.json
cargo run --bin experiment003 -- evaluate evidence/experiment-003-repeat.json
```

The 003 config separately selects communication, listener and judgment policies, plus explicit ID layout, speaking order, legitimate observation assignments and optional scripted interventions. Output refuses existing files. [Specification](experiments/003/specification.md), [findings](experiments/003/results.md), [validation](experiments/003/validation.md) and [frozen evidence](evidence/experiment-003.json) distinguish selected communication from externally supplied testimony and changed beliefs from executed votes. Listeners count visible source assertions without seeing hidden intentions or observer truth; ties permit abstention. Scores are uncalibrated support counts.

## Run Experiment 004

```powershell
cargo run --bin experiment004 -- run experiments/004/known-shared.json
cargo run --bin experiment004 -- evaluate evidence/experiment-004-repeat.json
```

Outputs refuse existing files. The [specification](experiments/004/specification.md) defines source-group assumptions, relay legality and scoped routes. [Findings](experiments/004/results.md), [validation](experiments/004/validation.md) and [frozen evidence](evidence/experiment-004.json) preserve independent/shared/hidden/incorrect origins, mistaken and deceptive propagation, silent and injected controls, negative judgments and storage measurements. Only received messages enter listener views; actual roots, speaker intent and objective truth remain observer-only. Historical 004 validation passed forty tests; its replay contracts remain in the current suite.

## Run Experiment 005

```powershell
cargo run --bin experiment005 -- run experiments/005/sincere.json
cargo run --bin experiment005 -- evaluate evidence/experiment-005-repeat.json
```

Outputs refuse existing files. The [specification](experiments/005/specification.md) defines explicit own-message revisions, branch disputes, local observation updates, selective costs and relay verification. [Findings](experiments/005/results.md), [validation](experiments/005/validation.md) and [frozen evidence](evidence/experiment-005.json) distinguish private change, selected disclosure and actual recipient recovery. Original messages remain immutable; missing references stay unavailable. The historical005 gate passed 59 tests; its frozen contract remains in the current suite.

## Run Experiment 006

```powershell
cargo run --bin experiment006 -- run experiments/006/accurate.json
cargo run --bin experiment006 -- example new-series-config.json
cargo run --bin experiment006 -- evaluate evidence/experiment-006-repeat.json
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Outputs refuse overwrites. The [specification](experiments/006/specification.md) preregisters stable six-person identities across at most eight independent incidents, bounded local experience, authenticated feedback delivery and transparent credibility weights extending unchanged005 revision/provenance rules. [Findings](experiments/006/results.md), [validation](experiments/006/validation.md) and [frozen evidence](evidence/experiment-006.json) preserve 352 series/1056 incidents, local observer disagreement, beneficial judgment and trusted falsehood. Seventy-five tests include all six frozen byte-regression contracts. No dependencies or ECS were added. Full observer reports contain secrets; only own received messages and explicitly delivered verification enter local memory. This is the historical 006 scope; Experiment 007 separately relaxes its accurate-verifier assumption.


## Run Experiment 007

The toolchain is pinned to Rust 1.99.0 and dependencies are locked. Outputs use create-new and refuse existing paths.

```powershell
cargo run --locked --release --bin experiment007 -- run experiments/007/authenticated-false.json
cargo run --locked --release --bin experiment007 -- scenario generated Clean 1000 ConflictAware Credibility new-config.json
cargo run --locked --release --bin experiment007 -- run new-config.json new-full-report.json
cargo run --locked --release --bin experiment007 -- evaluate new-evaluation.json
cargo run --locked --release --bin experiment007 -- replay new-windows-manifest.json
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --release
```

Use `scenario diagnostic RewardFalse 0 ConflictAware Credibility new-config.json` for diagnostic regeneration. Scenario names, seed, acceptance and listener in each frozen row identify the full report to regenerate. [Specification](experiments/007/specification.md), [findings](experiments/007/results.md), [validation](experiments/007/validation.md) and [frozen evidence](evidence/experiment-007.json) distinguish authentication, factual accuracy, local detectability, predictive quality and decision utility. Forecasts never receive future assertions or observer truth and never affect ballots. Compact evidence retains causal traces and six canonical digests per complete series; full reports remain observer-only.

Actual Linux replay is unavailable locally. The [CI workflow](.github/workflows/experiment007-replay.yml) independently regenerates Windows/Linux manifests and compares them; it has not been executed. After real executions on the same committed revision, run `cargo run --locked --release --bin experiment007 -- compare windows-manifest.json linux-manifest.json`. Synthetic comparator tests do not constitute OS validation. Experiment 008 is described separately below.


## Run Experiment 008

```powershell
cargo run --locked --release --bin experiment008 -- scenario diagnostic Accurate 0 ConflictDriven new-008-config.json
cargo run --locked --release --bin experiment008 -- run new-008-config.json new-008-report.json
cargo run --locked --release --bin experiment008 -- scenario generated selected 2000 CostSensitive new-generated.json
cargo run --locked --release --bin experiment008 -- evaluate new-008-evaluation.json
cargo run --locked --release --bin experiment008 -- replay new-008-windows.json
```

Use `scenario generated injected 2000 NeverInvestigate new-injected.json` for the matched controller control. File outputs refuse overwrites. [Specification](experiments/008/specification.md), [findings](experiments/008/results.md) and [validation](experiments/008/validation.md) distinguish optional investigation, costly missing/false results, private information and selected disclosure. Findings add support under visible provenance assumptions; they do not replace original observations automatically. Effort persists across incidents alongside the unchanged bounded local credibility mechanism. Full JSON reports contain observer secrets; typed policy views never receive the report or latent artifact content. The separate [008 CI workflow](.github/workflows/experiment008-replay.yml) preserves the pinned toolchain and compares independently regenerated Windows/Linux manifests. No Experiment 009 is included.

## What this repository must not claim

The simulation does not prove consciousness, genuine free will, human-equivalent lying, or broad social intelligence. It can establish narrower causal facts about behaviors of specified artificial policies under reproducible conditions.

## Source and licensing note

Commercial social-deduction games are design inspiration, not a requirement to reproduce protected artwork, character text, or a branded digital game. The first prototype should use original rules and terminology. Choose a repository license deliberately; this starter does not select one.
