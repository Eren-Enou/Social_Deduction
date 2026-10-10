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

Historical status after 005: alternative missing-reference/revision acceptance rules, provenance repair, broader order/source structures and weights, richer voluntary correction strategies remained open. Persistent reputation, learned credibility and long-term memory were deferred until the following authorization for 006.

Decided for Experiment 006, rules `multi-incident-006-v1`, preregistered in commit `0124577` before implementation/evaluation:

- Wrap the unchanged 005 engine instead of duplicating resolution. Use six stable identities and at most eight independent incidents, reset temporary observations/beliefs/messages/votes/policies, and namespace cross-incident references by episode. Preserve all historical engines, tests and frozen evidence.
- Persist only six local source summaries and eight FIFO experience records per observer. Retain bounded consequences after detail eviction, log evictions and never use the audit archive to reconstruct forgotten claims. Scores start neutral and counters saturate u16.
- Deliver versioned lab verification to explicit recipients after completed incidents. Treat this as authenticated experimental truth under a trusted-controller integrity assumption, without cryptography. Collective verdicts and participant claims remain unverified, including incorrect group judgments.
- Ingest actually received message/interpretation storage only. Require a source's own original assertion for source responsibility. Deduplicate each perceived group/incident, track faithful relay accuracy separately, and leave unseen/ambiguous origins unknown. Private intent and observer classifications are never updater inputs.
- Grade original and final accuracy separately. Final verified correct/incorrect assertions add +1/-1, clamped [-2,2]; no final assertion adds zero. Track corrections/retractions without automatic trust reward. This chosen final-accuracy rule can reward a correct repair despite an originally wrong claim. Retain records feedback without factual score changes.
- Extend unchanged 005 revision/provenance interpretation: contributing named-source weight max(0,1+score), opaque weight 1, private anchors 2/6; preserve branch disputes, retraction and own-source suppression. Frozen005 executes the same experience learning without using its scores. Zero credit exactly matches 005.
- Preregister 22 fixture families, two listeners, seeds 0–7, 352 series/1056 incidents, full serialized replay, matched current information and adverse/null controls. Build credibility through executed histories, never directly from hidden truth. Separate secondary effects on selected communication from fixed-message comparisons.
- Record actual cognition inputs, feedback/ingestion input/output and storage/processing counters at execution time. Serialized bytes and deterministic inspection counts are proxies; no runtime/RAM benchmark or calibrated/human learning claim.
- Freeze new evidence only after formatting, warnings-denied Clippy, full tests and old artifact integrity checks. Refuse overwrite and commit preregistration, implementation and reproducible findings; no remote push without authorization.

Open after 006 authorization: alternative weights/update eligibility/retention, outcome-verifier realism, attribution validation, wider identity/order structures and richer voluntary strategies. Global reputation, coalition behavior, trained models and Experiment 007 are outside this task.


Decided for Experiment 007, preregistered in `e98ca72` before primary evaluation:

- Preserve001�006 source, test contracts and frozen bytes. Wrap unchanged005 resolution/006 baseline rather than modifying them. Use pinned Rust1.99.0, locked dependencies and SHA-256 for new canonical manifests only.
- Treat issuer authentication as a simulated transport fact distinct from factual accuracy. Declare all fault injections and recipient scope in observer records. A recipient cannot infer a first well-associated authenticated lie merely because the observer knows it is false.
- Compare first-accepted unchanged006 updating with conservative007 quarantine for retained contradictory outcomes. Recompute bounded retained contributions in historical order over a consequence checkpoint; do not subtract from clamped scores. Quarantine supplies uncertainty, not knowledge of which outcome was true. No rollback from an audit archive after eviction.
- Use simple session/episode/serial association checks, not production cryptography or arbitrary verifier competence. Missing verification stays unknown; verdicts remain unverified. Tokens contain no fault labels or objective accuracy.
- Evaluate neutral1/2, ordinal006 score and Beta(1,1) historical frequency separately from decision weighting. The latter is an explicit probability model whose calibration must be measured, not presumed. Do not change policies on the held-out seeds.
- Use64 independent generated worlds, six fault streams, two acceptance policies and two listeners, plus16 diagnostic families over four seeds. Report all-voter correctness/wrongness, abstention, coverage, conditional accuracy, verdict outcomes and utilities at three error costs. Bootstrap whole matched worlds, never count dependent votes as independent samples.
- Freeze compact causal evidence with complete canonical report/section hashes; regenerate full reports through the CLI. Refuse overwrite. Keep actual platform/revision/toolchain/lock metadata outside canonical evidence. Require real executions on both OS for a portability claim; unavailable Linux remains explicitly unverified.

Open after007: verified recovery after forgotten detail, alternative acceptance/checkpoint/weight/predictor rules, broader distribution shift and verifier competence, real authentication, global reputation and richer communication. Experiment008 has not begun.

Decided for Experiment 008, preregistered in `eea133f` before primary evaluation:

- Add a bounded scheduler because 005 has no stage hook. Preserve 001-007 engines and byte contracts; reuse 005 action validation/branch standing/majority,006 weights/ingestion,007 local verification and float-free encoding. No dependencies, ECS or trust fitting.
- Initialize at most three latent artifacts before policy selection. Physical and derived records are observer-distinct; derived records copy their original observation at initialization. Only public descriptors/allowed IDs enter a catalog. Candidate, accuracy and causal root are absent until legal access; accuracy/root remain absent afterward.
- Give each individual one optional inspection opportunity between discussion rounds. Validate independently; accepted inspection costs one persistent token even for missing/false content. Illegal attempts consume the opportunity without resource or delivery. Budgets 0-3, at most four incidents, no replenishment/transfers.
- Persist only effort and unchanged bounded local memories. Episode findings/messages reset. Selected and controller deliveries carry distinct origins and ledgers; controller findings cost zero and do not demonstrate initiative.
- Fix six transparent policies, source ordering and local random streams before evaluation. U/C/warning scores are declared heuristics; uncertainty, limited resources and unreliable histories do not reveal future usefulness.
- Combine original anchor 2/briefing 6, findings 2, explicit reports 1 and existing testimony weights. Use maximum per visibly shared group, zero on disagreement. Hidden dependence stays apparently separate. Neutral comparison includes the same environmental evidence with unchanged neutral testimony weights. Support is uncalibrated.
- Explicit report metadata is legitimate dialogue containing descriptor/content/acquisition reference, never private intent or actual roots. The first explicit report on a visible original/revision branch supplies its environmental identity; later revisions retain it. Independent inspections and exact relays share artifact identity. Speaker-owned public messages do not add self-support.
- Explicit acquired reports have no ordinary testimony evidence parent; their separate observer ancestry resolves the acquisition.005 Construction denotes the legacy wire-lineage placeholder, not independent physical evidence. Received reports have one exact received parent and retain the depth-three cap. Ordinary inference retains 005 depth validation; overlong proposals can be rejected and are counted, never bypassed. Environmental ancestry supplements legacy lineage without inventing an observation for an inferential speaker.
- Exclude explicit environment-report revision branches from 006 ingestion so repeated artifact reports do not manufacture independent witness credit. Ordinary conclusions continue unchanged 006 responsibility assumptions; alternative responsibility rules remain open. Log removed branches implicitly through captured ingestion input and preserved received communication/audit, never edit prior messages.
- Preregister 16 diagnostic families*4 seeds*6 policies and 32 held-out worlds*7 treatments,608 series/1824 incidents, whole-world paired bootstraps, resource/error utility sensitivity and compact causal traces/full section hashes. Preserve null/adverse results and distinguish information effects from selected acquisition. Identical Injected diagnostic configurations across six policy labels are declared duplicate controls, not independent units.
- Keep platform metadata outside canonical hashes, refuse output overwrites, freeze only after complete gates and old integrity checks. Real Linux execution remains necessary for portability claims. No push or 009.

Experiment 008 completion record: primary 608 series/1824 incidents replay exactly, with a separate matched cost 1/cost 3 control. The full 124-test pre-freeze gate and added frozen regression pass; all prior bytes/engines remain preserved. Main findings are context-dependent information benefit, conserved/spent resources, misleading/missing harm, trusted misinformation and no guaranteed improvement from accurate clues. Actual Windows manifest is frozen; Linux remains unavailable/unverified. No thresholds, weights or generator distributions were changed after inspecting held-out outcomes. No 009 or push. See experiments/008/results.md and validation.md for full counts, hashes and Git/source provenance.

Decided for Experiment009, preregistered in621e668: additive target selection and actual variable costs, not historical008 changes. Public catalog records costs/categories/instrument claims/coverage/visible provenance; latent results exist before selection. Seven policies and integer possible-result enumeration are fixed in experiments/009/specification.md. Reuse008 belief/communication and005-007 legality/memory rules. Actual accepted inspection spends1-3 units even for missing/false evidence; one opportunity, no fallback/replenishment, conservation per observer. Known overlap uses local existing source evidence, unknown roots stay unknown. Method-key ties are declared semantic preferences rather than numeric ID/order preferences. Record original proposal if research intervention forces a choice; never label forced/free injected evidence voluntary. Use64 disjoint held-out worlds and separate16-family diagnostics, observer alternatives and selected/forced/injected controls. Preserve null/adverse outcomes; no held-out tuning or push/010. Clarifications before evaluation preserve original preregistration and acknowledge decision-change forecasts can reward harmful uncertainty.

Experiment 009 completion: 960 primary series/2880 incidents and 324 separate controls/alternatives replay, with 1284 full canonical section sets. Actual prices 1-3 conserve 18144 primary units=10151 spent+7993 retained. Complete 155-test gate and post-freeze regression pass. Value heuristics save resources but fail to show a general factual advantage; explicit reservation helps in scarcity diagnostics, and missing/misleading/known-versus-hidden provenance harms persist. Windows matches; Linux unverified. Source/tests committed before create-once data freeze ec98fa2; no held-out tuning, push or 010. See experiments/009/results.md and validation.md.
