# Experiment 010 findings

Local, bounded method experience changed estimates and later voluntary choices. The experiment did **not** establish a general predictive advantage over a fixed 0.5 forecast, calibrated reliability, or general voting superiority. Exploration was costly and its post-reversal prediction deteriorated. These negative results remain in the frozen evidence.

## Executed design and interpretation

[Specification](specification.md) was committed before implementation/evaluation. [Evidence](../../evidence/experiment-010.json) contains 3,632 series and 28,864 incidents: 2,560 voluntary held-out treatments over **128 independent worlds**, 720 diagnostic series, 288 executed matched-training series and 64 fixed-current history controls. Every series ran twice with equal complete reports and seven section hashes; an independent Windows execution regenerated the entire evaluation. See [validation](validation.md). No held-out tuning, push or Experiment 011.

Each held-out policy/feedback cell has 128 worlds, 1,024 incidents, 6,144 ballot opportunities and 7,813 initial effort units. There are 64 stationary and 64 shifted worlds. Diagnostic labels are controlled questions, not independent population samples; four labels deliberately share a generator. Training interventions are paid and identified as forced. Primary comparisons use matched worlds; different regime groups are stratified samples, not identical-world stationary/shift counterfactuals.

Intervals below resample whole worlds 512 times with seed 10010, endpoints 12/499. Rates/losses are averaged within world before uncertainty. Undefined conditional rates stay undefined; selected forecast cells report the smaller number of worlds with labels. Pooled Brier scores and forecast counts are descriptive and can differ from world-weighted means. No multiplicity correction or population-general inference is claimed. Additional disagreement, follow-up and illustrative-trace counts below are post hoc descriptive extractions from immutable rows, not policy tuning or new validation seeds.

## Complete voluntary outcomes

Clean feedback, full eight-incident series. C/W/A means correct/wrong/abstaining ballots; C/W/U means correct/wrong/unresolved verdicts.

| Policy | Effort spent | Ballots C/W/A | Verdicts C/W/U |
|---|---:|---:|---:|
| NeverInvestigate | 0 | 3544/1468/1132 | 491/186/347 |
| FirstAccessible | 6876 | 3589/1518/1037 | 539/161/324 |
| CheapestAvailable | 6379 | 3589/1493/1062 | 539/156/329 |
| SeededRandomTarget | 6839 | 3579/1515/1050 | 529/159/336 |
| Frozen009InformationValue | 2613 | 3665/1555/924 | 547/189/288 |
| Frozen009BudgetAware | 2244 | 3656/1548/940 | 544/187/293 |
| AdvertisedQuality | 2613 | 3665/1555/924 | 547/189/288 |
| ExperienceAdjusted | 1847 | 3698/1545/901 | 561/184/279 |
| ConservativeLearner | 2617 | 3661/1564/919 | 548/189/287 |
| ExploratoryLearner | 3865 | 3700/1509/935 | 556/171/297 |

AdvertisedQuality and Frozen009InformationValue have identical selections, beliefs and outcomes by design; their configuration/strategy labels differ. Baselines retain legal experience records and shadow count forecasts for audit, while their effective selection estimates remain the unchanged advertised descriptions. This is a control for using estimates, not a claim that every baseline actively learns reliability.

Against Frozen009InformationValue, ExperienceAdjusted saves 766 units and has 33 more correct ballots, 10 fewer wrong ballots and 14 more correct verdicts. The per-world correct-ballot difference is +0.258 [−0.164,+0.586]; correct-verdict difference +0.109 [−0.031,+0.250]; conditional-accuracy difference +0.00501 [−0.00266,+0.01307]. These do not establish a general judgment advantage. ConservativeLearner's conditional accuracy changes −0.00214 [−0.00428,−0.00018], and incorrect risk +0.00146 [+0.00033,+0.00293], a small adverse contrast among many unadjusted comparisons.

MixedFault on the same latent worlds: Frozen009InformationValue spends 3,429 units, produces 3608/1418/1118 ballots and 546/148/330 verdicts. ExperienceAdjusted spends 2,475, produces 3653/1407/1084 ballots and 562/141/321 verdicts. Its per-world correct-ballot difference +0.352 [−0.156,+0.852] and correct-verdict difference +0.125 [−0.031,+0.273] remain uncertain. Mixed feedback also enters the preserved 007 interpersonal mechanism; complete trajectories include that common mechanism and its interactions, not an isolated method-only feedback treatment. The silent fixed-current controls isolate method-history differences separately.

## Prediction and calibration

Executed matched training also produces both positive and negative assessment results. Each row below uses four seeds, four assessment incidents and 96 ballot opportunities, after identical paid training histories within its fixture. These are controlled descriptive cases, not population estimates.

| Matched fixture | Selector | Assessment effort | Correct ballots | Correct/wrong/unresolved verdicts |
|---|---|---:|---:|---:|
| MisleadingAdvertisement | frozen information value | 96 | 10 | 2/14/0 |
| MisleadingAdvertisement | experience adjusted | 174 | 60 | 12/4/0 |
| MisleadingAdvertisement | conservative | 96 | 10 | 2/14/0 |
| MisleadingAdvertisement | exploratory | 156 | 50 | 10/6/0 |
| ReliabilityReversal | frozen information value | 96 | 30 | 6/10/0 |
| ReliabilityReversal | experience adjusted | 96 | 30 | 6/10/0 |
| ReliabilityReversal | conservative | 114 | 40 | 8/8/0 |
| ReliabilityReversal | exploratory | 120 | 30 | 6/10/0 |
| TrustedMethodFails | frozen information value | 96 | 0 | 0/16/0 |
| TrustedMethodFails | experience adjusted | 96 | 0 | 0/16/0 |
| TrustedMethodFails | conservative | 120 | 5 | 1/15/0 |
| TrustedMethodFails | exploratory | 120 | 5 | 1/15/0 |

Matched ResourceScarcity exhausts the available effort during training: all four selectors spend zero assessment effort, have 16 correct/16 wrong/64 abstaining ballots and 16 unresolved verdicts. Matched MissingFeedback and ContradictoryVerification both give frozen/experience/conservative selectors 75 correct assessment ballots and 15 correct/1 wrong verdict; exploration spends 120 rather than 96 units and produces 60 correct ballots and 12 correct/4 wrong verdicts. Null and adverse controls are retained alongside the misleading-advertisement benefit. Across all 18 fixtures, tests require equality of actual training deliveries, method memories, interpersonal memories and testimony across the four selectors.

Clean feedback, assessment episodes 4-7, **all accessible** alternatives. Accuracy labels exclude missing results. Pooled conditional-accuracy Brier is shown; each interval is the world-paired loss difference from neutral 0.5, so negative favors the model.

| Effective model | Regime | Accuracy labels | Brier | Difference from neutral, interval |
|---|---|---:|---:|---:|
| ExperienceAdjusted | stationary | 1773 | 0.245365 | −0.003654 [−0.010658,+0.003648] |
| ExperienceAdjusted | shifted | 1669 | 0.255922 | +0.005867 [−0.002138,+0.012298] |
| ConservativeLearner | stationary | 1773 | 0.354569 | +0.093620 [+0.062595,+0.127638] |
| ConservativeLearner | shifted | 1669 | 0.348303 | +0.088980 [+0.051735,+0.133961] |
| ExploratoryLearner | stationary | 1773 | 0.243245 | −0.006268 [−0.014869,+0.001541] |
| ExploratoryLearner | shifted | 1669 | 0.265107 | +0.014851 [+0.005841,+0.023341] |

ExperienceAdjusted beats advertisements in both regimes: paired differences −0.096833 [−0.130009,−0.065368] stationary and −0.085272 [−0.133309,−0.046675] shifted. This does not establish a benefit over neutral. During clean assessment, 3,556/4,568 accessible forecasts have zero graded experience; only 163 have three or more. Neutral priors account for much of the advertisement advantage. ConservativeLearner adopts count accuracy in only 129/4,568 forecasts and missingness in 279; its threshold usually preserves misleading claims.

Across both regimes, ExperienceAdjusted's clean assessment accuracy Brier is 0.250484 (3,442 labels), versus neutral 0.250000, advertised 0.351685 and observer-process 0.190187. Its missingness Brier is 0.244027 (4,568 labels), versus neutral 0.250000, advertised 0.231665 and observer-process 0.135822. Missing labels are absent accuracy events, not failures. Observed process parameters are a research benchmark, never agent inputs.

Selected-only accuracy has 395 labels in 46 defined worlds: pooled model Brier 0.254960, neutral 0.250000 and advertised 0.312974. The world-weighted model-minus-neutral difference is −0.010777 [−0.035082,+0.013471]. Uninspected alternatives have 3,047 labels in 128 worlds: model 0.249904 versus neutral 0.250000 and advertised 0.356703, paired difference +0.003547 [−0.002276,+0.009461]. Selected success cannot stand for all available methods. Paid selected missingness uses 543 results, whereas uninspected missingness uses 4,025.

Calibration is not established. ExperienceAdjusted's accuracy bin discrepancy (ECE, pooled descriptive measure) is 0.026910 stationary and 0.047693 shifted. Its stationary 0.8-1 bin contains only 14 forecasts: mean prediction about 0.807, observed correctness 7/14. ConservativeLearner's stationary top bin has 441 forecasts at probability 1, but only 210 correct. ExploratoryLearner's shifted ECE is 0.100921. Correlated forecasts within world are not independent calibration trials, and no calibration-equivalence tolerance was preregistered.

## Answers to the twenty research questions

1. **Legitimate recurring-method experience?** Yes. Across all declared populations, 94,693 direct deliveries include 2,597 recognized duplicates: 92,096 eligible personal observations (81,042 nonmissing, 11,054 missing). There are 66,825 received indirect records, which do not become personal accuracy samples. Forced/injected records remain separately labeled; these totals are not voluntary-world population estimates.

2. **Changed estimates?** Yes, mechanically. There are 67,360 feedback effects changing count summaries, 64,519 final retained/checkpoint graded contributions (33,712 accepted-success and 30,807 accepted-failure labels), and 3,373 checkpoint graded contributions after eviction. Accepted-success means agreement with accepted verification, not observer-certified truth. MissingFeedback seed 0/ExperienceAdjusted obtains 48 nonmissing results but zero graded contributions; accuracy remains 0.5 while availability can be learned.

3. **Different individual assessments?** Yes. In clean ExperienceAdjusted histories, 750/2,734 shared family/episode opportunities have differing individual learned/effective assessments. Exploration has 1,238/2,734; ConservativeLearner has 942 differing count forecasts but only 180 differing effective assessments because its threshold masks sparse histories. These are descriptive within-world opportunities, not independent samples. SelectiveExposure executes different accessible training subsets for individuals, then exposes a common assessment catalog.

4. **Changed later choices?** Yes in controlled conditions. Each fixed-current cause (actual acquisition history, or identical acquisition history with clean/false feedback) has four paired seeds and 24 individual assessment opportunities per selector. ExperienceAdjusted and ExploratoryLearner each change 24/24 targets under each history cause; frozen and conservative selectors change 0/24. Current evidence, catalog, belief and budget are identical. Conservative histories have only two samples per family, below its three-sample threshold. In voluntary clean ExperienceAdjusted series, 683/1,007 opportunities following changed feedback choose a different target/decline status, and 324 retain it; incident, access and budget also change there, so those follow-ups are not isolated causal effects.

5. **Better sources than frozen baselines?** No general superiority established. Clean selected results are 582 correct/504 wrong/323 missing for ExperienceAdjusted, versus 588/551/411 for frozen information value; conditional realized correctness is about 0.536 versus 0.516, with different exposure counts. Observer process accuracy averaged over selected sources is 0.488112 versus 0.486129, while CheapestAvailable is 0.503054 and exploration 0.505159. These process averages are descriptive, not a paired significance claim or evidence of optimal selection. Forecast quality, sampling and judgment must be separated.

6. **Better future accuracy prediction?** Better than poorly advertised quality, not generally better than neutral. The table records the null stationary-neutral contrasts and the adverse shifted exploration contrast. Under MixedFault, ExperienceAdjusted's assessment Brier is 0.252059 (3,442 labels), paired neutral difference +0.001126 [−0.001530,+0.003852]; 3,907/4,568 forecasts have no graded history. No positive general learning claim follows.

7. **Calibrated probabilities?** Unsupported. The formula produces smoothed frequencies; the bins above show errors, sparse high-confidence cells and severe advertisement-driven miscalibration. They are not psychological confidence or validated Bayesian posteriors for this nonstationary, selected, fallibly labeled process.

8. **More correct ballots/verdicts?** Some point gains, no general demonstrated advantage. Clean experience adjustment adds 33 correct ballots and 14 correct verdicts against information value, but their paired intervals include zero. Exploration adds 35 correct ballots and 9 correct verdicts, with intervals [−0.289,+0.719] and [−0.070,+0.203] per world. ConservativeLearner has adverse accuracy/risk contrasts. All seven baselines, both feedback streams, regime strata and windows remain in the 252 frozen comparisons.

9. **Worth resource cost?** Depends on valuation. With wrong-verdict cost 1, ExperienceAdjusted-minus-information-value utility per world is +0.148 [−0.008,+0.297] at effort price 0; +1.645 [+1.172,+2.211] at 0.25; +6.133 [+4.336,+8.242] at 1. Savings support the priced-effort contrast, without proving better reasoning. Exploration is +0.211 [+0.031,+0.375] at zero effort price but −2.234 [−2.795,−1.635] at 0.25 and −9.570 [−11.563,−7.297] at 1. The frozen report preserves all nine cost/price combinations per comparison.

10. **Failed/wasted exploration?** Across the declared grid, 3,608 voluntary exploration actions cost 6,800 units; 1,604 return wrong findings, 619 missing findings and 631 no belief change. These categories overlap. Clean held-out exploration has 1,217 actions/2,360 units, 427 wrong and 308 missing/no-change results; MixedFault has 1,191 actions, 418 wrong and 311 missing/no-change results. Forced training is excluded. Exploration sometimes equals the exploitation proposal; its recorded spending is actual cost, not automatically incremental cost.

11. **Selective-exposure bias?** Exposure matters. The executed SelectiveExposure fixture gives different training subsets, and independent private counts differ when the assessment catalog becomes common. Selected-only and uninspected losses/denominators above differ. Matched training holds actual delivery/feedback histories constant across four selectors for 18 fixtures/four seeds. This isolates policy comparison from self-selected training in those controls; it does not remove selection bias from voluntary trajectories.

12. **Misleading advertisements influential?** Yes. ConservativeLearner continues using advertisement for accuracy in 4,439/4,568 clean assessment forecasts. Its effective Brier remains close to advertisement and far above neutral, including the top-bin failure above. The zero-history neutral policy and threshold policy are programmed prior choices; their differences cannot all be attributed to learned experience.

13. **After quality changed?** Sealed future outputs use the new process while past observations and claims remain unchanged. Assessment loss for exploration rises from pooled 0.243245 stationary to 0.265107 shifted, and its shifted-neutral interval is adverse. These are stratified world samples, not a same-world shift/no-shift causal estimate. Cumulative counts retain older successes and availability experience, so reversal can invalidate previously useful expectations.

14. **Could preferences reverse?** Mechanically yes, not reliably within this horizon. The executed engineering control's one accepted success followed by five accepted failures causes a later voluntary switch; absent feedback does not. Conversely, TrustedMethodFails seed 0/ExperienceAdjusted selects the favored method in all eight incidents: four initially correct then four wrong, producing wrong verdicts in all four assessment incidents. Before incident 4 it has accuracy 5/6; failures reduce this without sufficient incentive to switch from the cheap, known nonmissing source to an under-observed alternative. Exploration samples the alternative in incidents 3 and 7 in that seed, and all 12 such individual exploration results are wrong, costing 24 units. Adaptation is a declared count rule, not a guaranteed recovery narrative.

15. **False authenticated verification corrupts learning?** Yes. AuthenticatedFalse seed 0/ExperienceAdjusted receives 48 factually false accepted tokens and ends with 2 accepted-success/46 accepted-failure contributions despite actual good results in several episodes. In the fixed feedback controls, identical actual training acquisitions plus false actor claims change 24/24 adaptive assessment targets. Across the whole grid 11,850 delivered feedback items are factually false; 72,145 acceptances, 60,059 pending effects and 2,496 quarantine effects are recorded. Acceptance is not truth. Faulty feedback can also alter the separate preserved interpersonal mechanism.

16. **Provenance prevents duplicated learning?** For visible dependence, yes under the declared first-owner rule. KnownDependent seed 0/ExperienceAdjusted has 88 direct deliveries, 40 duplicates and 48 graded contributions. HiddenDependent has 96 deliveries, zero recognized duplicates and 96 graded contributions, although the observer knows shared ancestry. The learner receives no privileged correction. This contrast includes differing voluntary selection caused by visible overlap; it is not an identical-delivery estimate. Duplicate verification/id tests separately establish idempotence. Quarantine changes retained accuracy contributions; missingness and evicted compact consequences persist.

17. **Earlier experiments preserved?** Yes. All thirteen accepted frozen 001-009 artifacts retain their byte lengths/SHA-256, all old engines/tests/dependencies/toolchain are unchanged, and their complete regression suite passed. See the integrity table and command record in validation.

18. **Cross-platform replay?** Windows only. The independent actual Windows evaluation/metric/full-section hashes agree with the primary and an independent Python canonical reference. Linux/Windows agreement remains unverified because WSL is not installed, Docker is unavailable and no CI was triggered. The prepared workflow is not an executed result.

19. **Git commits?** `1f73c50` preregistration; `071e74b` core learning/selection; `0cf42d3` predictive evaluation/tests/CLI; `6315247` output buffering and exact-byte/create-once CLI regression. Both actual primary executions used `0cf42d3`; buffering followed the completed simulations and changes neither policies nor JSON values. Frozen-data/regression commit is `50b2db8`; the final findings commit is recorded in the completion response. No history rewrite or push.

20. **Unsupported capabilities/claims?** General intelligence, human-like trust, intentional deception in this experiment, consciousness, optimal information economics, calibrated probability, global reputation, social agreement on method ratings, general reinforcement learning, nonstationary recovery guarantees, arbitrary-host isolation, cryptographic verification, cross-platform agreement and external generality remain unsupported. Learned method estimates do not directly change 008 evidence weights. Direct/indirect experience, personal testimony credibility, accepted claims and objective truth remain separate. Ordinary pure instrument reports do not automatically establish reporter honesty.

## Audit, overhead and reproduction

Full reports contain objective history and captured individual views/effects. Compact evidence retains predecision expectations, alternative scores, private belief/balance, input hashes, deliveries, feedback/count changes, evictions, communications and ballots, plus complete-report and section digests. Regenerate full views on the committed source and require matching original hashes; do not explain an old decision using a changed policy. Individuals cannot consult the observer archive to reconstruct forgotten cognition.

Maximum observed serialized full report is 6,655,663 bytes; maximum final interpersonal-memory serialization 42,039 bytes. Across rows, final method-memory serialization totals 69,410,041 bytes. There are 4,748 direct record evictions and 3,373 checkpoint graded consequences. The primary grid has zero unverifiable-eviction feedback events; separate engineering tests exercise late/evicted references. Storage/counter measures are proxies, not controlled heap or wall-clock benchmarks. JSON writing initially issued many small file writes; the tested buffer preserves exact serialization and create-once protection.

```powershell
cargo run --locked --release --bin experiment010 -- scenario generated Clean 4000 ExperienceAdjusted new-config.json
cargo run --locked --release --bin experiment010 -- run new-config.json new-full-report.json
cargo run --locked --release --bin experiment010 -- scenario matched TrustedMethodFails 0 ExperienceAdjusted new-matched.json
cargo run --locked --release --bin experiment010 -- fixed feedback 0 ExperienceAdjusted 1 new-fixed.json
cargo run --locked --release --bin experiment010 -- evaluate new-evidence.json
cargo run --locked --release --bin experiment010 -- replay new-windows.json
```

Diagnostic rows use `scenario diagnostic Family seed Policy`; matched rows use `scenario matched`. Fixed rows use their cause, seed, policy and history suffix 0/1. Existing files are refused. For forensic historical reproduction, use source `0cf42d3`; `6315247` changes file buffering only. Current-source frozen regression requires identical full evaluation values, raw bytes, canonical metrics and all full-section hashes.
