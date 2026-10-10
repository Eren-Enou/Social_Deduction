use super::*;
use crate::{experiment004 as e4, experiment005 as e5};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fixture {
    Neutral,
    Accurate,
    Inaccurate,
    DifferentHistories,
    MissingFeedback,
    WrongVerdict,
    Corrected,
    Uncorrected,
    MisleadingCorrection,
    RelayOrigin,
    OpaqueRelay,
    Misattributed,
    SelectiveFeedback,
    Reversal,
    IdentityPermuted,
    OrderReversed,
    InjectedExperience,
    TrustedFalse,
    DelayedFeedback,
    Eviction,
    Interaction,
    UnverifiedClaim,
}
pub const FIXTURES: [Fixture; 22] = [
    Fixture::Neutral,
    Fixture::Accurate,
    Fixture::Inaccurate,
    Fixture::DifferentHistories,
    Fixture::MissingFeedback,
    Fixture::WrongVerdict,
    Fixture::Corrected,
    Fixture::Uncorrected,
    Fixture::MisleadingCorrection,
    Fixture::RelayOrigin,
    Fixture::OpaqueRelay,
    Fixture::Misattributed,
    Fixture::SelectiveFeedback,
    Fixture::Reversal,
    Fixture::IdentityPermuted,
    Fixture::OrderReversed,
    Fixture::InjectedExperience,
    Fixture::TrustedFalse,
    Fixture::DelayedFeedback,
    Fixture::Eviction,
    Fixture::Interaction,
    Fixture::UnverifiedClaim,
];
fn truthful() -> e5::SpeakerKind {
    e5::SpeakerKind::Retain(e4::SpeakerKind::baseline(e5::PolicyKind::Truthful))
}
fn history(seed: u64, accurate: bool) -> e5::Config {
    let mut c = e5::Config::new(seed);
    c.observations[0] = Some(if accurate {
        c.actor()
    } else {
        (c.actor() + 1) % 6
    });
    c.speakers[0] = truthful();
    c
}
fn assessment(seed: u64, false_source: bool) -> e5::Config {
    let mut c = e5::Config::new(seed);
    let actor = c.actor();
    let wrong = (actor + 1) % 6;
    c.observations[0] = Some(if false_source { wrong } else { actor });
    c.observations[2] = Some(if false_source { actor } else { wrong });
    c.speakers[0] = truthful();
    c.speakers[2] = truthful();
    c.routes[0] = vec![1, 3, 4, 5];
    c.routes[2] = vec![1, 3, 4, 5];
    c
}
pub fn fixture_config(f: Fixture, seed: u64, listener: ListenerKind) -> Config {
    let mut c = Config::new(vec![]);
    c.listener = listener;
    let base = seed * 16;
    let history_flags: Vec<bool> = match f {
        Fixture::Neutral => vec![],
        Fixture::Inaccurate
        | Fixture::WrongVerdict
        | Fixture::Corrected
        | Fixture::Uncorrected
        | Fixture::MisleadingCorrection
        | Fixture::RelayOrigin
        | Fixture::OpaqueRelay
        | Fixture::Misattributed => vec![false],
        Fixture::DifferentHistories => vec![true, false],
        Fixture::SelectiveFeedback => vec![true, true, false, false],
        Fixture::Reversal => vec![true, true, false, false, false],
        Fixture::Eviction => vec![true, false, false, false, false, false, false],
        _ => vec![true, true],
    };
    for (i, accurate) in history_flags.into_iter().enumerate() {
        let mut h = history(base + i as u64, accurate);
        if matches!(
            f,
            Fixture::Corrected | Fixture::Uncorrected | Fixture::MisleadingCorrection
        ) {
            h.later.push(e5::LaterObservation {
                before_slot: 6,
                agent: 0,
                candidate: Some(h.actor()),
            });
            h.speakers[0] = match f {
                Fixture::Corrected => e5::SpeakerKind::Sincere,
                Fixture::MisleadingCorrection => e5::SpeakerKind::Misleading,
                _ => truthful(),
            };
        }
        if matches!(
            f,
            Fixture::RelayOrigin | Fixture::OpaqueRelay | Fixture::Misattributed
        ) {
            for id in [1, 2] {
                h.speakers[id] = e5::SpeakerKind::Retain(e4::SpeakerKind::Relay {
                    source: 0,
                    presentation: if f == Fixture::Misattributed {
                        e4::Presentation::Named(3)
                    } else {
                        e4::Presentation::Disclose
                    },
                });
                h.routes[id] = vec![3, 4, 5];
            }
            if f != Fixture::RelayOrigin {
                h.routes[0] = vec![0, 1, 2];
            }
        }
        if f == Fixture::InjectedExperience {
            h.speakers[0] =
                e5::SpeakerKind::Retain(e4::SpeakerKind::baseline(e5::PolicyKind::Silent));
            let mut choice =
                e5::Choice::silence("declared experience intervention, not policy initiative");
            choice.action = e5::Action::Speak {
                body: e5::Body::Assert(h.actor()),
                attribution: e5::Attribution::Firsthand,
                basis: e5::Basis::Private,
                recipients: (0..6).collect(),
            };
            choice.intent = e5::Intent::Sincere;
            h.injections.push(e5::Injection {
                before_slot: 0,
                speaker: 0,
                choice,
            });
        }
        c.incidents.push(h);
        if !matches!(f, Fixture::MissingFeedback | Fixture::DelayedFeedback)
            && !(f == Fixture::Eviction && i == 0)
        {
            c.feedback.push(FeedbackPlan {
                after_episode: i as u8,
                about_episode: i as u8,
                recipients: match f {
                    Fixture::DifferentHistories => {
                        if i == 0 {
                            vec![3]
                        } else {
                            vec![5]
                        }
                    }
                    Fixture::SelectiveFeedback => {
                        if i < 2 {
                            vec![3]
                        } else {
                            vec![5]
                        }
                    }
                    _ => (0..6).collect(),
                },
                kind: match f {
                    Fixture::WrongVerdict => FeedbackKind::CollectiveVerdict,
                    Fixture::UnverifiedClaim => {
                        FeedbackKind::ParticipantClaim((c.incidents[i].actor() + 1) % 6)
                    }
                    _ => FeedbackKind::Verification,
                },
            });
        }
    }
    if f == Fixture::DelayedFeedback {
        for episode in 0..2 {
            c.feedback.push(FeedbackPlan {
                after_episode: 1,
                about_episode: episode,
                recipients: (0..6).collect(),
                kind: FeedbackKind::Verification,
            });
        }
    }
    let mut current = assessment(base + c.incidents.len() as u64, f == Fixture::TrustedFalse);
    if f == Fixture::Interaction {
        current.routes[0].push(2);
    }
    c.incidents.push(current);
    if f == Fixture::Eviction {
        c.feedback.push(FeedbackPlan {
            after_episode: 7,
            about_episode: 0,
            recipients: (0..6).collect(),
            kind: FeedbackKind::Verification,
        });
    }
    if f == Fixture::OrderReversed {
        for h in &mut c.incidents {
            h.order = [5, 4, 3, 2, 1, 0];
        }
    }
    if f == Fixture::IdentityPermuted {
        let map = [3, 1, 0, 2, 4, 5];
        for h in &mut c.incidents {
            let old = h.clone();
            for (id, &new_id) in map.iter().enumerate() {
                let to = usize::from(new_id);
                h.observations[to] = old.observations[id];
                h.speakers[to] = old.speakers[id];
                h.routes[to] = old.routes[id]
                    .iter()
                    .map(|&r| map[usize::from(r)])
                    .collect();
                h.second_routes[to] = old.second_routes[id]
                    .iter()
                    .map(|&r| map[usize::from(r)])
                    .collect();
            }
        }
        for feedback in &mut c.feedback {
            feedback.recipients = feedback
                .recipients
                .iter()
                .map(|&id| map[usize::from(id)])
                .collect();
        }
    }
    c
}
pub fn last_episode(r: &Report) -> &Episode {
    r.episodes.last().unwrap()
}
pub fn judgment(r: &Report, id: u8) -> &e5::Decision {
    e5::judgment(&last_episode(r).incident, id)
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub incidents: usize,
    pub correct_verdicts: usize,
    pub wrong_verdicts: usize,
    pub unresolved: usize,
    pub target_correct_votes: usize,
    pub target_wrong_votes: usize,
    pub target_abstentions: usize,
    pub target_scores_sum: [i32; 6],
    pub target_verified_sum: [u32; 6],
    pub observer_score_disagreements: usize,
    pub graded_sources: usize,
    pub unknown_sources: usize,
    pub unverified_ignored: usize,
    pub duplicate_verifications: usize,
    pub evictions: usize,
    pub late_feedback_without_record: usize,
    pub duplicates_prevented: usize,
    pub record_inspections: usize,
    pub group_weight_operations: usize,
    pub cognition_calls: usize,
    pub memory_bytes: usize,
    pub pending_records: usize,
    pub retained_verified_records: usize,
    pub original_accurate: usize,
    pub original_inaccurate: usize,
    pub final_accurate: usize,
    pub final_inaccurate: usize,
    pub corrections: usize,
    pub retractions: usize,
    pub relay_accurate: usize,
    pub relay_inaccurate: usize,
    pub max_memory_records: usize,
    pub report_bytes: usize,
    pub rejected: usize,
}
impl Counts {
    fn add(&mut self, r: &Report) -> Result<(), String> {
        self.incidents += r.episodes.len();
        let last = last_episode(r);
        match last.incident.correct {
            Some(true) => self.correct_verdicts += 1,
            Some(false) => self.wrong_verdicts += 1,
            None => self.unresolved += 1,
        }
        let d = judgment(r, 5);
        match d.choice.action {
            e5::Action::Vote(Some(id)) if id == last.incident.config.actor() => {
                self.target_correct_votes += 1
            }
            e5::Action::Vote(Some(_)) => self.target_wrong_votes += 1,
            e5::Action::Vote(None) => self.target_abstentions += 1,
            _ => {}
        }
        for id in 0..6 {
            self.target_scores_sum[id] += i32::from(last.memory_before[5].credits[id].score);
            self.target_verified_sum[id] +=
                u32::from(last.memory_before[5].credits[id].verified_count());
        }
        self.observer_score_disagreements += usize::from(
            last.memory_before[3]
                .credits
                .iter()
                .map(|c| c.score)
                .ne(last.memory_before[5].credits.iter().map(|c| c.score)),
        );
        for episode in &r.episodes {
            self.cognition_calls += episode.cognition.len();
            self.group_weight_operations += episode
                .cognition
                .iter()
                .map(|c| c.weights.len())
                .sum::<usize>();
            self.rejected += episode
                .incident
                .decisions
                .iter()
                .filter(|d| d.rejection.is_some())
                .count();
            for i in &episode.ingestions {
                self.evictions += i.evicted.len();
                self.duplicates_prevented += i.duplicate_reports_prevented;
                self.record_inspections += i.record_inspections;
            }
            for m in &episode.memory_before {
                self.max_memory_records = self.max_memory_records.max(m.records.len());
            }
        }
        for f in &r.feedback {
            self.graded_sources += f.effect.graded_sources;
            self.unknown_sources += f.effect.unknown_sources;
            self.unverified_ignored += f.effect.ignored_unverified;
            self.duplicate_verifications += f.effect.duplicate_verifications;
            self.late_feedback_without_record += usize::from(f.effect.no_retained_experience);
            self.record_inspections += f.effect.record_inspections;
        }
        for m in &r.final_memory {
            self.pending_records += m.records.iter().filter(|r| r.verified.is_none()).count();
            self.retained_verified_records +=
                m.records.iter().filter(|r| r.verified.is_some()).count();
            for c in &m.credits {
                self.original_accurate += usize::from(c.original_accurate);
                self.original_inaccurate += usize::from(c.original_inaccurate);
                self.final_accurate += usize::from(c.accurate);
                self.final_inaccurate += usize::from(c.inaccurate);
                self.corrections += usize::from(c.corrections);
                self.retractions += usize::from(c.retractions);
                self.relay_accurate += usize::from(c.relay_accurate);
                self.relay_inaccurate += usize::from(c.relay_inaccurate);
            }
            self.max_memory_records = self.max_memory_records.max(m.records.len());
            self.memory_bytes += serde_json::to_vec(m).map_err(|e| e.to_string())?.len();
        }
        self.report_bytes += serde_json::to_vec(r).map_err(|e| e.to_string())?.len();
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
pub struct Pair {
    pub fixture: Fixture,
    pub seed: u64,
    pub only_listener_differs: bool,
    pub identical_current_config: bool,
    pub identical_current_wire: bool,
    pub identical_current_private_inputs: bool,
    pub identical_memory_before: bool,
    pub target_support_changed: bool,
    pub target_vote_changed: bool,
    pub verdict_changed: bool,
    pub communication_changed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Control {
    pub control: Fixture,
    pub treatment: Fixture,
    pub seed: u64,
    pub target_scores_identical: bool,
    pub target_credit_inputs_identical: bool,
    pub target_votes_identical: bool,
    pub identical_current_wire: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub series: usize,
    pub incidents: usize,
    pub replay_matches: usize,
    pub fixtures: Vec<FixtureEvidence>,
    pub listener_pairs: Vec<Pair>,
    pub controls: Vec<Control>,
}
fn row(e: &Evaluation, f: Fixture, l: ListenerKind) -> &FixtureEvidence {
    e.fixtures
        .iter()
        .find(|v| v.fixture == f && v.listener == l)
        .unwrap()
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut e = Evaluation {
        version: "evaluation-006-v1".into(),
        series: 0,
        incidents: 0,
        replay_matches: 0,
        fixtures: vec![],
        listener_pairs: vec![],
        controls: vec![],
    };
    for f in FIXTURES {
        for l in [ListenerKind::Frozen005, ListenerKind::Credibility] {
            let mut evidence = FixtureEvidence {
                fixture: f,
                listener: l,
                counts: Counts::default(),
                reports: vec![],
            };
            for seed in 0..8 {
                let c = fixture_config(f, seed, l);
                let r = run(c.clone())?;
                if serde_json::to_vec(&r).map_err(|e| e.to_string())?
                    != serde_json::to_vec(&run(c)?).map_err(|e| e.to_string())?
                {
                    return Err("006 replay mismatch".into());
                }
                e.series += 1;
                e.replay_matches += 1;
                e.incidents += r.episodes.len();
                evidence.counts.add(&r)?;
                evidence.reports.push(r);
            }
            e.fixtures.push(evidence);
        }
    }
    for f in FIXTURES {
        let pairs = row(&e, f, ListenerKind::Frozen005)
            .reports
            .iter()
            .zip(&row(&e, f, ListenerKind::Credibility).reports)
            .map(|(a, b)| {
                let mut matched = b.config.clone();
                matched.listener = a.config.listener;
                let x = last_episode(a);
                let y = last_episode(b);
                Pair {
                    fixture: f,
                    seed: a.config.incidents[0].seed / 16,
                    only_listener_differs: matched == a.config,
                    identical_current_config: x.incident.config == y.incident.config,
                    identical_current_wire: x
                        .incident
                        .testimony
                        .iter()
                        .map(|t| (&t.message, &t.recipients))
                        .eq(y
                            .incident
                            .testimony
                            .iter()
                            .map(|t| (&t.message, &t.recipients))),
                    identical_current_private_inputs: x
                        .incident
                        .decisions
                        .iter()
                        .zip(&y.incident.decisions)
                        .all(|(a, b)| {
                            a.input.original == b.input.original
                                && a.input.observations == b.input.observations
                        }),
                    identical_memory_before: x.memory_before == y.memory_before,
                    target_support_changed: judgment(a, 5).input.current.scores
                        != judgment(b, 5).input.current.scores,
                    target_vote_changed: judgment(a, 5).choice.action
                        != judgment(b, 5).choice.action,
                    verdict_changed: x.incident.verdict != y.incident.verdict,
                    communication_changed: x.incident.decisions[..12]
                        .iter()
                        .map(|d| &d.choice.action)
                        .ne(y.incident.decisions[..12].iter().map(|d| &d.choice.action)),
                }
            })
            .collect::<Vec<_>>();
        e.listener_pairs.extend(pairs);
    }
    for (a, b) in [
        (Fixture::Accurate, Fixture::MissingFeedback),
        (Fixture::Accurate, Fixture::Inaccurate),
        (Fixture::Accurate, Fixture::TrustedFalse),
        (Fixture::Accurate, Fixture::DelayedFeedback),
        (Fixture::Accurate, Fixture::IdentityPermuted),
        (Fixture::Accurate, Fixture::OrderReversed),
        (Fixture::Accurate, Fixture::InjectedExperience),
        (Fixture::Corrected, Fixture::Uncorrected),
        (Fixture::Corrected, Fixture::MisleadingCorrection),
        (Fixture::DifferentHistories, Fixture::SelectiveFeedback),
        (Fixture::RelayOrigin, Fixture::OpaqueRelay),
        (Fixture::RelayOrigin, Fixture::Misattributed),
    ] {
        let controls = row(&e, a, ListenerKind::Credibility)
            .reports
            .iter()
            .zip(&row(&e, b, ListenerKind::Credibility).reports)
            .map(|(x, y)| Control {
                control: a,
                treatment: b,
                seed: x.config.incidents[0].seed / 16,
                target_scores_identical: judgment(x, 5).input.current.scores
                    == judgment(y, 5).input.current.scores,
                target_credit_inputs_identical: last_episode(x).memory_before[5].credits
                    == last_episode(y).memory_before[5].credits,
                target_votes_identical: judgment(x, 5).choice.action
                    == judgment(y, 5).choice.action,
                identical_current_wire: last_episode(x)
                    .incident
                    .testimony
                    .iter()
                    .map(|t| (&t.message, &t.recipients))
                    .eq(last_episode(y)
                        .incident
                        .testimony
                        .iter()
                        .map(|t| (&t.message, &t.recipients))),
            })
            .collect::<Vec<_>>();
        e.controls.extend(controls);
    }
    Ok(e)
}
