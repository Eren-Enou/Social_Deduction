# Experiment 002: Voluntary Deception Under Conflicting Incentives

Status: preregistered before implementation/evaluation; implementation and validation are now complete. This document preserves the specified fixture family and predictions; actual findings belong in results.md.

## Question and hypothesis

Can a specified replaceable policy select faithful belief reporting, intentional contradiction, uncertainty, or silence using legitimate local knowledge and incentives? Prediction: for matched seeds, accuracy weights select faithful reporting by the responsible participant, blame-avoidance weights select contradiction, and speech costs select silence. This prediction tests an explicit programmed scoring mechanism, not emergent intelligence or effective persuasion.

## Fixed rules and information

`incident-002-v1`: six individuals, identical seed-to-incident mapping and initial witness assignments to 001, two discussion rounds in ID order, one legal communication per turn, six simultaneous private ballots, four matching votes for a verdict. No changes to 001 source behavior or frozen evidence. Structured uncertainty is a new 002 message with no actor assertion. Rejected actions consume opportunities. Public records contain only speaker, message, and optional prior-message response reference.

The controller privately briefs only the responsible individual with `Participated` when briefing is enabled. This overrides its candidate belief to its own ID without deleting its original witness observation. Others receive no participation briefing. Local beliefs remain fixed after this initial assignment; no dialogue-based revision. Audit records log all assignments and incentives, while policy views contain only the individual's own state and public messages.

## Mechanism specified before implementation

Every participant has nonnegative integer weights: accuracy A, avoid-blame B, contradiction cost D, speech cost C. Weights are local private inputs. For a known candidate b, truth score = A - C - B if b is self (otherwise A - C); contradiction score = -A - D - C + B if b is self (otherwise -A - D - C); silence score = 0. The contradiction candidate is (b+1) mod 6. With no candidate, uncertainty score = A-C and silence score = 0; deliberate contradiction is unavailable because there is no belief to contradict. Scores are i64 to avoid overflow from u32 weights.

These are uncalibrated preferences over communication signals. The B bonus assumes redirecting a self-attribution is valued; it is not a prediction that listeners will be persuaded or that blame will actually be avoided. No rewards, resources, or objective payoff are secretly transferred. Strictly highest score wins; ties favor silence, then truth, then contradiction. A policy may speak once and chooses silence on subsequent turns. Unknown beliefs may produce uncertainty rather than unsupported assertions. Ballots remain sincere-to-belief (or abstention); silence control always abstains.

Controls: truthful reports belief or uncertainty once; silent never speaks; deceptive control deliberately contradicts known beliefs once and otherwise stays silent; incentive-sensitive enumerates available options and records scores. Control policies are explicitly scripted. Intent labels are policy self-reports validated against decision-time belief, never exposed to listeners. Objective falsehood alone is insufficient to classify deception.

Classification: accepted ReportBelief assertion equal to belief is sincere truth when equal to actor, otherwise sincere mistake. Accepted ContradictBelief assertion unequal to known belief is intentional contradiction; factual correctness is recorded separately. Silence and uncertainty are distinct. Contradictory or unsupported intent/action combinations are unsubstantiated, not lies. Invalid actions are rejected and not classified as delivered speech.

## Comparisons and outcomes

Seeds 0–31, preserved without tuning:

- All truthful, all silent, all deceptive controls with briefing and accuracy profile.
- All incentive-sensitive with matched accuracy (A=6,B=0,D=1,C=1), blame (A=1,B=12,D=1,C=1), and quiet (A=0,B=0,D=2,C=1) profiles.
- Targeted intervention: only the actual actor gets blame weights, others retain accuracy weights; assignment is logged as a controller intervention, not a policy's hidden actor input.
- All incentive-sensitive accuracy profile with briefing disabled (negative information control).

256 games; 18 decisions each. Record classification counts, objective false assertions, per-actor first discussion choice, rejected decisions, tallies, verdicts, correctness, and byte-level replay. Pair accuracy vs blame/quiet/actor-only blame reports, checking seed, actor, observations, briefing, policy IDs, and rules; only specified incentives may differ. Assert responsible choices differ where predicted; compare ballots/outcomes to detect unsuccessful persuasion. Silent control must have no public messages or verdicts; truthful fixtures must preserve sincere mistaken-witness assertions when witness 2 is not the actor. Briefing-disabled games must never receive invented participation knowledge.

Information-isolation and illegal-action tests, immutable decision snapshots, deterministic replay, and full frozen 001 report comparisons are required. Do not relax 001 tests. Deception's no-effect outcomes are negative results to preserve. Cross-platform replay, psychology, calibrated utility, learning, trust, and Experiment 003 remain untested/deferred.
