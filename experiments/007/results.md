# Experiment 007 findings

Implemented and evaluated on Windows on 2026-10-09. The preregistration was committed as `e98ca72` before implementation/primary evaluation. The mechanisms, seeds and metrics were not tuned after held-out outcomes. Engineering correctness and scientific usefulness are separate conclusions. Cross-platform agreement remains unverified.

## Evaluation population

The frozen evaluation contains **1792 series,5200 incidents,1792 successful complete canonical replays**,88 aggregate groups and 44 paired comparisons. There are 1536 generated series:64 held-out worlds (seeds 1000–1063), six verification streams, two acceptance policies and two listeners. Another 256 series cover 16 diagnostics, four seeds and both policy factors. Development seeds 0–15 were disjoint. The independent inference units are 64 worlds, not the repeated stream/policy variants or individual ballots. All 44 generated comparisons matched current delivered testimony and current private inputs in 64/64 worlds; no primary action was rejected. The complete report and five section digests are retained for each row; scenario keys regenerate detailed reports.

## Verification consequences and local detection

H1 is demonstrated in declared diagnostic interventions, not naturally discovered verifier incompetence. In `RewardFalse`, seed 0, actual history actor 1 is reported as 2 through a recognized authenticated channel. Observer 5 rewards source 0's sincere false claim: score 0 becomes+1. In the subsequent incident the actor is 5, but the trusted source sincerely names 0; both acceptance policies produce five wrong ballots, one correct ballot and wrong verdict 0. `PenalizeTrue` gives a truthful historical source score-1 under the same authenticated wrong outcome; the next incident also yields a wrong verdict. Factual falsehood does not establish speaker lying; the audit preserves sincere intent separately.

H2 is demonstrated within the declared protocol. Recipients can check recognized issuer/channel, version/session/serial association, retained episode availability, repeated deliveries and incompatible authenticated outcomes. They cannot distinguish a first, internally well-associated authenticated false outcome from an accurate one using the available wire/local inputs. An internally coherent but wrong association, issuer competence or undelivered contradiction likewise has no truth oracle. Missing/selective feedback leaves recipients differently informed; an observer's missing token does not reveal why delivery was absent. Collective verdicts remain unverified.

Both true-first and false-first retained conflicts become sticky quarantine under 007, with that episode contributing zero factual credit after bounded recomputation. Seed 0 `ConflictFalseFirst` changes score-1 to 0 and the final outcome from a wrong verdict to unresolved, with one correct, one wrong and four abstaining ballots. True-first quarantine also removes useful credit: a previously correct verdict becomes unresolved. This is uncertainty management, not recovery of verified truth. The unchanged 006 baseline retains whichever valid outcome arrived first. Duplicate/reissued agreement adds no extra evidence.

In `DelayedContradiction`, both policies initially reach the same wrong verdict. The later contradictory token changes 007's credit+1 to 0; the next incident becomes unresolved, while 006 remains wrong. Earlier records/decisions remain immutable. In `EvictedConflict`, the first episode's detail has been evicted; the later contradiction is Pending with `unverifiable_eviction=true`, leaves score-2 unchanged and never consults the archive. H5's equal-local-input controls pass despite altered hidden actor/private-state/fault labels. These programmed rules are not emergent skepticism.

## Held-out future testimony prediction

Target: the first executed original assertion per declared current source, forecast by one nonsource observer before that incident. Across each generated stream there are 116 scored assertions from 64 worlds;61/116 are factually correct. Forecasts are scored against observer truth only after execution. Ordinal 006 scores are categories, not probabilities. Neutral probability 0.5 has Brier 0.250000, clipped natural-log loss 0.693147 and pooled five-bin ECE 0.025862. This neutral prior was analytic, not fitted to these outcomes.

| Stream / acceptance | Pooled Brier | Pooled log loss | ECE | Ordinal correct / predicted |
|---|---:|---:|---:|---:|
| Clean / either |0.303515|0.821765|0.231896|38/81|
| Authenticated false / either |0.284384|0.777203|0.217385|44/84|
| Unauthenticated or missing / either |0.250000|0.693147|0.025862|0/0 (undefined accuracy)|
| Conflict / unchanged 006 |0.303515|0.821765|0.231896|38/81|
| Conflict / conflict-aware 007 |0.250000|0.693147|0.025862|0/0 (undefined accuracy)|
| Selective / either |0.279260|0.761268|0.130316|4/18|

H3 does not establish an overall predictive advantage or good calibration. Clean world-averaged Brier is 0.299293; difference from neutral+0.049293, world-bootstrap percentile interval[+0.006260,+0.091124]. Authenticated-false difference is+0.045168[+0.006006,+0.083997]; selective+0.025640[+0.005928,+0.048936]. Loss is first averaged within world for these intervals; pooled losses weight all assertions and are intentionally different. These 64-world,512-replicate intervals describe this generator only, with no multiplicity correction or generality claim.

Clean stationary cases show limited positive signal: pooled Brier 0.244187 over 53 assertions, versus 0.353425 over 63 changed-behavior assertions. These conditional cells have no separately preregistered interval and do not establish a robust advantage. Clean history without corrections has Brier 0.291119 (64 assertions); with correction histories 0.318771 (52). Historical credit grades final claims, while the forecast target is the next initial assertion: correct historical repairs can inflate predicted reliability without making new initial observations better. Under false verification, stationary Brier 0.326357 and changed-regime 0.249074 illustrate accidental cancellation, not a reliable learning mechanism.

For clean verification, count strata 0,1,2,3,4+ contain 30,33,21,16,16 assertions with Brier 0.250000,0.282828,0.321428,0.292500,0.434027. More locally verified experience did not monotonically improve prediction. Reliability bins [0,.2),[.2,.4),[.4,.6),[.6,.8),[.8,1] have sample counts 0,14,35,40,27 and factual successes 0,7,23,19,12. The highest bin's mean forecast is 0.817284 while only 12/27=0.444444 are accurate. Empty bins remain undefined. Ordinal correct 38/81, coverage 81/116, does not establish reliable future-accuracy prediction; zero score is unknown. Its separate score bins and conditional diagnostics remain in the evidence. No forecast is used as a ballot weight.

## Decision reliability and coverage

Each cell uses 64 current incidents,384 eligible voters. The neutral 005 listener has the same current outcomes under every stream:172 correct ballots,128 wrong,84 abstentions;24 correct verdicts,11 wrong,29 unresolved. It learns the same local experience but does not use scores in cognition.

| Credibility condition | Correct / wrong / abstaining ballots | Correct / wrong / unresolved verdicts |
|---|---|---|
| Clean, either acceptance |176 /117 /91|26 /12 /26|
| Authenticated false, either |122 /92 /170|12 /2 /50|
| Unauthenticated or missing, either |172 /128 /84|24 /11 /29|
| Conflict, unchanged 006 |176 /117 /91|26 /12 /26|
| Conflict, conflict-aware 007 |172 /128 /84|24 /11 /29|
| Selective, either |173 /126 /85|25 /11 /28|

H4 has mixed effects. Clean weighting changes 61/384 ballots and 11/64 verdicts. All-eligible ballot correctness rises 44.7916% to 45.8333%, while coverage falls 78.1250% to 76.3020% and conditional ballot accuracy rises 57.3333% to 60.0682%. Correct verdict rate rises 37.5% to 40.625%, wrong rate 17.1875% to 18.75% and coverage 54.6875% to 59.375%. Conditional verdict accuracy slightly falls 68.5714% to 68.4210%. Paired correct-verdict-rate difference+3.125 percentage points has interval[-4.6875,+12.5]; the small apparent advantage is not robust.

Authenticated false weighting changes 112 ballots and 27 verdicts. All-eligible correct ballot rate falls to 31.7708%, coverage 55.7291%, conditional ballot accuracy 57.0093%. Correct verdict rate falls 18.75%, wrong rate 3.125%, coverage 21.875%; conditional verdict accuracy rises 85.7142% because many cases become unresolved. Correct-verdict-rate difference from neutral-18.75 percentage points has interval[-28.125,-9.375]; coverage difference-32.8125 points[-46.875,-20.3125]. Fewer incorrect verdicts therefore do not establish improved overall correctness. Across this generator false outcomes usually penalize sources; the diagnostics separately preserve false evidence that rewards a wrong source.

Neutral expected utility per world for wrong costs 1,2,4 is 0.203125,0.031250,-0.312500. Clean weighting is 0.218750,0.031250,-0.343750: slight gain at cost 1, tie at 2, loss at 4. Authenticated-false weighting is 0.156250,0.125000,0.062500: lower coverage loses at cost 1 but avoids costly errors at higher penalties. Paired utility intervals all include zero; these are declared utility sensitivities, not an optimality finding. Conflict quarantine restores neutral outcomes and removes both beneficial and harmful first-accepted influence. Selective verification changes 13 ballots and one verdict, adding one correct verdict with no new wrong verdict; the small effect remains generator-specific.

## Reproducibility and limits

All 001–006 byte-replay contracts pass and original hashes/source/test behavior are preserved. Windows independently regenerated the entire 007 evaluation and manifest on committed implementation `241cd816dcdcc0169a59184b53a17fd4e3dac890`; frozen regression compares both full compact bytes and every canonical manifest field. H6 remains **unverified**: WSL is not installed, Docker is absent, Linux did not run and GitHub CI was not triggered. A synthetic comparator test proves comparator behavior only. Actual execution metadata and digests are in validation.md and the Windows manifest.

Untested: actual Linux/other-architecture replay; live CI; cryptographic authentication; naturally competent/fallible verifiers; arbitrary malicious custom host policies; richer repeated source schedules, adaptive corruption, calibrated model fitting, independent external distributions, alternative memory/checkpoint/predictor rules, truth recovery after eviction, trained social reasoning, global reputation and coalitions. No GUI, language model or Experiment 008 was introduced. The chosen quarantine, credit, weights and forecasting rules are programmed baselines. Scientific findings establish narrow causal failure modes and mixed decision effects, not inherently reliable trust, human cognition or emergent intelligence.
