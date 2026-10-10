# Experiment 005 findings

Completed 2026-10-09. Mechanisms, weights, hypotheses, fixtures and seeds were specified before implementation and primary evaluation in specification.md. Frozen `evidence/experiment-005.json` contains 576 games, all with byte-identical replays and zero rejected selected decisions. No primary weights or rules were tuned after outcomes were inspected. This is fixed programmed behavior, not spontaneous remorse, skepticism or social intelligence.

## Primary outcomes

Each row contains 16 games per listener. C/W/N means correct/wrong/no verdict. Recovery and uncertainty refer to recipient 5, comparing its last round-one belief with judgment. Recovery requires an earlier wrong candidate and a final correct candidate; uncertainty is a separate outcome. MissingOriginal starts unknown and therefore does not count as becoming uncertain.

| Fixture | Frozen004 C/W/N | Revision C/W/N | Revision recoveries | Revision became uncertain |
|---|---|---|---:|---:|
| Retain | 0/16/0 | 0/16/0 | 0 | 0 |
| Sincere | 0/16/0 | 16/0/0 | 16 | 0 |
| Retraction | 0/16/0 | 0/0/16 | 0 | 16 |
| Misleading | 0/16/0 | 0/16/0 | 0 | 0 |
| WrongSincere | 0/16/0 | 0/16/0 | 0 | 0 |
| SelectiveLow | 0/16/0 | 16/0/0 | 16 | 0 |
| SelectiveHigh | 0/16/0 | 0/16/0 | 0 | 0 |
| RelayBoth | 0/16/0 | 16/0/0 | 16 | 0 |
| RelayIntermediate | 0/16/0 | 0/16/0 | 0 | 0 |
| RelayFinal | 0/16/0 | 0/0/16 | 0 | 16 |
| RelayNeither | 0/16/0 | 0/16/0 | 0 | 0 |
| RelayWithheld | 0/16/0 | 0/16/0 | 0 | 0 |
| MissingOriginal | 0/0/16 | 0/0/16 | 0 | 0 |
| Independent | 0/0/16 | 0/0/16 | 0 | 16 |
| Injected | 0/16/0 | 16/0/0 | 16 | 0 |
| Branches | 0/16/0 | 0/0/16 | 0 | 16 |
| Sequential | 0/16/0 | 0/16/0 | 0 | 0 |
| Dispute | 0/16/0 | 0/0/16 | 0 | 16 |

Totals across the declared family: Frozen004 had 0 correct, 256 wrong and 32 unresolved games; Revision had 64 correct, 128 wrong and 96 unresolved games. These totals are fixture-weighted software comparisons, not estimates of real-world accuracy. Revision produced 64 recipient-5 recoveries, including 16 from injected testimony; only 48 were in policy-selected correction fixtures.

All 288 within-fixture comparisons changed only listener configuration. Delivered messages/routes matched in 256 pairs. RelayBoth and RelayIntermediate each changed voluntary relay communication in all 16 pairs, so their outcomes include a communication response to listener behavior rather than a fixed-message intervention. Eleven fixtures changed recipient-5 votes and verdict IDs in every seed (176 pairs). Changes in Misleading, WrongSincere and Sequential were to a different wrong candidate, not successful recovery.

## Private change, disclosure and listener response

Retain and SelectiveHigh changed source0's private belief to the true candidate in all runs but selected no correction. Their 32 matched recipient comparisons (both listeners) had identical inputs, assessments and votes. A further engineering perturbation changes the source's undisclosed later observation while preserving recipient behavior. Private cognition did not silently update the audience.

SelectiveLow and SelectiveHigh differ only in source0's revision cost. Benefit 2/cost 1 yields revision score +1; benefit 2/cost 3 yields -1 and silence. Both recognize the changed belief in all 16 games per listener. Low selects 16 corrections; high withholds at 16 legal opportunities. Tie-silence and large numeric cost limits are unit-tested. This is an explicitly coded signal preference, not learned honesty or calibrated expected utility.

Sincere communicates 16 repairs of earlier mistakes per listener. Frozen004 ignores revisions and still reaches 16 wrong verdicts. Revision replaces the old contribution within the same source group, producing 16 correct votes and verdicts. Retraction instead supplies no alternative, removes that speaker's support, and produces 16 recipient abstentions and unresolved games under Revision. Neither process rewrites the original assertion, original observation or earlier belief snapshot.

## Relay limits and failed propagation

RelayBoth's revision listener lets relay1 adopt the source correction and select a faithful correction of its own relay in all 16 games. Final recipients receive both revised messages, retain one provenance contribution and recover. Frozen004 leaves relay1 believing the original assertion, so it withholds the new replacement despite receiving it. This fixture demonstrates a change in subsequent selected communication as well as voting.

RelayIntermediate also selects 16 faithful forwarded corrections under Revision, but final recipients never receive the referenced source correction. The listener marks the forwarded revision InvalidRevision (unverifiable forwarded reference), leaves both old assertions standing and reaches 16 wrong verdicts. No archive lookup supplies the missing source message. Thus successful transmission alone does not guarantee acceptance or recovery. The initial engineering test expected a contested group; that expectation was corrected to this preregistered reference requirement, without changing the listener.

RelayFinal delivers the source correction to final recipients but not relay1. Recipients retain the corrected source claim and the stale relay within one perceived group; conflicting candidates make that group contribute zero. They abstain and all 16 games remain unresolved. Source0's correction has no authority to revise relay1's claim. RelayNeither preserves wrong beliefs when the correction reaches neither relay nor final recipients. RelayWithheld delivers it to relay1, but cost suppresses forwarding; final recipients still reach wrong verdicts. MissingOriginal supplies a correction without its target to recipients 2–5; they stay unknown rather than reconstructing the original.

## Accuracy, intent and contested evidence

WrongSincere has 16 sincerely selected but still-wrong revisions per listener. Misleading has 16 explicitly belief-contrary corrections per listener. Both deliver the same candidate replacement and both generate wrong verdicts under Revision, while retaining distinct private evidence and observer intent/classification. Engineering checks establish identical recipient inputs/assessments/votes despite these hidden differences. False correction content alone cannot reveal intent.

An additional engineering-only adverse fixture starts with a correct original assertion. The misleading control, while retaining a correct private belief, changes its communicated candidate contrary to that belief. With identical wire messages, Frozen004 preserves a correct verdict and Revision reaches a wrong verdict. This case was added after the primary freeze, changes no primary data, and demonstrates that revision handling can worsen accuracy; it is not an independent preregistered success claim.

False corrections also propagate as FaithfulCorrection in a separate engineering relay fixture. Original deception and received exact copying remain different classifications. No old sincere mistake is retrospectively relabeled as intentional deception when a speaker repairs it.

Branches provides two controller-injected corrections of the same initial message. Revision retains two competing leaves, disputes the branch and abstains; newest-message priority is not used. Sequential instead explicitly targets the first correction with a second revision: its final replacement is accepted within one group, yielding 16 wrong verdicts. In both fixtures the second injected revision has unspecified intent and is Unclassifiable, rather than inventing a sincere or deceptive explanation.

Independent supplies a separate new source alongside the unrevised old source, giving two competing contributions and uncertainty. Sincere replaces one source's contribution and recovers using one group. Dispute supplies a separate report against the corrected source; Revision retains both independent groups, ties and abstains. Ordinary disagreement cannot retract another speaker's statement.

Sincere and Injected have identical wire/delivery streams and recipient inputs/assessments/votes in all 32 comparisons. Injected selects zero communication; its revision is explicitly an intervention. Matching listener effects do not turn researcher-controlled testimony into agent initiative. The 384 mechanism comparisons retain lists of changed configuration fields so broader policy, observation and routing interventions are not mislabeled single-factor testimony controls.

## Communication and storage measurements

Counts below are per game. Delivered message copies also equal final received-memory copies; sent-memory copies are retained separately. Serialized bytes are storage/communication proxies, not RAM allocations or runtime benchmarks.

| Fixture/listener | Received copies | Sent copies | Delivered Message bytes | Audit lineage bytes |
|---|---:|---:|---:|---:|
| Retain/Revision | 6 | 1 | 396 | 98 |
| Sincere/Revision | 12 | 2 | 1068 | 193 |
| Retraction/Revision | 12 | 2 | 1086 | 196 |
| RelayBoth/Frozen004 | 16 | 3 | 1348 | 306 |
| RelayBoth/Revision | 20 | 4 | 1784 | 415 |
| RelayIntermediate/Revision | 16 | 4 | 1336 | 415 |
| RelayFinal/Revision | 15 | 3 | 1236 | 306 |
| MissingOriginal/Revision | 8 | 2 | 804 | 193 |
| Branches/Revision | 18 | 3 | 1740 | 290 |

Direct correction adds six received copies, one sent copy, 672 serialized delivered bytes and 95 lineage bytes relative to Retain. Revision-aware RelayBoth adds four delivered copies and 436 delivered bytes relative to its frozen-listener counterpart because the relay chooses to communicate. Compact report bytes across 16 Sincere games were 615,590 (Frozen004) versus 625,126 (Revision), reflecting interpretations/snapshots as well as messages. The entire pretty-printed frozen evaluation is 72,225,029 bytes. No timing, heap or cross-platform benchmark was executed.

## Limits and remaining questions

Latest declared private observation supersedes the private anchor by design; it is not automatically reliable. Both listener variants share that 005 observation rule, while the frozen testimony implementation itself remains unchanged. Source/group claims remain unverified. First ordinary assertion rules still apply; valid revisions retain the initial group's identity and do not repair malformed original provenance.

Two individual opportunities limit selected correction sequences to one initial assertion plus one revision. More complex conflicting sequences here are explicit injections, not emergent behavior. Branch standing and group conflict suppression are assumptions; requiring recipients to see forwarded source corrections can reject useful information. Inference lineage records declared basis and contributing inputs, not a complete causal model of arbitrary custom code. Memory retains finite game history without evictions.

Broader speaking-order/source structures, probabilities, reputation, learning, long-term memory, complex strategy, psychological realism and hostile host-side policies remain untested. The 16 seeds vary incident identity, not human behavior. Future mechanisms need separate authorization, preregistration and new immutable evidence. No Experiment 006 has begun.
