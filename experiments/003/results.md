# Experiment 003 findings

Completed 2026-10-09 with Rust/Cargo 1.99.0 and unchanged Cargo.lock dependencies. The listener, hypotheses, fixtures and seeds were specified before the primary evaluation. No listener weights or rules were tuned after evaluation.

## Executed evidence

Frozen `evidence/experiment-003.json` contains 272 games (17 fixtures × seeds 0–15), 4,896 listener assessments, 4,896 subsequent decisions, message origins and deliveries, and 176 matched recipient comparisons. Every serialized replay matched; zero baseline decisions were rejected. Versions: incident-003-v1, config-003-v1, source-count-003-v1, belief-vote-003-v1, builtin-003-v1, evaluation-003-v1. Communication reuses incentive-baselines-002-v1 through a current-belief adapter.

| Fixture (16 games each) | Candidate-changing updates | Updates to wrong candidate | Votes different from original belief | Abstentions | Correct / wrong / no verdict |
|---|---:|---:|---:|---:|---|
| TruthRetain | 0 | 0 | 0 | 37 | 0 / 0 / 16 |
| Truth | 63 | 0 | 50 | 0 | 16 / 0 / 0 |
| Deceptive | 67 | 61 | 63 | 6 | 0 / 0 / 16 |
| Silence | 0 | 0 | 0 | 37 | 0 / 0 / 16 |
| Blame | 16 | 16 | 16 | 21 | 0 / 0 / 16 |
| OriginalVote | 63 | 0 | 0 | 37 | 0 / 0 / 16 |
| Relabeled | 63 | 0 | 50 | 0 | 16 / 0 / 0 |
| ReverseOrder | 55 | 0 | 50 | 2 | 16 / 0 / 0 |
| Absent | 0 | 0 | 0 | 64 | 0 / 0 / 16 |
| OneTruth (injected) | 64 | 0 | 64 | 0 | 16 / 0 / 0 |
| OneMistake (injected) | 64 | 64 | 64 | 0 | 0 / 16 / 0 |
| OneLie (injected) | 64 | 64 | 64 | 0 | 0 / 16 / 0 |
| Conflict (injected) | 0 | 0 | 0 | 64 | 0 / 0 / 16 |
| Repeat (injected) | 64 | 0 | 64 | 0 | 16 / 0 / 0 |
| Uncertain (injected) | 0 | 0 | 0 | 64 | 0 / 0 / 16 |
| HiddenTruth | 80 | 0 | 80 | 0 | 16 / 0 / 0 |
| HiddenContradiction | 80 | 0 | 80 | 0 | 16 / 0 / 0 |

Update counts are executions, not distinct individuals; an individual can change candidate more than once. Votes are executed ballot choices, compared with that individual's original private candidate. Support scores and margins are not calibrated confidence. All injected fixtures have zero selected individual messages; their assertions are explicitly controller interventions. The silent communication policy does not force abstention: judgment is a separate replaceable policy in 003.

## Causal comparisons

For recipient 5, each absent→OneTruth/OneMistake/OneLie comparison changed belief, vote and verdict in all 16 matched seeds. Original recipient state and all other initial settings matched; the supplied proposition caused the difference. OneMistake preserves sincere source-belief agreement, whereas OneLie records a contrary intent and contrary source belief. Neither classification is supplied to the listener. These controls establish susceptibility to correct and incorrect testimony, not voluntary persuasion. Separate engineering fixtures also execute selected truthful-to-mistaken-belief and selected deceptive speech without any injection and verify the same wrong-belief/vote chain.

Conflict produced tied support and abstention in all 16 comparisons: support changed while candidate and behavior stayed unknown. Uncertainty had no support, candidate, vote or verdict effect. Repeating the same source's assertion produced identical scores, margins, votes and verdicts to one assertion in all 16 seeds; 192 repeat-assessment executions were ignored. A retain policy ignored testimony (1,392 assessment executions), and a stronger private observation retained its candidate against one competing assertion in tests. These are explicit heuristic rejection/retention behaviors, not detection of dishonesty.

HiddenTruth and HiddenContradiction supplied identical public messages from the same speaker. Recipient 5 had identical complete listener inputs and identical updates and judgment choices across all 16 pairs despite different source private beliefs, policy-selected intents and observer classifications. The deliberate contradiction happened to be factually correct. A listener cannot discover intention from the observer label because that label is never present in its view.

Enabling revision changed recipient 5's candidate and vote in 14/16 TruthRetain→Truth pairs; the other two retained its privately briefed self-knowledge. Verdicts changed in all 16 games. Truth→OriginalVote retained identical updates and testimony, but the original-belief voting gate prevented all 50 ballot changes across the six-person games and restored unresolved outcomes. This demonstrates the causal link from current belief to executed action independently of majority-rule changes.

## Natural-policy outcomes and confounding

Truth produced 16 correct verdicts, but this is a result of the explicit evidence weights, fixed observations, private actor briefing, and coded policies. It is not proof of intelligent reasoning. The mistaken witness often became uncertain before its turn; Truth therefore produced zero sincere mistaken messages, while TruthRetain produced 13. The original mistaken observations remain in every report. Changing a belief did not retrospectively change an observation or old communication.

Deceptive emitted 92 deliberate contradictions and generated 61 transitions to wrong candidates, yet every game remained unresolved. Blame emitted 30 contradictions, generated 16 wrong-candidate transitions, and also reached no verdicts. Positive support margins can accompany wrong beliefs; successful communication influence does not imply accurate knowledge or successful game outcomes.

The separately evaluated ID relabeling preserved mapped final support scores, candidates, ballots and verdicts in all 16 seeds. This establishes symmetry for the tested truthful configuration, not all permutations or all policies. Reversed speaking order produced 6 sincere mistaken messages, versus zero in Truth, and altered recipient 5's final scores in 11/16 pairs. Recipient 5's candidate/vote and verdicts stayed unchanged; across all individuals abstentions increased to two. Order remains a measurable confound even when the verdict agrees.

## Conclusions and limits

The narrow implementation hypotheses are supported: received testimony can cause local belief changes and resolved votes; identical visible information cannot reveal hidden intent; declared controls preserve uncertainty and null effects. Incorrect adoption is an erroneous inference under a literal scoring rule, not demonstrated linguistic misunderstanding. There is no semantic language model.

This is a programmed heuristic, not emergent social intelligence or calibrated probability. Counting distinct public speakers does not establish independent causal sources: testimony relays may share an original witness, and listeners are not given that hidden provenance. The first-assertion rule can ignore genuine corrections; private/testimony weights and briefing dominance are assumptions. Only one relabeling and one order reversal were evaluated. Broader source structures, alternative policies, cross-platform replay, and hostile custom host-side policies remain untested. No persistent trust, reputation learning, long-term memory, natural dialogue, strategic planning or Experiment 004 was implemented.

See validation.md for engineering checks, initial failed test expectations, and historical integrity. No commits or pushes were made.
