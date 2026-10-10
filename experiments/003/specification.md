# Experiment 003: Evidence-Based Belief Revision and Social Influence

Status: preregistered before implementation and primary evaluation; now implemented, evaluated and validated. This document preserves the original algorithm, predictions and fixture family. No listener parameters were tuned on the specified evaluation seeds.

## Mechanism and scope

Rules `incident-003-v1`, config `config-003-v1`, listener `source-count-003-v1`, judgment `belief-vote-003-v1`. Six individuals, two discussion rounds, one opportunity per turn, six simultaneous ballots, strict four-vote majority and legal abstention are preserved. Engine 001/002 and frozen evidence are untouched. Reuse 002 structured assertions/uncertainty, intent validation and communication policies through a local-view adapter. Communication uses the belief at its decision time, not its original belief. The programmed one-message budget remains a baseline-policy feature.

Each individual retains original observation, private participation briefing and incentives; current belief has six integer support scores, a unique positive maximum candidate (otherwise unknown), and top-minus-runner-up margin. Scores and margin are uncalibrated evidence counts, not probabilities. Initial observation contributes 2 to its candidate; participation briefing contributes 6 to self, replacing the observation's score but preserving the observation record. Belief revision recomputes from these original private inputs and the complete legitimately received message list, never from previous scores. This prevents repeated processing or relay of one's own speech from reinforcing one's own belief.

Evidence listener: ignore own messages; ignore uncertainty; the first assertion from each other speaker contributes 1. Later assertions from that speaker, whether identical or contradictory, are ignored and recorded as repeats. An uncertain message does not reserve that speaker's assertion. Different speakers are distinct public sources; the listener has no hidden common-source knowledge. Agreement adds support, disagreement adds competing support. Scores tied for first place cause uncertainty/abstention; absence of evidence retains an unknown candidate. A participation briefing has weight 6, greater than all five other speakers combined. A retain listener ignores all testimony and preserves its prior. Both implement replaceable interfaces with no observer data.

Update before each scheduled communication and ballot, using only received messages. Record prior belief, full listener input, per-message assessment/references, result and observer-only correctness. Delivery records show which message was available when. Original observations, old decisions and old claims never change. Current-belief judgment votes a unique supported candidate; original-belief judgment uses the initial private candidate; abstain judgment is a negative control. All are replaceable and can choose abstention. Invalid actions consume opportunities and have no effect. Configurations permit at most 12 separately logged external injections; these do not consume the 12 scheduled individual communication opportunities.

## Hypotheses and predictions

H1: an isolated unknown recipient's belief and executed vote change under one asserted testimony versus matched absence. Both sincere mistake and deliberate contradiction may cause a wrong belief. This tests the implemented heuristic, not sophisticated reasoning.

H2: identical listener-visible inputs produce identical updates and votes despite different hidden speaker beliefs, selected intents, or observer classifications.

H3: conflict, silence, uncertainty, retained private evidence and repeated same-speaker messages produce uncertainty or null effects in declared controls. Repetition must not create independent corroboration. Current-belief and original-belief voting can differ without any rule change.

No success target for natural-game verdicts or correctness. Preserve incorrect, uncertain, and unchanged results.

## Fixtures specified in advance

All fixtures use seeds 0–15 (16 seeds, selected before evaluation). Identity layout maps logical incident IDs and witness slots to visible IDs; actor mapping and mistaken candidate mapping are explicit. Role permutation and speaking order are separate configuration fields validated as permutations.

Natural-policy fixtures (no injected testimony):

1. TruthRetain: all 002 truthful communication, retain listeners, current-belief judgment.
2. Truth: same with evidence listeners.
3. Deceptive: all explicit 002 deceptive controls, evidence listeners.
4. Silence: all silent communication, evidence listeners (judgment remains independent).
5. Blame: all incentive-sensitive communication with blame weights, evidence listeners.
6. OriginalVote: identical to Truth except original-belief judgment.
7. Relabeled: Truth with layout [3,4,5,0,1,2] and speaking order following that layout. Analyze relabel symmetry separately.
8. ReverseOrder: Truth with identity layout and reversed speaking order. Analyze order effects separately.

Participation briefings enabled and original 001 witness assignments maintained in this family. All private incentive assignments and policy selections are logged controller settings.

Scripted testimony fixtures: all communication silent, evidence listeners and current judgment, briefing disabled; source 0 initially observes actor, source 1 observes the next logical candidate, all others observe nothing. Injections occur before discussion, are delivered to IDs 2–5 only, and are explicitly marked interventions in the observer record. Recipients never see intervention origin or intent. These are not voluntary speaker decisions.

- Absent: no injection.
- OneTruth: source 0 asserts actor, ReportBelief.
- OneMistake: source 1 asserts next candidate, ReportBelief.
- OneLie: source 0 asserts next candidate, ContradictBelief.
- Conflict: source 0 asserts actor and source 1 asserts next candidate.
- Repeat: source 0 asserts actor twice.
- Uncertain: unknown source 2 expresses uncertainty.

Hidden-intention fixtures (selected communication, no injections): all observations absent except source 0, briefing disabled, source 0 first in speaking order, others silent. HiddenTruth source 0 observes actor and uses truthful policy; HiddenContradiction source 0 observes previous candidate and uses deceptive control, which asserts actor. Identical public messages reach recipient 5, whose prior and listener inputs match, despite different hidden source belief/intent/classification. Here contradiction is factually correct; falsity alone cannot reveal intent.

Total: 17 fixtures × 16 seeds = 272 games. Each report includes 18 decisions and 18 listener assessments, explicit controller settings, message origins/deliveries and outcomes.

## Metrics and counterfactuals

Compare TruthRetain→Truth, Truth→OriginalVote, Truth→ReverseOrder; Absent→each of six injected variants; OneTruth→Repeat; HiddenTruth→HiddenContradiction, for recipient 5 at each seed (176 pairs). Record matched original private inputs, identical final listener-visible inputs where applicable, candidate/score/margin changes, executed vote changes, verdict changes and null effects. Natural enablement/order comparisons have broader mechanisms and are not testimony-only interventions. Relabeled games are evaluated separately using inverse ID mapping.

Per fixture measure updates with candidate changes, score/margin changes, resulting correct/incorrect/unknown candidates, ignored message assessments, false assertions, sincere mistakes, intentional contradictions, rejected actions, abstentions, correct/incorrect/no verdict. Assessment counts are executions across turns, not counts of independent observations. No inference of calibrated confidence or independent hidden causal sources.

## Validation

Test local-view isolation, hidden-intent invariance, mistaken and deceptive influence, conflict and null effects, repeat deduplication, private-evidence retention, legal resolution, executed vote gating, immutable historical input snapshots and valid causal/delivery references, permutations/order control, version/config validation, byte replay, CLI output protection, and all unchanged historical 001/002 tests. Run fmt, Clippy with warnings denied, full suite and frozen baseline comparisons. Freeze 003 evidence only once; later changes require a new version/artifact. Document actual outcomes, independent engineering validation, negative findings, and limits. No trust/reputation/learning/natural dialogue, and no Experiment 004.
