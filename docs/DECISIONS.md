# Decisions

Decided for rules version incident-001-v1:

- Six IDs (0–5); one engine-selected actor displaced a shared marker. No hidden role powers or incentives.
- Observation intervention: individuals 0 and 1 see the actor; 2 sees a mistaken candidate (next ID); 3–5 receive no identifying observation. Controller assignments are recorded, not agent initiative.
- Two discussion rounds, stable ID order, one silence or candidate claim per opportunity. Claims may cite a prior public claim as a response, but need not do so.
- One simultaneous private ballot per individual after discussion. Abstention legal. Four matching votes required; ties and insufficient support yield no verdict. Actor correctness is observer-only evaluation after termination.
- Initial belief copies the local observation. Baselines do not revise beliefs or treat public assertions as verified evidence. Memory is unbounded within this finite game; no evictions.
- Silent policy always abstains; truthful policy reports its belief once and votes it; seeded-random policy chooses silence/claim and abstention/actor without privileged information. Random claims have unspecified intent, not intentional-deception labels.
- SplitMix64 with per-agent streams, explicit algorithm and policy versions. No Bevy: simple sequential transitions suffice.
- Illegal actions are rejected, logged, and consume their scheduled opportunity; no ballot is substituted for an illegal ballot.
- No license selected. No commits or pushes performed by development automation.

Historical status at 001 completion: richer evidence, belief revision, incentives, bounded memory, and deception policies were open/deferred. The following section records later 002 decisions without changing 001.

Decided for Experiment 002, rules `incident-002-v1`:

- Separate module/config/CLI and frozen evidence; preserve the entire 001 replay contract.
- Add private participation briefing only to the responsible individual, enabled by default and explicitly logged. Initial belief gives this direct self-knowledge precedence over the unchanged witness observation; no historical observation is deleted.
- Four policies: truthful, silent, explicit deceptive control, and incentive-sensitive. No LLM, ECS, personality, trust or learning.
- Nonnegative private weights score faithful assertion, intentional contradiction and silence; uncertainty is available with no candidate belief. Formulas, tie order, and weights are preregistered in experiments/002/specification.md. These uncalibrated signal preferences do not model actual listener persuasion or payoff transfers.
- Keep two legal discussion opportunities per individual; baselines choose at most one message. Silence, uncertainty, or abstention remain legal. Public messages contain no private belief or intent labels.
- Simultaneous ballots remain sincere-to-local-belief except silent abstention, providing a deliberate negative control for speech effects. Four matching votes still required; no rule changes to force verdicts.
- Classification needs decision-time belief plus selected intent; objective falsehood is a separate metric. Inconsistent or unsupported intent is unsubstantiated, not deception.
- Seeds 0–31 and eight fixtures are evaluated without tuning; all null and mistaken results preserved. Actor-only incentive assignment is an explicit controller intervention, not policy access to truth.

Historical status after 002: listener sensitivity, alternative utility assumptions, calibrated consequences, richer evidence, belief revision, bounded memory and social learning were open; 003 was deferred until subsequently authorized.

Decided for Experiment 003, rules `incident-003-v1`:

- Separate versioned engine/config/evidence; preserve 001/002 source behavior, tests and frozen findings.
- Recompute uncalibrated support from original private evidence: observation weight 2; participation briefing weight 6 instead of observation contribution. Preserve the original observation even when it disagrees with current belief.
- One support point for the first asserted candidate per other visible speaker. Own speech, uncertainty and later same-speaker assertions add no support. No hidden source correlations or trust scores are accessible to listeners. Unique positive maximum selects a candidate; ties/missing support yield uncertainty.
- Retain and evidence listeners are replaceable. Current, original and abstaining judgment policies are separately replaceable; all legal majority/tie/termination rules remain fixed. Reuse 002 communication and classifications through a local current-belief adapter.
- Before each action record local inputs, previous support, per-message assessments and new support. Record message delivery and action effects separately. Objective correctness is evaluated only by observer code.
- Distinguish voluntary policy-selected messages from scripted interventions. At most 12 injected messages under a separate explicit controller budget; do not label them agent initiative.
- Preregister 17 fixtures × seeds 0–15, 176 recipient comparisons, one relabeling and one order reversal. Keep mistaken, wrong, uncertain and unchanged outcomes; no tuning on these results.

Historical status after 003: alternative support weights, first-assertion correction rules, observable provenance, broader speaking-order/source-structure controls and incentives under different listener behavior were open; persistent trust, reputation, long-term memory and learning remained deferred. The following records subsequently authorized 004 decisions.

Decided for Experiment 004, rules `incident-004-v1`:

- Prioritize provenance before reputation learning: speaker count in 003 conflated repeated information with separate support. Learning credibility on that foundation could reward repetition or conceal dependency. First measure the effect of source relationships that listeners can actually see.
- Preserve 001–003 source behavior and frozen evidence; add a separate module, CLI, config and artifact. No new dependencies or ECS.
- Extend delivery only to explicitly permitted recipient subsets. Retain six individuals, two rounds, legal silence/abstention, simultaneous ballots, four-vote threshold and bounded controller injections.
- Separate wire Message attribution from private selected basis/strategy and observer Lineage. Firsthand, named, unspecified and prior-message references are claims. Even visible matching links do not verify actual origins.
- Exact-copy relays require a previously received assertion. Voluntary relay baseline forwards only when its current candidate agrees, otherwise remaining silent. Explicit misleading attribution and unverified naming remain distinct from original belief-contrary intent.
- Track observation acquisitions, briefings or constructions as observer roots, with immutable parents, selected decisions, scoped deliveries and maximum depth three. These count acquisition chains, not statistical independence. Inference lineage uses declared basis and recorded contributing evidence, not a complete model of arbitrary policy internals.
- Reuse unchanged 003 source-count code. Provenance listener counts the first declared group once using weight 1; private anchors retain weights 2/6. Missing references use opaque ID groups under an unverified association assumption. Invalid or overlong visible chains add zero; no audit lookup. Returned own-source echoes add no support.
- Preserve first-assertion semantics, including blocked later corrections. Do not silently fix correction/retraction behavior in 004.
- Preregister 13 fixtures × two mechanisms × seeds 0–15, matched listener pairs, hidden-origin and attribution-only controls before primary evaluation. Preserve incorrect and unresolved judgments; no weight tuning.
- Freeze only after formatting, warnings-denied Clippy and tests; refuse existing outputs. Report serialized storage/communication counts as proxies, not runtime or RAM benchmarks.

Historical status after 004: correction handling, alternative weights, broader order/source structures, richer voluntary strategies and attribution validation were open; reputation, learned credibility and long-term memory remained deferred. The following records subsequently authorized 005 choices.

Decided for Experiment 005, rules `incident-005-v1`:

- Separate versioned module/config/CLI/evidence; preserve all 001–004 engines, tests and frozen reports. Correction handling follows evidence of first-assertion limitations, before reputation learning.
- Keep six individuals, two discussion opportunities, six ballots, four-vote threshold and voluntary abstention/silence. Replace the one-message policy restriction only in new correction policies; Retain adapts unchanged 004 behavior.
- Use explicit Revise referencing an own earlier assertion/revision, with optional replacement and forwarded received revision reference. Retraction communicates uncertainty without an alternative. Never mutate/delete the target or revise another speaker's claim.
- Controller delivers at most 12 later private observations before declared decision slots. Latest own observation supersedes the weight-2 anchor; None withdraws it. Preserve old observations and briefings. Observation accuracy is not guaranteed; schedules and recipients remain outside individual views.
- Sincere revises upon local mismatch; Selective uses explicit benefit-minus-cost and ties to silence; Misleading is an instrumented contrary control. RelayRevision forwards a received correction only on local agreement and cost selection. Record recognition independently from communication.
- Compute visible branch leaves through explicit targets; conflicting leaves dispute rather than newest-message priority. Valid revisions conserve original group identity. Conflicting active group members contribute zero; retraction withdraws only that speaker's support. Missing originals/forwarded references cannot be repaired from audit. Require a visible source revision for forwarded-revision acceptance, retaining the resulting useful-information rejection as a negative finding.
- Compare against frozen 004 testimony semantics using an original-message projection. Both 005 listener variants share later private observations; do not describe that extension as historical 004 behavior.
- Separate evidence parents/roots from revision ancestry, with evidence depth three. Classify repair, still-wrong sincerity, uncertainty, intent-supported contradiction, faithful forwarding, withheld opportunity and insufficient evidence separately. Do not retroactively label repaired mistakes lies.
- Preregister 18 fixtures × two listeners × seeds 0–15, 288 listener and 384 mechanism comparisons. Report confounding when changed listeners alter selected forwarding, distinct from identical-message comparisons. Count true recovery separately from uncertainty or changed wrong answers.
- Freeze only after engineering gates, preserve null/negative results and all old hashes. Additional post-freeze engineering controls change no primary mechanism or evidence. Serialized bytes/copies are storage proxies, not heap/runtime benchmarks.

Open after 005: alternative missing-reference/revision acceptance rules, provenance repair, broader order/source structures and weights, richer voluntary correction strategies. Persistent reputation, learned credibility and long-term memory remain deferred. No subsequent experiment has begun.
