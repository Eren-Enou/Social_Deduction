# Experiment 002 results

Completed 2026-10-09 with Rust/Cargo 1.99.0 and the existing locked dependencies. Rules `incident-002-v1`, policies `incentive-baselines-002-v1`, configuration `config-002-v1`, evaluation `evaluation-002-v1`, PRNG `splitmix64-v1`. Predictions and scores were specified before implementation and evaluation in specification.md. No parameters were changed to improve outcomes after evaluation.

## Executed evidence

`evidence/experiment-002.json` contains 256 complete games: eight fixtures × seeds 0–31, 4,608 decision-time input/action/intent/classification snapshots, controller assignments, resolver effects, public messages, and final outcomes. All 256 serialized replays matched. All 4,608 baseline decisions were legal; zero rejected or unsubstantiated actions. Each fixture has 384 discussion opportunities; counts below include silence on the second round after a policy has spoken. Ballots are excluded from these communication counts.

| Fixture | Sincere true | Sincere mistake | Intentional contradiction | Silence | Uncertainty | Verdicts |
|---|---:|---:|---:|---:|---:|---:|
| Truthful control | 89 | 26 | 0 | 192 | 77 | 0 |
| Silent control | 0 | 0 | 0 | 384 | 0 | 0 |
| Deceptive control | 0 | 0 | 115 | 269 | 0 | 0 |
| Accuracy incentives | 89 | 26 | 0 | 192 | 77 | 0 |
| Blame incentives | 0 | 0 | 35 | 349 | 0 | 0 |
| Quiet incentives | 0 | 0 | 0 | 384 | 0 | 0 |
| Actor-only blame incentives | 57 | 26 | 32 | 192 | 77 | 0 |
| Accuracy, briefing disabled | 64 | 32 | 0 | 192 | 96 | 0 |

The responsible individual's first opportunity selected sincere truth in all 32 accuracy games, intentional contradiction in all 32 blame games and all 32 actor-only blame games, and silence in all 32 quiet games. Deception was available in the accuracy games and was not selected. All 96 matched comparisons confirmed identical configurations and initial private states except for the declared incentives, changed actor speech, identical ballots, and identical outcomes. Only the actor's incentive vector changes in the actor-only intervention; other participants' policy-selected actions remain unchanged.

The blame fixture's additional three contradictions were made by the mistaken witness when its belief incorrectly attributed responsibility to itself. The policy uses perceived self-attribution, not the engine's responsibility flag. Twenty-six sincere false witness claims persist with briefing enabled; six are removed when the actor itself is witness 2 and its participation briefing supplies correct self-knowledge. With briefing disabled all 32 mistaken witness claims remain. Intent and factual accuracy are recorded independently; tests also establish that a deliberately contradictory statement may happen to be objectively true.

## Negative outcomes and limits

All games ended without a verdict. Changing speech did not change a single ballot or outcome in the matched comparisons. This is expected under the fixed listener baseline, which does not adopt public claims as evidence. There is no demonstrated successful persuasion or avoidance of actual blame. Silence is voluntary policy selection, but repeated silence after speaking also reflects the explicitly programmed one-message policy budget.

The narrow hypothesis is supported for this specified scoring policy and fixture family: legitimate private incentive changes cause different selected communication actions. This relationship is directly programmed, not evidence of emergent intelligence. Scores are preferences for signals, not calibrated expected consequences; the model's blame-redirection bonus is an assumption. The explicit deceptive control is a scripted positive control, not spontaneous deceptive behavior. No claims about psychology, free will, generality, trust, strategic learning, or optimal behavior follow.

## Validation and historical preservation

Formatting check, Clippy all targets with warnings denied, and the full 15-test integration suite passed. The six original tests were unchanged. New tests cover private briefing delivery and isolation, belief/intent/factual classification, truthful choice when deception is available, recorded utility selection and tie behavior, numeric bounds, invalid actions/configurations, bounded phases, resolver effects, replay, matched interventions, and unchanged Experiment 001 reports. The complete original CLI evaluation is compared byte-for-byte to frozen 001 evidence in the suite. The 002 evaluation is also compared to frozen evidence. The example JSON config matches the CLI and library output, and attempts to overwrite existing reports fail without changing their contents.

SHA-256 of frozen 001 evidence remains `5D3B9ED1B7AE71029BDFA2251FC702E1455C38DB0E22550EC21282A2821E7356`.
SHA-256 of frozen 002 evidence: `08FEC1B1685F8E9D5C064374BDEC25C4AE6F67DA08AE6A88C013A9E16FBB1D32`.

Next questions (proposals only): can a separately preregistered minimal listener rule make speech causally relevant to judgment, and do incentive effects survive alternative defensible signal-utility assumptions? Cross-platform replay and hostile custom-policy side channels remain untested. Advanced belief revision, reputation, long-term memory, learning, natural dialogue, and Experiment 003 were not implemented.
