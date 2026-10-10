use super::*;
use crate::Phase;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fixture {
    Independent,
    KnownShared,
    HiddenShared,
    UnspecifiedShared,
    OpaqueShared,
    Chain,
    WrongAttribution,
    CollapsedIndependent,
    MistakenOrigin,
    DeceptiveOrigin,
    NoMessage,
    Disagreement,
    InjectedShared,
}
pub const FIXTURES: [Fixture; 13] = [
    Fixture::Independent,
    Fixture::KnownShared,
    Fixture::HiddenShared,
    Fixture::UnspecifiedShared,
    Fixture::OpaqueShared,
    Fixture::Chain,
    Fixture::WrongAttribution,
    Fixture::CollapsedIndependent,
    Fixture::MistakenOrigin,
    Fixture::DeceptiveOrigin,
    Fixture::NoMessage,
    Fixture::Disagreement,
    Fixture::InjectedShared,
];
pub fn fixture_config(fixture: Fixture, seed: u64, listener: ListenerKind) -> Config {
    let mut config = Config::new(seed);
    let actor = config.actor();
    let wrong = (actor + 1) % 6;
    let false_origin = matches!(fixture, Fixture::MistakenOrigin | Fixture::DeceptiveOrigin);
    let (p, q) = if false_origin {
        (wrong, actor)
    } else {
        (actor, wrong)
    };
    config.listeners = [listener; 6];
    config.observations = [Some(p), None, None, Some(q), Some(q), Some(q)];
    config.speakers[0] = SpeakerKind::baseline(PolicyKind::Truthful);
    config.speakers[1] = SpeakerKind::Relay {
        source: 0,
        presentation: Presentation::Disclose,
    };
    config.speakers[2] = SpeakerKind::Relay {
        source: 0,
        presentation: Presentation::Disclose,
    };
    config.routes[1] = vec![3, 4, 5];
    config.routes[2] = vec![3, 4, 5];
    match fixture {
        Fixture::Independent | Fixture::CollapsedIndependent => {
            config.observations[1] = Some(p);
            config.observations[2] = Some(p);
            config.routes[1] = (0..6).collect();
            config.routes[2] = (0..6).collect();
            for id in [1, 2] {
                config.speakers[id] = SpeakerKind::Baseline {
                    kind: PolicyKind::Truthful,
                    attribution: if fixture == Fixture::CollapsedIndependent {
                        Some(Attribution::NamedSource(0))
                    } else {
                        None
                    },
                };
            }
        }
        Fixture::HiddenShared => {
            for id in [1, 2] {
                config.speakers[id] = SpeakerKind::Relay {
                    source: 0,
                    presentation: Presentation::Firsthand,
                };
            }
        }
        Fixture::UnspecifiedShared => {
            for id in [1, 2] {
                config.speakers[id] = SpeakerKind::Relay {
                    source: 0,
                    presentation: Presentation::Unspecified,
                };
            }
        }
        Fixture::OpaqueShared => config.routes[0] = vec![0, 1, 2],
        Fixture::Chain => {
            config.routes[1] = vec![2, 3, 4, 5];
            config.speakers[2] = SpeakerKind::Relay {
                source: 1,
                presentation: Presentation::Disclose,
            };
        }
        Fixture::WrongAttribution => {
            for id in [1, 2] {
                config.speakers[id] = SpeakerKind::Relay {
                    source: 0,
                    presentation: Presentation::Named(3),
                };
            }
        }
        Fixture::MistakenOrigin | Fixture::KnownShared => {}
        Fixture::DeceptiveOrigin => {
            config.observations[0] = Some(q);
            config.speakers[0] = SpeakerKind::baseline(PolicyKind::DeceptiveControl);
        }
        Fixture::NoMessage => config.speakers[0] = SpeakerKind::baseline(PolicyKind::Silent),
        Fixture::Disagreement => config.observations[1] = Some(q),
        Fixture::InjectedShared => {
            config.speakers = [SpeakerKind::baseline(PolicyKind::Silent); 6];
            for speaker in 0..3 {
                config.injections.push(Injection {
                    speaker,
                    choice: Choice {
                        action: Action::Speak {
                            speech: Speech::Assert(p),
                            attribution: if speaker == 0 {
                                Attribution::Firsthand
                            } else {
                                Attribution::Reference(0)
                            },
                            basis: if speaker == 0 {
                                Basis::PrivateObservation
                            } else {
                                Basis::Received(0)
                            },
                            recipients: config.routes[usize::from(speaker)].clone(),
                        },
                        strategy: if speaker == 0 {
                            Strategy::Baseline(Intent::ReportBelief)
                        } else {
                            Strategy::RelayReceived
                        },
                        scores: vec![],
                        mechanism: "explicit controller intervention, not a selected relay".into(),
                    },
                });
            }
        }
    }
    config
}
pub fn judgment(report: &Report, id: u8) -> &Decision {
    report
        .decisions
        .iter()
        .find(|d| d.input.id == id && d.input.phase == Phase::Judgment)
        .unwrap()
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub selected_messages: usize,
    pub injected_messages: usize,
    pub selected_relays: usize,
    pub injected_relays: usize,
    pub speaker_count_sum: usize,
    pub objective_chain_count_sum: usize,
    pub target_received_speakers_sum: usize,
    pub target_actual_chains_sum: usize,
    pub target_perceived_groups_sum: usize,
    pub target_accepted_sum: usize,
    pub target_discounted_sum: usize,
    pub target_ignored_sum: usize,
    pub target_uncertain_sum: usize,
    pub target_visible_link_sum: usize,
    pub target_missing_reference_sum: usize,
    pub target_scores_sum: [u64; 6],
    pub candidate_changes: usize,
    pub score_changes: usize,
    pub rejected_decisions: usize,
    pub abstentions: usize,
    pub false_assertions: usize,
    pub sincere_mistakes: usize,
    pub original_contradictions: usize,
    pub faithful_false_relays: usize,
    pub mismatched_attributions: usize,
    pub correct_verdicts: usize,
    pub incorrect_verdicts: usize,
    pub no_verdicts: usize,
    pub max_depth: u8,
    pub deliveries: usize,
    pub retained_message_copies: usize,
    pub attribution_bytes: usize,
    pub causal_lineage_bytes: usize,
    pub delivered_message_bytes: usize,
    pub report_bytes: usize,
}
impl Counts {
    fn add(&mut self, r: &Report) -> Result<(), String> {
        let d = judgment(r, 5);
        let u = &r.updates[d.update];
        self.speaker_count_sum += r
            .testimony
            .iter()
            .map(|t| t.message.speaker)
            .collect::<BTreeSet<_>>()
            .len();
        self.objective_chain_count_sum += r
            .testimony
            .iter()
            .flat_map(|t| t.lineage.roots.iter().copied())
            .collect::<BTreeSet<_>>()
            .len();
        self.target_received_speakers_sum += u
            .input
            .messages
            .iter()
            .filter(|m| m.speaker != 5 && matches!(m.speech, Speech::Assert(_)))
            .map(|m| m.speaker)
            .collect::<BTreeSet<_>>()
            .len();
        self.target_actual_chains_sum += u.observer_received_roots.len();
        self.target_perceived_groups_sum += u.assessment.perceived_groups.len();
        for source in &u.assessment.sources {
            match source.treatment {
                Treatment::Accepted => self.target_accepted_sum += 1,
                Treatment::DuplicateDiscount => self.target_discounted_sum += 1,
                Treatment::UncertainProvenance => self.target_uncertain_sum += 1,
                _ => self.target_ignored_sum += 1,
            }
            self.target_visible_link_sum +=
                usize::from(source.verification == Verification::VisibleLink);
            self.target_missing_reference_sum +=
                usize::from(source.verification == Verification::MissingReference);
        }
        for (sum, &score) in self
            .target_scores_sum
            .iter_mut()
            .zip(&d.input.current.scores)
        {
            *sum += u64::from(score);
        }
        for update in &r.updates {
            self.candidate_changes +=
                usize::from(update.input.prior.candidate != update.assessment.after.candidate);
            self.score_changes +=
                usize::from(update.input.prior.scores != update.assessment.after.scores);
        }
        for decision in &r.decisions {
            self.rejected_decisions += usize::from(decision.rejection.is_some());
            self.abstentions += usize::from(
                decision.rejection.is_none() && decision.choice.action == Action::Vote(None),
            );
        }
        for t in &r.testimony {
            let relay = matches!(t.lineage.basis, Basis::Received(_));
            match t.origin {
                Origin::Selected { .. } => {
                    self.selected_messages += 1;
                    self.selected_relays += usize::from(relay)
                }
                Origin::Injected { .. } => {
                    self.injected_messages += 1;
                    self.injected_relays += usize::from(relay)
                }
            }
            self.false_assertions += usize::from(t.matches_truth == Some(false));
            self.sincere_mistakes +=
                usize::from(t.classification == CommunicationClass::SincereMistake);
            self.original_contradictions +=
                usize::from(t.classification == CommunicationClass::IntentionalContradiction);
            self.faithful_false_relays += usize::from(
                t.classification == CommunicationClass::FaithfulRelay
                    && t.matches_truth == Some(false),
            );
            self.mismatched_attributions += usize::from(t.attribution_compatible == Some(false));
            self.max_depth = self.max_depth.max(t.lineage.depth);
            self.attribution_bytes += serde_json::to_vec(&t.message.attribution)
                .map_err(|e| e.to_string())?
                .len();
            self.causal_lineage_bytes += serde_json::to_vec(&t.lineage)
                .map_err(|e| e.to_string())?
                .len();
            self.delivered_message_bytes += serde_json::to_vec(&t.message)
                .map_err(|e| e.to_string())?
                .len()
                * t.recipients.len();
        }
        self.deliveries += r
            .objective_history
            .iter()
            .filter(|e| matches!(e, Event::Delivered { .. }))
            .count();
        self.retained_message_copies += r.retained_message_copies;
        self.report_bytes += serde_json::to_vec(r).map_err(|e| e.to_string())?.len();
        match r.correct {
            Some(true) => self.correct_verdicts += 1,
            Some(false) => self.incorrect_verdicts += 1,
            None => self.no_verdicts += 1,
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureEvidence {
    pub fixture: Fixture,
    pub listener: ListenerKind,
    pub counts: Counts,
    pub reports: Vec<Report>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListenerPair {
    pub fixture: Fixture,
    pub seed: u64,
    pub only_listeners_differ: bool,
    pub identical_delivered_messages: bool,
    pub target_actual_chains: usize,
    pub baseline_groups: usize,
    pub provenance_groups: usize,
    pub repeated_contributions_avoided: usize,
    pub target_scores_changed: bool,
    pub target_candidate_changed: bool,
    pub target_vote_changed: bool,
    pub verdict_changed: bool,
    pub choices_changed: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HiddenOriginControl {
    pub seed: u64,
    pub listener: ListenerKind,
    pub identical_target_update_inputs: bool,
    pub identical_target_assessments: bool,
    pub identical_target_votes: bool,
    pub independent_actual_chains: usize,
    pub hidden_actual_chains: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionPair {
    pub control: Fixture,
    pub treatment: Fixture,
    pub seed: u64,
    pub matched_initial_states: bool,
    pub same_speakers_and_propositions: bool,
    pub actual_roots_unchanged: bool,
    pub target_groups_changed: bool,
    pub target_vote_changed: bool,
    pub verdict_changed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub runs: usize,
    pub replay_matches: usize,
    pub fixtures: Vec<FixtureEvidence>,
    pub listener_pairs: Vec<ListenerPair>,
    pub hidden_origin_controls: Vec<HiddenOriginControl>,
    pub attribution_pairs: Vec<AttributionPair>,
}
fn evidence(e: &Evaluation, f: Fixture, l: ListenerKind) -> &FixtureEvidence {
    e.fixtures
        .iter()
        .find(|v| v.fixture == f && v.listener == l)
        .unwrap()
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut e = Evaluation {
        version: "evaluation-004-v1".into(),
        runs: 0,
        replay_matches: 0,
        fixtures: vec![],
        listener_pairs: vec![],
        hidden_origin_controls: vec![],
        attribution_pairs: vec![],
    };
    for fixture in FIXTURES {
        for listener in [ListenerKind::SourceCount, ListenerKind::Provenance] {
            let mut entry = FixtureEvidence {
                fixture,
                listener,
                counts: Counts::default(),
                reports: vec![],
            };
            for seed in 0..16 {
                let config = fixture_config(fixture, seed, listener);
                let report = run(config.clone())?;
                let replay = run(config)?;
                if serde_json::to_vec(&report).map_err(|e| e.to_string())?
                    != serde_json::to_vec(&replay).map_err(|e| e.to_string())?
                {
                    return Err("004 replay mismatch".into());
                }
                entry.counts.add(&report)?;
                entry.reports.push(report);
                e.runs += 1;
                e.replay_matches += 1;
            }
            e.fixtures.push(entry);
        }
    }
    for fixture in FIXTURES {
        let mut pairs = vec![];
        for (b, a) in evidence(&e, fixture, ListenerKind::SourceCount)
            .reports
            .iter()
            .zip(&evidence(&e, fixture, ListenerKind::Provenance).reports)
        {
            let mut matched = a.config.clone();
            matched.listeners = b.config.listeners;
            let before = judgment(b, 5);
            let after = judgment(a, 5);
            let bg = b.updates[before.update].assessment.perceived_groups.len();
            let ag = a.updates[after.update].assessment.perceived_groups.len();
            pairs.push(ListenerPair {
                fixture,
                seed: b.config.seed,
                only_listeners_differ: matched == b.config,
                identical_delivered_messages: b
                    .testimony
                    .iter()
                    .map(|t| (&t.message, &t.recipients))
                    .eq(a.testimony.iter().map(|t| (&t.message, &t.recipients))),
                target_actual_chains: b.updates[before.update].observer_received_roots.len(),
                baseline_groups: bg,
                provenance_groups: ag,
                repeated_contributions_avoided: bg.saturating_sub(ag),
                target_scores_changed: before.input.current.scores != after.input.current.scores,
                target_candidate_changed: before.input.current.candidate
                    != after.input.current.candidate,
                target_vote_changed: before.choice.action != after.choice.action,
                verdict_changed: b.verdict != a.verdict,
                choices_changed: b
                    .decisions
                    .iter()
                    .zip(&a.decisions)
                    .filter(|(b, a)| b.choice.action != a.choice.action)
                    .count(),
            });
        }
        e.listener_pairs.extend(pairs);
    }
    for listener in [ListenerKind::SourceCount, ListenerKind::Provenance] {
        let mut controls = vec![];
        for (a, b) in evidence(&e, Fixture::Independent, listener)
            .reports
            .iter()
            .zip(&evidence(&e, Fixture::HiddenShared, listener).reports)
        {
            let ad = judgment(a, 5);
            let bd = judgment(b, 5);
            controls.push(HiddenOriginControl {
                seed: a.config.seed,
                listener,
                identical_target_update_inputs: a
                    .updates
                    .iter()
                    .filter(|u| u.input.id == 5)
                    .map(|u| &u.input)
                    .eq(b
                        .updates
                        .iter()
                        .filter(|u| u.input.id == 5)
                        .map(|u| &u.input)),
                identical_target_assessments: a
                    .updates
                    .iter()
                    .filter(|u| u.input.id == 5)
                    .map(|u| &u.assessment)
                    .eq(b
                        .updates
                        .iter()
                        .filter(|u| u.input.id == 5)
                        .map(|u| &u.assessment)),
                identical_target_votes: ad.choice.action == bd.choice.action,
                independent_actual_chains: a.updates[ad.update].observer_received_roots.len(),
                hidden_actual_chains: b.updates[bd.update].observer_received_roots.len(),
            });
        }
        e.hidden_origin_controls.extend(controls);
    }
    for (control, treatment) in [
        (Fixture::KnownShared, Fixture::UnspecifiedShared),
        (Fixture::Independent, Fixture::CollapsedIndependent),
    ] {
        let mut pairs = vec![];
        for (a, b) in evidence(&e, control, ListenerKind::Provenance)
            .reports
            .iter()
            .zip(&evidence(&e, treatment, ListenerKind::Provenance).reports)
        {
            let ad = judgment(a, 5);
            let bd = judgment(b, 5);
            pairs.push(AttributionPair {
                control,
                treatment,
                seed: a.config.seed,
                matched_initial_states: a.config.observations == b.config.observations
                    && a.config.incentives == b.config.incentives,
                same_speakers_and_propositions: a
                    .testimony
                    .iter()
                    .map(|t| (t.message.speaker, &t.message.speech))
                    .eq(b
                        .testimony
                        .iter()
                        .map(|t| (t.message.speaker, &t.message.speech))),
                actual_roots_unchanged: a
                    .testimony
                    .iter()
                    .map(|t| &t.lineage.roots)
                    .eq(b.testimony.iter().map(|t| &t.lineage.roots)),
                target_groups_changed: a.updates[ad.update].assessment.perceived_groups
                    != b.updates[bd.update].assessment.perceived_groups,
                target_vote_changed: ad.choice.action != bd.choice.action,
                verdict_changed: a.verdict != b.verdict,
            });
        }
        e.attribution_pairs.extend(pairs);
    }
    Ok(e)
}
