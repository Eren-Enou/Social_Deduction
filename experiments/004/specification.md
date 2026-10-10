# Experiment 004: Evidence Provenance, Information Relays, and False Corroboration

Status: implemented, validated and evaluated; evidence frozen after engineering validation on 2026-10-09. The mechanisms, hypotheses and primary fixtures below were preregistered before implementation/primary evaluation and were not tuned afterward. Preserve 001–003 engines, validation and frozen evidence. See results.md and validation.md for actual outcomes and limitations.

## Versioned mechanisms

`incident-004-v1`, `provenance-004-v1`, `provenance-group-004-v1`, `relay-004-v1`, `config-004-v1`. Six participants, two discussion rounds, one communication opportunity per turn, simultaneous private judgment, four matching votes and legal abstention are unchanged. Recipient-scoped delivery is the sole communication-rule extension: a policy may choose any subset of its declared permitted recipients, or silence. At most 12 selected messages plus 12 separately logged controller injections. Actual relay depth is capped at three links. Game-local memory retains all received messages; no evictions or long-term memory.

A delivered Message contains immediate speaker, proposition/uncertainty and reported attribution: Firsthand, Unspecified, NamedSource(ID), or Reference(message ID). These are claims, not verified origins. A sender may cite only an earlier message it received; recipients receive the reference ID, not an inaccessible parent payload. Named-source claims are allowed without access to that source's private state. A visible reference can establish that matching public propositions and an earlier message exist, not that the sender actually obtained its information there.

Separate private selected action basis: PrivateObservation, Inference, or Received(message ID). Received basis requires an actual prior delivery to sender, exact asserted-proposition copying and depth ≤3. Private basis must match original observation or briefing. Inference audit roots are reconstructed from the recorded current candidate's contributing private evidence and message assessments; unsupported construction has its decision as an audit root. Observer-only roots identify separately assigned observation acquisitions/briefings or constructions, parent links, decisions and deliveries. They measure information acquisition chains, not statistical independence of witnesses sharing an incident. Reported attribution can disagree with actual basis. Neither actual basis, audit roots/depth, intent, truth nor classification travels in Message.

Reused 002 truthful/deceptive/silent/incentive-sensitive policies are adapted to current local belief and available scoped recipients; original-supported reports select private basis, other assertions select inference basis. A replaceable relay baseline forwards its first available assertion from the configured immediate speaker only when its current candidate agrees. Otherwise it chooses silence and may reconsider on its second opportunity. Once it speaks it remains silent. Presentation may disclose the exact received parent, leave origin unspecified, claim firsthand (explicit misattribution control), or name an unverified source. Faithful copying is classified as faithful relay even when false; selected misleading attribution is a separate strategy. Only belief-contrary assertions with recorded contradictory intent are original intentional contradictions. Inaccurate unverified naming is not itself proof of intentional deception.

## Listener algorithm specified in advance

Private observation contributes 2; briefing contributes 6 instead; support is recomputed, never accumulated from prior scores. Unique positive maximum chooses candidate; ties/missing evidence produce uncertainty. Keep first-assertion-per-speaker semantics, ignoring later corrections/retractions. Ignore uncertainty and one's own statements.

Source-count baseline directly calls the unchanged `source-count-003-v1` code after stripping attribution. Each other speaker's first assertion contributes 1.

Provenance-aware listener resolves groups using only its received Message list:

- Firsthand/Unspecified: immediate speaker group, weight 1; firsthand origin is unverified.
- NamedSource: named-speaker group, weight 1, explicitly unverified. This trusts the reported association for grouping, not truth or real independence.
- Reference: follow visible earlier matching-proposition references, at most three links. End at the declared speaker/named group. If a parent is absent, use its opaque message-ID group, weight 1, mark MissingReference; no archive lookup. Two matching claims citing the same missing parent share a perceived group under an explicit unverified grouping assumption.
- Matching public links are recorded as VisibleLink, while `origin_verified` stays false. Inconsistent propositions, malformed/forward/cyclic references or excessive visible depth add zero support and are recorded as uncertain provenance.
- First assertion per perceived group contributes 1; later group members add zero (duplicate discount), including conflicts. A group visibly attributed to oneself adds zero (returned own-source echo). Unknown hidden dependencies are not grouped by the audit.

A speaker's first malformed assertion still occupies its first-assertion slot; no correction rule is silently introduced. All assessments record groups, checked visible reference IDs, verification limits, acceptance/discount/ignore/uncertainty and resulting support. Judgment independently selects current/original belief or abstention via replaceable interfaces.

## Hypotheses

H1: visible common-source references reduce repeated-information support relative to source-count, and may change ballots/verdicts with a competing private observation.

H2: identical listener-visible inputs yield identical assessments/behavior despite different hidden actual roots. Hidden or falsely independent origins cannot be discovered through audit data.

H3: inaccurate declared associations can over-discount independent information or make common-source information appear independent. Provenance awareness is not inherently better; preserve incorrect and unresolved results.

H4: false information can be faithfully relayed without labeling every forwarder an intentional liar. Silence/disagreement/missing assertions remain legitimate non-relay outcomes.

## Primary fixtures (chosen before implementation)

Seeds 0–15; both SourceCount and Provenance listeners in every fixture (same six-person listener mechanism within a game). Fixed speaking order [0,1,2,3,4,5], no briefing; source 0 initially observes proposition P, recipients 3–5 each observe competing Q; sources/relays 1–2 otherwise unknown. Current-belief voting throughout. For true-origin fixtures P=actor, Q=next ID. For mistaken/deceptive origins P=next ID, Q=actor. Source 0 speaks to all; relays 1–2 send to recipients 3–5 unless specified.

1. Independent: sources 0–2 separately observe P and truthfully assert firsthand; source routes include peers and recipients.
2. KnownShared: source 0 truthfully reports P; 1–2 disclose relays of message 0.
3. HiddenShared: same actual basis as KnownShared, but relays claim firsthand. Recipient messages match Independent despite different hidden roots; this is an explicitly misleading-attribution control.
4. UnspecifiedShared: same relays with no origin claim; listeners cannot infer dependence merely from shared proposition.
5. OpaqueShared: source 0 sends only to 0–2; relays cite its ID to 3–5, who never receive the parent payload.
6. Chain: 1 relays 0 to 2–5; 2 selects 1's relay and discloses it to 3–5. All earlier public links are available to final recipients.
7. WrongAttribution: shared-origin relays name source 3, unverified and actually incorrect, instead of the true parent.
8. CollapsedIndependent: Independent except sources 1–2 name source 0 despite reporting their own observations. Audit independence remains three acquisitions; perceived grouping may collapse it.
9. MistakenOrigin: KnownShared with source 0 sincerely observing false P and recipients privately observing true Q.
10. DeceptiveOrigin: KnownShared with source 0 observing true Q and deliberately asserting P. Relays faithfully repeat P.
11. NoMessage: source 0 silent; 1–2 are eligible relay policies but receive no assertion.
12. Disagreement: KnownShared except relay 1 originally observes Q and may refuse to relay P.
13. InjectedShared: communication policies silent; controller injects source 0's private-supported assertion and two exact disclosed copies under Received basis, delivered along the same source/recipient routes. These are interventions, not individual choices.

Total 13 × 2 × 16 = 416 games. Within-fixture listener pairs isolate the listener mechanism; confirm identical delivered messages and initial settings except listeners where possible (selected policies may respond to revised beliefs). Additional Independent vs HiddenShared receiver-5 controls must have identical visible update inputs, while observer roots differ. KnownShared vs UnspecifiedShared comparisons change only relay presentation; Independent vs CollapsedIndependent change only reported attribution. Compare KnownShared and Chain final grouping separately from path/order overhead. Do not tune weight 1/2/6 or thresholds to manufacture verdicts.

## Measurements and gates

Per game record selected/injected assertions and relay counts, unique public speakers, objective acquisition/construction roots; per final listener record actually received speakers, observer-only roots behind received assertions, distinct perceived groups, contributing groups, accepted/discounted/ignored/uncertain assessments, verified-link status without verified-origin claims, scores/candidate/margin and ballots. Measure candidate/score changes, false assertions, sincere mistakes, original contradictions, faithful false relays, attribution mismatch, correct/wrong/no verdict, maximum depth, delivery copies, stored message copies and serialized attribution/audit-lineage byte overhead (not runtime benchmarks or RAM allocation). Report repeated-information contributions avoided relative to baseline, not alleged calibrated confidence.

Validate received-only relay access, inaccessible/invalid/future/self-cyclic references, matching copies, recipient scoping, depth bounds, lineage/delivery/decision references, hidden-root invariance, unverified-firsthand/named/opaque-source semantics, independent/shared/incorrect grouping, false relay classification, disagreement/silence, executed votes, immutable audit inputs and historical frozen checks. Run fmt, warnings-denied Clippy, full suite and primary byte replay. Freeze evidence with create-new only after gates pass; add frozen-evidence regression check afterward. Document negative outcomes, assumptions and limitations. No reputation, learned credibility, long-term memory, strategic planning, coalitions, natural dialogue, GUI or networking. Corrections remain deferred.
