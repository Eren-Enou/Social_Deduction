# Experiment 005: Correction, Retraction, and Belief Recovery

Status: implemented, validated and evaluated; evidence frozen after engineering validation on 2026-10-09. Mechanisms, hypotheses and primary fixtures below were preregistered before implementation and primary evaluation, without tuning afterward. Preserve engines, tests and frozen evidence 001–004. The supplied request ends midway through comparison J; selected versus explicitly injected revision comparisons implement its stated comparison within the completed scope. See results.md and validation.md for actual outcomes. No reputation, learning, natural dialogue, networking, GUI or subsequent experiment.

## Mechanisms and hypotheses

Versions: incident-005-v1, config-005-v1, revision-005-v1, revision-listener-005-v1, correction-policy-005-v1. Six individuals, two discussion opportunities each, six private ballots, four matching votes, legal silence/abstention. At most 12 selected messages, 12 controller injections, 12 later private observations. All snapshots/history immutable. Game-local received/sent message lists and private observation history are finite and unbounded within the game; no evictions.

H1: a sincere own-claim revision can change recipient support and executed ballots relative to the frozen 004 listener, without changing objective truth or the earlier record. H2: private change or withholding alone cannot change a recipient with identical visible inputs. H3: wrong sincere and deliberately misleading corrections can mislead; corrections are not privileged truth. H4: relayed revisions can be selected or withheld, and scoped delivery may leave stale or conflicting source-group members. H5: contested revisions and missing originals produce uncertainty/ignored revisions under the declared rule, rather than archive reconstruction or newest-message priority.

## Wire content, legality and causality

Message has ID, immediate speaker, claimed 004 Attribution and Body: Assert(candidate), Uncertain, or Revise {target, replacement: Option(candidate), forwarded: Option(message ID)}. Replacement None is retraction/uncertainty. A revision targets an earlier assertion or revision by the SAME speaker; the speaker must possess it in its own sent record. It can revise its own relay, never another speaker's statement. Multiple revisions of one target are legal competing branches. Subsequent sequential revisions target the earlier revision. An ordinary repeated assertion is not a correction.

Actual basis, selected intent, private belief, correctness, roots and delivery history never travel in Message. Private Basis records a supporting own observation, local inference, an exact received assertion, or exact received revision. Received revision requires an earlier delivered Revise with the same replacement and an explicit forwarded ID; the relay's target is its own previous communication. Claimed prior-message attribution requires receipt; firsthand/named sources remain unverified. Observer lineage separately records evidence parents, observation acquisitions/construction roots, bounded evidence depth three, and revision_of historical dependency. Revising history is distinct from acquiring evidence; the earlier target is not automatically a supporting evidence root for the replacement.

Controller observations are delivered before declared decision slots 0–17 to one individual only and logged. Observation {candidate: Option(ID)} supersedes that individual's private anchor (weight 2), preserving every earlier observation; None withdraws that anchor. This is a declared new assessment, not an error label or guaranteed truth. Participation briefing, if explicitly configured, retains weight 6 precedence. Listeners never see someone else's observation assignment or intervention schedule.

## Policies specified in advance

Retain adapts unchanged 004 baseline/relay communication, preserving its once-spoken restriction. Sincere starts by asserting its current candidate (or uncertainty) and on its second opportunity revises its last own assertion if the current candidate differs; unknown candidate retracts. Selective uses the same available action with local revision benefit B and cost C: score revise=B-C, silence=0; strict positive selects revision, ties silence. Initial reporting remains sincere. Misleading control deliberately replaces with (current candidate+2) mod 6, with contrary intent; unknown belief cannot support a contradiction and yields silence. These are programmed rules, not remorse or spontaneous manipulation.

RelayRevision first forwards a received source assertion only when its current candidate agrees. On the next opportunity it may revise its own relay using a received source revision with matching current candidate/retraction uncertainty, explicit received-revision basis and forwarded reference; cost/benefit gate may select silence. Its earlier source must be the same configured speaker. All policies can choose no message. Own sent records are local even when a message was not delivered back to oneself. Decisions record recognized mismatch/eligible source correction and local option scores independently from selected action.

## Listener standing and contested revisions

Frozen baseline calls unchanged 004 Provenance on only original Assert/Uncertain messages, ignoring typed revisions explicitly. Effective private anchor uses the same later observation in both listener variants, so comparison isolates testimony revision handling. Baseline projection keeps original IDs and provenance; references to revision messages remain inaccessible in that projection.

Revision-aware listener recomputes from received records only. First assertion per speaker becomes an initial branch, with group resolved through 004 visible provenance; own statements contribute zero. Later ordinary assertions stay ignored. A valid same-speaker revision follows a visible earlier assertion/revision, resolves to that original branch/group, and becomes a child standing. Missing original, foreign ownership, cycles/forward references or malformed provenance add no replacement support and retain explicit statuses. No payload is reconstructed from audit. Forwarded reference must be a visible earlier revision with matching replacement; it confirms visible copying only, never actual source truth/intent.

Current branch leaves determine standing: one leaf supplies its replacement; multiple leaves with identical replacement agree; distinct replacements (including retraction versus assertion) dispute that speaker's branch and contribute zero. Sequential revisions are accepted through explicit links, not mere latest timestamps. Retraction leaves provide no support. Historical ancestors remain recorded as superseded. Original malformed branches remain uncertain even if later revised; repairing provenance is deferred.

Perceived provenance group remains the ORIGINAL branch group for revisions: changing attribution cannot manufacture a new independent group. For active branches in one group, matching candidates contribute one; conflicting candidates or a disputed branch make the group contested and contribute zero. A retracted member is removed but does not retract another speaker's stale relay. Returned own-source group contributes zero. Different groups provide separate competing points. All original source claims stay unverified. Unique positive score maximum selects candidate, ties/absence cause uncertainty; scores are heuristic, not probabilities.

## Primary fixture family

Seeds 0–15, both Frozen004 and Revision listeners: 18 fixtures × 2 × 16 = 576 games. Fixed order [0,1,2,3,4,5], no briefing, current-belief judgment. Initial P=(actor+1)%6, source0 privately P; others unknown. Source0 reports to all in round1, then gets Q=actor before slot6. Other communication silent unless specified. At slot6 source0 can revise; scoped first/second route tables are declared independently. No weights or fixtures tuned after primary outcomes.

1. Retain: source0 retains its old public assertion after privately changing to Q.
2. Sincere: source0 sincerely corrects P to Q, both rounds to all.
3. Retraction: later private observation None; source retracts without replacement.
4. Misleading: source knows Q but deliberately corrects to (Q+2)%6.
5. WrongSincere: new private evidence Q=(actor+2)%6 is mistaken; sincere revision stays wrong.
6. SelectiveLow: sincere selective correction B=2,C=1.
7. SelectiveHigh: identical except B=2,C=3; chooses silence despite changed belief.
8. RelayBoth: source0 initial to all; relay1 forwards to 2–5; source revision reaches relay1 and final recipients; relay voluntarily revises its own statement.
9. RelayIntermediate: source revision reaches only 0/1; relay1 can forward its own correction, final recipients retain source0's stale original.
10. RelayFinal: source revision reaches 0/2–5, not relay1; relay1 keeps its earlier report.
11. RelayNeither: source revision reaches only0, neither relay nor final recipients.
12. RelayWithheld: same source delivery as RelayIntermediate, relay correction benefit0/cost1 suppresses forwarding.
13. MissingOriginal: source initial reaches only0/1; source correction reaches all; 2–5 cannot validate the target.
14. Independent: source retain; source2 independently observes true Q and reports in round2 (not a revision); listeners may tie separate groups rather than count a replacement.
15. Injected: all policies silent; controller injects source0 initial before slot0 and same sincere correction before slot6, alongside the same private observation. Wire/delivery stream matches Sincere; origins differ explicitly.
16. Branches: silent policies; injected source0 P before slot0, corrections targeting same initial message before slots6 and7 to Q and R=(actor+2)%6. Competing leaves dispute rather than newest wins.
17. Sequential: same injections but second correction targets the first revision; explicit chain ends at R (wrong).
18. Dispute: Sincere plus independent source2 reports P in round2; separate groups compete. This is disagreement, not authority to revise source0.

Within-fixture listener comparisons record whether delivered streams match (listeners can change relay choices). Additional recipient5 comparisons Retain/Sincere, Sincere/SelectiveHigh, SelectiveLow/SelectiveHigh, relay route variants, Sincere/Injected, Sincere/Independent, Branches/Sequential and Sincere/WrongSincere record actual configuration differences, local inputs and executed effects. Hidden private-change control Retain vs SelectiveHigh checks equal recipient input/assessment/vote despite distinct policy choice mechanisms. Selected/injected comparison checks wire identity without treating injection as initiative.

## Measures and validation

Record recognized mismatches, selected corrections/retractions/withheld opportunities, forwarded corrections, injected revisions, sincere repairs of mistakes, sincere still-wrong revisions, misleading corrections, unclassifiable cases, original/active/superseded/retracted/disputed/missing/ignored message assessments, perceived contributing groups, support/candidate/margin, executed ballots, correct/wrong/no verdict, delivery copies, private observations, serialized messages/lineage/report size. Recovery is observer-only: compare recipient5's last round1 belief with judgment; true recovery requires prior wrong candidate and final correct candidate, distinct from uncertainty or reduced wrong support.

Test ownership, receipt, exact forwarded replacement, recipient scoping, IDs, versions, two-turn budgets, concurrent branches/sequential links, absent originals, unverified references, group conservation, stale relays, private-change isolation, false correction classification, immutable originals/snapshots, observations and causal/delivery references, legal votes/termination and byte replay. Preserve and run all historical frozen regressions and SHA-256 checks. Run fmt, Clippy warnings denied and full tests before create-new evidence freeze; add frozen 005 comparison afterward and rerun relevant gates. Document null/negative outcomes, no parameters tuned, cross-platform/RAM/performance/psychology untested.
