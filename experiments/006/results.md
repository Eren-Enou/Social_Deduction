# Experiment 006 findings

Evaluated 2026-10-09 on Windows with Rust/Cargo 1.99.0. Preregistration commit `0124577` preceded implementation and primary execution; implementation commit `a63fa3b` preceded the create-once freeze in `33afd2a`. Algorithms, weights, fixtures and seeds were not tuned after seeing primary outcomes. These are controlled consequences of explicitly programmed heuristics, not evidence of human psychology or calibrated social learning.

## Execution and causal comparisons

The frozen evidence contains 22 fixtures × two listeners × eight base seeds: 352 series and 1056 incidents, plus 352 successful byte-identical series replays. There are 176 listener pairs and 96 additional between-fixture controls. Each incident completes 18 legal decisions; zero selected actions were rejected. Full pretty JSON is 405,837,038 bytes, SHA-256 `34E27342C14C1B09FE44CD268E9D9F5DC38D7335797BBDBDB32DE4098FC5A360`.

All 176 listener pairs differ only in listener use of local credibility. Current configs, private inputs and complete memory-before snapshots match in every pair. In 168 pairs, present message bodies, identities, order and recipients also match: 120 target belief-score/ballot changes and 104 verdict changes. Forty-eight verdicts become correct, 56 become incorrect. The other eight pairs are Interaction: trust changes B's selected current statement, so they are secondary communication effects rather than fixed-message evidence; all eight become correct.

All Frozen005 assessment incidents remain unresolved and target5 abstains, because the opposing reports contribute equal support. Across Credibility's 176 assessment incidents there are 56 correct, 56 incorrect and 64 unresolved verdicts; target5 casts 56 correct votes, 72 incorrect votes and 48 abstentions. These totals summarize the chosen fixture mix, not expected performance on a sampled external distribution. Experience changes decisions without reliably improving their accuracy.

Between-fixture controls do not all preserve present information. Accurate/MissingFeedback, Accurate/DelayedFeedback and Accurate/InjectedExperience preserve current wire content at all eight seeds. Different history lengths select different current episode seeds; identity/order controls deliberately alter identity/order. These broader contrasts are not mislabeled fixed-present treatments. The within-fixture listener pairs provide the principal causal comparison.

## Per-family assessment outcomes

Each row summarizes eight Credibility assessment incidents. Frozen005 has 0 correct, 0 wrong, 8 unresolved for every family. Target is observer5; A is source0 except IdentityPermuted, where A is source3. Score is its locally acquired value before the assessment, not assigned evaluator truth.

| Fixture | Target's A score | Correct / wrong / unresolved verdicts | Target correct / wrong / abstain |
|---|---:|---|---|
| Neutral | 0 | 0 / 0 / 8 | 0 / 0 / 8 |
| Accurate | +2 | 8 / 0 / 0 | 8 / 0 / 0 |
| Inaccurate | -1 | 0 / 8 / 0 | 0 / 8 / 0 |
| DifferentHistories | -1 | 0 / 0 / 8 | 0 / 8 / 0 |
| MissingFeedback | 0 | 0 / 0 / 8 | 0 / 0 / 8 |
| WrongVerdict | 0 | 0 / 0 / 8 | 0 / 0 / 8 |
| Corrected | +1 | 8 / 0 / 0 | 8 / 0 / 0 |
| Uncorrected | -1 | 0 / 8 / 0 | 0 / 8 / 0 |
| MisleadingCorrection | -1 | 0 / 8 / 0 | 0 / 8 / 0 |
| RelayOrigin | -1 | 0 / 8 / 0 | 0 / 8 / 0 |
| OpaqueRelay | 0 | 0 / 0 / 8 | 0 / 0 / 8 |
| Misattributed | 0 | 0 / 0 / 8 | 0 / 0 / 8 |
| SelectiveFeedback | -2 | 0 / 0 / 8 | 0 / 8 / 0 |
| Reversal | -1 | 0 / 8 / 0 | 0 / 8 / 0 |
| IdentityPermuted | +2 | 8 / 0 / 0 | 8 / 0 / 0 |
| OrderReversed | +2 | 8 / 0 / 0 | 8 / 0 / 0 |
| InjectedExperience | +2 | 8 / 0 / 0 | 8 / 0 / 0 |
| TrustedFalse | +2 | 0 / 8 / 0 | 0 / 8 / 0 |
| DelayedFeedback | +2 | 8 / 0 / 0 | 8 / 0 / 0 |
| Eviction | -2 | 0 / 8 / 0 | 0 / 8 / 0 |
| Interaction | +2 | 8 / 0 / 0 | 8 / 0 / 0 |
| UnverifiedClaim | 0 | 0 / 0 / 8 | 0 / 0 / 8 |

## Supported mechanisms, adverse results and null results

H1 is supported within these controls: legitimately delivered experiences change later reliance and executed ballots. Accurate seed0 has two verified correct source0 reports, score +2 and weight3. In the current incident actor4 receives support3 against candidate5 support1; target5 votes4 and the verdict is correct. Frozen005 sees identical current messages, ties at 1:1 and remains unresolved. MissingFeedback executes the same histories/current wire but acquires no factual credit and stays unresolved.

H2 is supported by primary null controls and engineering interventions. MissingFeedback, WrongVerdict and UnverifiedClaim retain zero factual credit. Across both mechanisms there are 3552 feedback deliveries, including 288 unauthenticated deliveries; none of those 288 changes any credit summary. The wrong historical group verdict remains wrong observer history, not authenticated truth. Engineering controls change an unseen historical private state, the actual historical actor seed, or injected private intent while holding observer5's received experience constant; local memory and subsequent decisions remain identical. Selected versus injected histories produce the same target credit inputs/support/votes for all eight primary seeds despite different observer origins. These checks cannot rule out arbitrary malicious host-side side channels.

UnverifiedClaim announcements and verdict-feedback deliveries are scheduled controller diagnostics. They do not demonstrate voluntary participant outcome reporting. In-incident statements/corrections remain policy-selected except explicitly logged injection fixtures.

H3 is supported: DifferentHistories gives observer3 score +1 and observer5 score -1 for the same source. With identical present messages, 3 votes for the actual actor and 5 for the competing wrong candidate. SelectiveFeedback gives +2 versus -2. Both families remain collectively unresolved across all seeds. There are 24 observer3/5 score-vector disagreements per mechanism: eight in each of those families plus eight in IdentityPermuted, where observer3 is the source and self-experience is excluded. This is local perspective, not a shared reputation property.

H4's beneficial and adverse possibilities are both observed. TrustedFalse seed0 preserves source0's legitimate +2 history but reverses present accuracy: candidate5 gets weight3 while actual actor4 gets weight1. Target5 votes5 and an incorrect verdict replaces Frozen005's uncertainty. All eight TrustedFalse seeds do this. Inaccurate, Uncorrected, MisleadingCorrection, RelayOrigin, Reversal and Eviction also yield eight wrong assessment verdicts each because an earlier failure suppresses source A's now-correct report. Reputation neither verifies current testimony nor guarantees better judgment. A general improvement hypothesis is unsupported.

H5 is supported: every Reversal seed has source score path +1, +2, +1, 0, -1 after five authenticated experiences. The first impression is reversible, though the resulting negative prior harms its later correct testimony in this fixture. DelayedFeedback keeps score0 after incident0, then verifies retained incidents0/1 after incident1 and reaches +2 before assessment. It matches ordinary timely feedback's final target credit/support/ballot in all eight seeds.

H6 is supported under the declared eligibility rules. Corrected records one wrong original, one observed correction and one verified correct final assertion, score +1. MisleadingCorrection records the same correction count but a verified wrong final assertion, score -1. Correction choice alone supplies no accuracy credit. The initially mistaken sincere report is not relabeled intentional deception. Primary evaluation includes 80 observed corrections per listener mechanism, but zero retractions; retraction/disputed branches are covered by engineering tests, not a primary multi-incident retraction comparison. A correct repair's reward is an explicit final-accuracy assumption, not a discovery that correcting is universally trustworthy.

RelayOrigin gives observer5 one source experience despite an original plus two visible copies. Source0 gets one inaccuracy, not three; relay1/2 keep score0 with separate inaccurate-relay counters. Across each mechanism, 96 duplicate assertion contributions are prevented, 144 received relay reports are verified inaccurate, and 48 ambiguous-source records receive verification without assigning original-source credit. OpaqueRelay/Misattributed do not fetch hidden origin0 or punish named source3 without its received assertion; all assessment scores stay neutral. These rules do not solve malicious attribution, particularly a false association with an already trusted name.

Identity/source-role permutation and reversed order preserve target support and ballots in all eight tested seeds after interpreting the changed source identity. Only one role permutation and one reversal were studied; comprehensive identity/order independence is untested.

## Bounded storage and processing measurements

Per listener mechanism, final memories contain 1808 pending and 1336 retained verified records across all series. Credit summaries record 1288 verified final-source experiences: 712 accurate and 576 inaccurate; separate original accuracy counts are 672 accurate and 616 inaccurate. A verified record can have unknown source responsibility and is not equivalent to a graded source. Counters retain consequences after detail eviction.

Maximum local record count is eight. Each mechanism logs 32 detail evictions (four assessment recipients × eight Eviction seeds). Late verification of their evicted first experience has no eligible record and never reconstructs a claim. The `late_feedback_without_record` counter is 320 per mechanism but includes all delivered feedback lacking retained experience, such as a speaker receiving verification of its own excluded claim; it must not be read as 320 late evictions. Observer2 retains the first Eviction experience and may legitimately update when late verification arrives. Observers with evicted details retain separately accumulated earlier consequences.

Final compact local-memory serialization totals 2,140,928 bytes for Frozen005 and 2,140,992 for Credibility across 1056 final individual memories per mechanism; per-memory sizes span 987–3850 bytes. The 64-byte difference comes from changed current experience in Interaction. Compact series report sizes total 62,155,246 and 62,015,294 bytes respectively, excluding evaluation wrappers and pretty-print whitespace. The full frozen wrapper is 405,837,038 bytes. Audit snapshots intentionally duplicate local inputs and histories to permit review; they are not policy-accessible memory.

Each mechanism executes 9504 cognition callbacks, 9144 contributing-group weighting operations and 8832 retained-record inspection counts. Inspection counts measure lengths scanned for ingestion deduplication/feedback eligibility, not every internal source-interpretation operation. Both baselines execute the memory subsystem for matched inputs, so these numbers are not an incremental runtime benchmark against historical standalone005. No wall-clock efficiency or heap/RAM claims are supported.

## Limits and untested questions

The transparent ±1, capped-score and 0–3 weighting mechanisms are programmed. Their causal effects are observed here; sophisticated social learning, calibrated trust, moral honesty, optimality, emergence and external generality are unsupported. Incomplete verification can bias legitimate local assessments. Neutrality and unresolved verdicts remain valid outcomes.

Trusted verification is deliberately supplied laboratory truth. Faulty/corrupted authenticated feedback, cryptographic verification, natural discovery and verifier credibility are untested. No tests sandbox hostile Rust policies; interfaces prevent ordinary accidental access. Histories are short, witnesses/roles are controlled, evaluation seeds are small and preregistered, and cross-platform execution is untested. No reputation sharing, personality, coalitions, trained models, GUI or Experiment007 was implemented. See validation.md for exact engineering gates and preserved historical evidence.
