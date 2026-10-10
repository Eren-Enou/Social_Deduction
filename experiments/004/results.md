# Experiment 004 findings

Completed on 2026-10-09 using the preregistered mechanisms, weights and seeds. Frozen evidence: `evidence/experiment-004.json`. All 416 games reproduced byte-identical reports; none rejected a selected action. These results describe fixed software heuristics, not learned skepticism or general social intelligence.

## What provenance changed

Each row contains 16 games per listener. C/W/N means correct/wrong/no verdict. Speaker, actual acquisition-chain and perceived contributing-group counts refer to recipient 5 at judgment, per game. Actual chains count separate assigned observation acquisitions or constructions, not statistical independence of real witnesses. Perceived groups are unverified associations available through messages.

| Fixture | Received speakers | Actual chains | Source-count groups | Provenance groups | Source-count C/W/N | Provenance C/W/N |
|---|---:|---:|---:|---:|---|---|
| Independent | 3 | 3 | 3 | 3 | 16/0/0 | 16/0/0 |
| KnownShared | 3 | 1 | 3 | 1 | 16/0/0 | 0/0/16 |
| HiddenShared | 3 | 1 | 3 | 3 | 16/0/0 | 16/0/0 |
| UnspecifiedShared | 3 | 1 | 3 | 3 | 16/0/0 | 16/0/0 |
| OpaqueShared | 2 | 1 | 2 | 1 | 0/0/16 | 0/0/16 |
| Chain | 3 | 1 | 3 | 1 | 16/0/0 | 0/0/16 |
| WrongAttribution | 3 | 1 | 3 | 2 | 16/0/0 | 0/0/16 |
| CollapsedIndependent | 3 | 3 | 3 | 1 | 16/0/0 | 0/0/16 |
| MistakenOrigin | 3 | 1 | 3 | 1 | 0/16/0 | 0/0/16 |
| DeceptiveOrigin | 3 | 1 | 3 | 1 | 0/16/0 | 16/0/0 |
| NoMessage | 0 | 0 | 0 | 0 | 0/0/16 | 0/0/16 |
| Disagreement | 2 | 1 | 2 | 1 | 0/0/16 | 0/16/0 |
| InjectedShared | 3 | 1 | 3 | 1 | 16/0/0 | 0/0/16 |

All 208 within-fixture listener pairs had identical delivered messages and recipient routes, with only listener selection changed in their configurations. KnownShared, Chain, CollapsedIndependent, MistakenOrigin, DeceptiveOrigin and InjectedShared avoided two recipient-5 contributions per game; OpaqueShared, WrongAttribution and Disagreement avoided one. The other fixtures avoided none.

Eight fixtures changed recipient-5 ballots and final verdicts in every seed: KnownShared, Chain, WrongAttribution, CollapsedIndependent, MistakenOrigin, DeceptiveOrigin, Disagreement and InjectedShared. OpaqueShared changed ballots but still had no verdict. All changed pairs differed in the three final recipients' ballots; no matched pair changed selected communication. Support interpretation influenced executed judgment, but this evaluation did not establish improved communication strategy.

KnownShared and Chain both collapsed three speakers to one declared group. Their competing private observation retained weight 2 against that group's weight 1. The chain's extra path did not add support. Independent retained three contributions, overcoming the competing private observation. These differences are consequences of explicit grouping and weights, not discovered intelligence.

## Information boundaries and attribution failures

All 32 Independent versus HiddenShared controls had exactly identical recipient-5 listener inputs, assessments and votes despite three actual acquisitions versus one. HiddenShared and UnspecifiedShared stayed indistinguishable from independent reports to this heuristic. The audit did not reveal the hidden dependency to a listener.

All 32 attribution-only comparisons retained initial states, propositions and actual roots. Removing KnownShared's references or incorrectly naming source 0 in CollapsedIndependent changed perceived groups, recipient-5 ballots and verdicts in every comparison. Incorrect naming can therefore erase useful independent evidence. WrongAttribution appeared to have two groups despite one actual acquisition; the listener never corrected that association from the audit.

OpaqueShared's missing parent payload was never delivered to final recipients. Its common reference ID produced one opaque group under the preregistered assumption, with MissingReference recorded. Visible references verify earlier matching messages, not their actual origin. Every source assessment retained `origin_verified=false`, including firsthand and named-source claims.

## Voluntary transmission and misinformation

NoMessage selected no messages: relay policies lacked an assertion and remained silent. Disagreement's relay 1 received the assertion but refused to forward it because its current candidate disagreed. Only one relay was selected per game. InjectedShared selected no communication; its three messages, including two copies, were explicitly controller injections. Similar grouping outcomes do not turn those interventions into agent initiative.

For each listener mechanism, MistakenOrigin produced 16 original sincere mistakes and 32 faithful false relays across 16 games, with no original intentional contradictions. DeceptiveOrigin produced 16 original intentional contradictions and 32 faithful false relays. False content did not make every forwarder an intentional liar. HiddenShared's chosen firsthand misattribution was separately recorded as misleading relay strategy. WrongAttribution's unverified naming mismatch was retained without assuming deliberate intent from inaccuracy alone.

## Negative findings

Provenance grouping prevented correct verdicts in KnownShared, Chain, WrongAttribution, CollapsedIndependent and InjectedShared. It helped in DeceptiveOrigin and prevented wrong verdicts in MistakenOrigin, but those successes do not establish that it is generally better.

Disagreement became worse: source-count produced no verdict, whereas provenance produced 16 wrong verdicts. Three recipients retained their mistaken private Q and the refusing relay also voted Q, producing four wrong ballots. The preserved negative outcome exposes the dependence on private anchors and the majority threshold.

The first-assertion rule still cannot incorporate genuine corrections. Engineering unit checks show that copying one of two conflicting chains does not break a provenance tie, and that even a malformed first assertion blocks that speaker's later genuine correction. Neither behavior was silently replaced after evaluation.

## Communication and storage measurements

The following counts are per game, using the provenance listener. Delivered copies are also final retained private message copies; the finite game has no evictions. Bytes measure serialized values, not actual heap allocation, runtime cost, or the entire duplicated audit snapshot storage.

| Fixture | Delivered/retained copies | Attribution value bytes | Audit lineage bytes | Delivered Message bytes |
|---|---:|---:|---:|---:|
| Independent | 18 | 33 | 243 | 1224 |
| KnownShared | 12 | 41 | 233 | 840 |
| OpaqueShared | 9 | 41 | 233 | 636 |
| Chain | 13 | 41 | 233 | 912 |
| NoMessage | 0 | 0 | 0 | 0 |
| InjectedShared | 12 | 41 | 233 | 840 |

KnownShared versus Chain isolates the additional scoped delivery: one extra retained copy and 72 serialized delivered bytes, without extra perceived support. The evaluator also records compact full-report bytes; the frozen pretty-printed evaluation occupies 46,286,603 bytes. Full decision snapshots deliberately duplicate inputs for traceability. No performance or RAM benchmark was run.

## Limits and next questions

The study uses six individuals, two rounds, fixed order, seeds 0–15, exact proposition copies, weights 1/2/6, and unverified attribution grouping. The seeds vary incident identity; they are not independent samples of human social behavior. No calibrated probabilities, reputation, learned credibility, strategic planning, long-term memory, or statistical witness-independence inference were tested.

Inference lineage records the declared basis and contributing current-candidate evidence from recorded assessments; it is not a complete causal explanation of arbitrary custom Rust code. Typed inputs prevent accidental audit access, but callers can create external side channels in custom policies. Cross-platform replay was not executed. Corrections, broader order/source structures, alternative weights and richer voluntary strategies require separately preregistered evaluations. No subsequent experiment has begun.
