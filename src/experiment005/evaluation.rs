use super::*;
use crate::{Phase, experiment004 as e4};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fixture {
    Retain,
    Sincere,
    Retraction,
    Misleading,
    WrongSincere,
    SelectiveLow,
    SelectiveHigh,
    RelayBoth,
    RelayIntermediate,
    RelayFinal,
    RelayNeither,
    RelayWithheld,
    MissingOriginal,
    Independent,
    Injected,
    Branches,
    Sequential,
    Dispute,
}
pub const FIXTURES: [Fixture; 18] = [
    Fixture::Retain,
    Fixture::Sincere,
    Fixture::Retraction,
    Fixture::Misleading,
    Fixture::WrongSincere,
    Fixture::SelectiveLow,
    Fixture::SelectiveHigh,
    Fixture::RelayBoth,
    Fixture::RelayIntermediate,
    Fixture::RelayFinal,
    Fixture::RelayNeither,
    Fixture::RelayWithheld,
    Fixture::MissingOriginal,
    Fixture::Independent,
    Fixture::Injected,
    Fixture::Branches,
    Fixture::Sequential,
    Fixture::Dispute,
];
fn injected(
    before_slot: usize,
    body: Body,
    attribution: Attribution,
    basis: Basis,
    intent: Intent,
) -> Injection {
    let mut choice = Choice::silence("explicit controller intervention; not policy initiative");
    choice.action = Action::Speak {
        body,
        attribution,
        basis,
        recipients: (0..6).collect(),
    };
    choice.intent = intent;
    Injection {
        before_slot,
        speaker: 0,
        choice,
    }
}
pub fn fixture_config(f: Fixture, seed: u64, listener: ListenerKind) -> Config {
    let mut c = Config::new(seed);
    let actor = c.actor();
    let wrong = (actor + 1) % 6;
    let other = (actor + 2) % 6;
    c.observations[0] = Some(wrong);
    c.listeners = [listener; 6];
    c.speakers[0] = SpeakerKind::Sincere;
    c.later.push(LaterObservation {
        before_slot: 6,
        agent: 0,
        candidate: Some(actor),
    });
    match f {
        Fixture::Retain => {
            c.speakers[0] = SpeakerKind::Retain(e4::SpeakerKind::baseline(PolicyKind::Truthful))
        }
        Fixture::Sincere => {}
        Fixture::Retraction => c.later[0].candidate = None,
        Fixture::Misleading => c.speakers[0] = SpeakerKind::Misleading,
        Fixture::WrongSincere => c.later[0].candidate = Some(other),
        Fixture::SelectiveLow | Fixture::SelectiveHigh => {
            c.speakers[0] = SpeakerKind::Selective;
            c.revision_cost[0] = if f == Fixture::SelectiveHigh { 3 } else { 1 };
        }
        Fixture::RelayBoth
        | Fixture::RelayIntermediate
        | Fixture::RelayFinal
        | Fixture::RelayNeither
        | Fixture::RelayWithheld => {
            c.speakers[1] = SpeakerKind::RelayRevision {
                source: 0,
                benefit: if f == Fixture::RelayWithheld { 0 } else { 2 },
                cost: 1,
            };
            c.routes[1] = vec![2, 3, 4, 5];
            c.second_routes[1] = vec![2, 3, 4, 5];
            c.second_routes[0] = match f {
                Fixture::RelayIntermediate | Fixture::RelayWithheld => vec![0, 1],
                Fixture::RelayFinal => vec![0, 2, 3, 4, 5],
                Fixture::RelayNeither => vec![0],
                _ => (0..6).collect(),
            };
        }
        Fixture::MissingOriginal => c.routes[0] = vec![0, 1],
        Fixture::Independent => {
            c.speakers[0] = SpeakerKind::Retain(e4::SpeakerKind::baseline(PolicyKind::Truthful));
            c.observations[2] = Some(actor);
            c.speakers[2] = SpeakerKind::LateTruth;
        }
        Fixture::Dispute => {
            c.observations[2] = Some(wrong);
            c.speakers[2] = SpeakerKind::LateTruth;
        }
        Fixture::Injected | Fixture::Branches | Fixture::Sequential => {
            c.speakers[0] = SpeakerKind::Retain(e4::SpeakerKind::baseline(PolicyKind::Silent));
            c.injections.push(injected(
                0,
                Body::Assert(wrong),
                Attribution::Firsthand,
                Basis::Private,
                Intent::Sincere,
            ));
            c.injections.push(injected(
                6,
                Body::Revise {
                    target: 0,
                    replacement: Some(actor),
                    forwarded: None,
                },
                Attribution::Unspecified,
                Basis::Private,
                Intent::Sincere,
            ));
            if f != Fixture::Injected {
                c.injections.push(injected(
                    7,
                    Body::Revise {
                        target: if f == Fixture::Sequential { 1 } else { 0 },
                        replacement: Some(other),
                        forwarded: None,
                    },
                    Attribution::Unspecified,
                    Basis::Inference,
                    Intent::Unspecified,
                ));
            }
        }
    }
    c
}
pub fn judgment(r: &Report, id: u8) -> &Decision {
    r.decisions
        .iter()
        .find(|d| d.input.id == id && d.input.phase == Phase::Judgment)
        .unwrap()
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub selected_assertions: usize,
    pub selected_corrections: usize,
    pub selected_retractions: usize,
    pub injected_revisions: usize,
    pub recognized_changes: usize,
    pub withheld: usize,
    pub faithful_corrections: usize,
    pub sincere_repairs: usize,
    pub sincere_still_wrong: usize,
    pub misleading_corrections: usize,
    pub unclassifiable: usize,
    pub rejected: usize,
    pub target_recoveries: usize,
    pub target_became_uncertain: usize,
    pub target_still_wrong: usize,
    pub target_correct_votes: usize,
    pub target_wrong_votes: usize,
    pub target_abstentions: usize,
    pub correct_verdicts: usize,
    pub wrong_verdicts: usize,
    pub no_verdicts: usize,
    pub candidate_changes: usize,
    pub score_changes: usize,
    pub target_active: usize,
    pub target_superseded: usize,
    pub target_retracted: usize,
    pub target_disputed: usize,
    pub target_missing: usize,
    pub target_ignored: usize,
    pub target_contributing_groups: usize,
    pub target_contested_groups: usize,
    pub observation_deliveries: usize,
    pub delivered_copies: usize,
    pub retained_sent: usize,
    pub message_bytes: usize,
    pub lineage_bytes: usize,
    pub report_bytes: usize,
    pub max_depth: u8,
}
impl Counts {
    fn add(&mut self, r: &Report) -> Result<(), String> {
        let final_d = judgment(r, 5);
        let last_first = r
            .decisions
            .iter()
            .find(|d| d.input.phase == Phase::Discussion && d.input.id == 5)
            .unwrap();
        let old = last_first.input.current.candidate;
        let final_c = final_d.input.current.candidate;
        let actor = r.config.actor();
        self.target_recoveries +=
            usize::from(old.is_some_and(|c| c != actor) && final_c == Some(actor));
        self.target_became_uncertain += usize::from(old.is_some() && final_c.is_none());
        self.target_still_wrong += usize::from(final_c.is_some_and(|c| c != actor));
        match final_d.choice.action {
            Action::Vote(Some(c)) if c == actor => self.target_correct_votes += 1,
            Action::Vote(Some(_)) => self.target_wrong_votes += 1,
            Action::Vote(None) => self.target_abstentions += 1,
            _ => {}
        }
        for d in &r.decisions {
            self.recognized_changes += usize::from(d.choice.recognized_change);
            self.withheld += usize::from(d.classification == Classification::Withheld);
            self.rejected += usize::from(d.rejection.is_some());
        }
        for t in &r.testimony {
            match (&t.origin, &t.message.body) {
                (Origin::Selected { .. }, Body::Assert(_)) => self.selected_assertions += 1,
                (
                    Origin::Selected { .. },
                    Body::Revise {
                        replacement: Some(_),
                        ..
                    },
                ) => self.selected_corrections += 1,
                (
                    Origin::Selected { .. },
                    Body::Revise {
                        replacement: None, ..
                    },
                ) => self.selected_retractions += 1,
                (Origin::Injected { .. }, Body::Revise { .. }) => self.injected_revisions += 1,
                _ => {}
            }
            self.faithful_corrections +=
                usize::from(t.classification == Classification::FaithfulCorrection);
            self.sincere_repairs += usize::from(t.classification == Classification::SincereRepair);
            self.sincere_still_wrong +=
                usize::from(t.classification == Classification::SincereStillWrong);
            self.misleading_corrections += usize::from(
                t.classification == Classification::IntentionalContradiction
                    && matches!(t.message.body, Body::Revise { .. }),
            );
            self.unclassifiable += usize::from(t.classification == Classification::Unclassifiable);
            self.message_bytes += serde_json::to_vec(&t.message)
                .map_err(|e| e.to_string())?
                .len()
                * t.recipients.len();
            self.lineage_bytes += serde_json::to_vec(&t.lineage)
                .map_err(|e| e.to_string())?
                .len();
            self.max_depth = self.max_depth.max(t.lineage.depth);
        }
        for u in &r.updates {
            self.candidate_changes +=
                usize::from(u.input.prior.candidate != u.assessment.after.candidate);
            self.score_changes += usize::from(u.input.prior.scores != u.assessment.after.scores);
        }
        let a = &r.updates[final_d.update].assessment;
        for i in &a.interpretations {
            match i.standing {
                Standing::Active => self.target_active += 1,
                Standing::Superseded => self.target_superseded += 1,
                Standing::Retracted => self.target_retracted += 1,
                Standing::Disputed => self.target_disputed += 1,
                Standing::MissingOriginal => self.target_missing += 1,
                _ => self.target_ignored += 1,
            }
        }
        self.target_contributing_groups += a.groups.iter().filter(|g| g.contributes).count();
        self.target_contested_groups += a.groups.iter().filter(|g| g.contested).count();
        self.observation_deliveries += r.config.later.len();
        self.delivered_copies += r.retained_received_copies;
        self.retained_sent += r.retained_sent_copies;
        self.report_bytes += serde_json::to_vec(r).map_err(|e| e.to_string())?.len();
        match r.correct {
            Some(true) => self.correct_verdicts += 1,
            Some(false) => self.wrong_verdicts += 1,
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
pub struct Comparison {
    pub control: Fixture,
    pub treatment: Fixture,
    pub listener: ListenerKind,
    pub seed: u64,
    pub changed_config_fields: Vec<String>,
    pub identical_wire_and_recipients: bool,
    pub identical_target_inputs: bool,
    pub identical_target_assessments: bool,
    pub scores_changed: bool,
    pub candidate_changed: bool,
    pub vote_changed: bool,
    pub verdict_changed: bool,
    pub selected_communication_changed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub runs: usize,
    pub replay_matches: usize,
    pub fixtures: Vec<FixtureEvidence>,
    pub listener_pairs: Vec<Comparison>,
    pub mechanism_pairs: Vec<Comparison>,
}
fn compare(
    a: &Report,
    b: &Report,
    control: Fixture,
    treatment: Fixture,
    listener: ListenerKind,
) -> Comparison {
    let av = serde_json::to_value(&a.config).unwrap();
    let bv = serde_json::to_value(&b.config).unwrap();
    let changed_config_fields = av
        .as_object()
        .unwrap()
        .iter()
        .filter(|(k, v)| bv.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .collect();
    let ad = judgment(a, 5);
    let bd = judgment(b, 5);
    Comparison {
        control,
        treatment,
        listener,
        seed: a.config.seed,
        changed_config_fields,
        identical_wire_and_recipients: a
            .testimony
            .iter()
            .map(|t| (&t.message, &t.recipients))
            .eq(b.testimony.iter().map(|t| (&t.message, &t.recipients))),
        identical_target_inputs: a
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
        scores_changed: ad.input.current.scores != bd.input.current.scores,
        candidate_changed: ad.input.current.candidate != bd.input.current.candidate,
        vote_changed: ad.choice.action != bd.choice.action,
        verdict_changed: a.verdict != b.verdict,
        selected_communication_changed: a.decisions[..12]
            .iter()
            .map(|d| &d.choice.action)
            .ne(b.decisions[..12].iter().map(|d| &d.choice.action)),
    }
}
fn evidence(e: &Evaluation, f: Fixture, l: ListenerKind) -> &FixtureEvidence {
    e.fixtures
        .iter()
        .find(|v| v.fixture == f && v.listener == l)
        .unwrap()
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut e = Evaluation {
        version: "evaluation-005-v1".into(),
        runs: 0,
        replay_matches: 0,
        fixtures: vec![],
        listener_pairs: vec![],
        mechanism_pairs: vec![],
    };
    for f in FIXTURES {
        for l in [ListenerKind::Frozen004, ListenerKind::Revision] {
            let mut row = FixtureEvidence {
                fixture: f,
                listener: l,
                counts: Counts::default(),
                reports: vec![],
            };
            for seed in 0..16 {
                let c = fixture_config(f, seed, l);
                let r = run(c.clone())?;
                if serde_json::to_vec(&r).map_err(|e| e.to_string())?
                    != serde_json::to_vec(&run(c)?).map_err(|e| e.to_string())?
                {
                    return Err("005 replay mismatch".into());
                }
                row.counts.add(&r)?;
                row.reports.push(r);
                e.runs += 1;
                e.replay_matches += 1;
            }
            e.fixtures.push(row);
        }
    }
    for f in FIXTURES {
        let pairs = evidence(&e, f, ListenerKind::Frozen004)
            .reports
            .iter()
            .zip(&evidence(&e, f, ListenerKind::Revision).reports)
            .map(|(a, b)| compare(a, b, f, f, ListenerKind::Revision))
            .collect::<Vec<_>>();
        e.listener_pairs.extend(pairs);
    }
    for (a, b) in [
        (Fixture::Retain, Fixture::Sincere),
        (Fixture::Retain, Fixture::SelectiveHigh),
        (Fixture::Sincere, Fixture::SelectiveHigh),
        (Fixture::SelectiveLow, Fixture::SelectiveHigh),
        (Fixture::RelayBoth, Fixture::RelayIntermediate),
        (Fixture::RelayBoth, Fixture::RelayFinal),
        (Fixture::RelayBoth, Fixture::RelayNeither),
        (Fixture::RelayIntermediate, Fixture::RelayWithheld),
        (Fixture::Sincere, Fixture::Injected),
        (Fixture::Sincere, Fixture::Independent),
        (Fixture::Branches, Fixture::Sequential),
        (Fixture::Sincere, Fixture::WrongSincere),
    ] {
        for l in [ListenerKind::Frozen004, ListenerKind::Revision] {
            let pairs = evidence(&e, a, l)
                .reports
                .iter()
                .zip(&evidence(&e, b, l).reports)
                .map(|(x, y)| compare(x, y, a, b, l))
                .collect::<Vec<_>>();
            e.mechanism_pairs.extend(pairs);
        }
    }
    Ok(e)
}
