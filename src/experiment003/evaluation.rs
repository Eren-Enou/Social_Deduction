use super::*;
use crate::{N, Phase};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fixture {
    TruthRetain,
    Truth,
    Deceptive,
    Silence,
    Blame,
    OriginalVote,
    Relabeled,
    ReverseOrder,
    Absent,
    OneTruth,
    OneMistake,
    OneLie,
    Conflict,
    Repeat,
    Uncertain,
    HiddenTruth,
    HiddenContradiction,
}
pub const FIXTURES: [Fixture; 17] = [
    Fixture::TruthRetain,
    Fixture::Truth,
    Fixture::Deceptive,
    Fixture::Silence,
    Fixture::Blame,
    Fixture::OriginalVote,
    Fixture::Relabeled,
    Fixture::ReverseOrder,
    Fixture::Absent,
    Fixture::OneTruth,
    Fixture::OneMistake,
    Fixture::OneLie,
    Fixture::Conflict,
    Fixture::Repeat,
    Fixture::Uncertain,
    Fixture::HiddenTruth,
    Fixture::HiddenContradiction,
];
pub fn fixture_config(fixture: Fixture, seed: u64) -> Config {
    let mut config = Config::new(seed);
    let actor = config.actor();
    let wrong = (actor + 1) % N;
    match fixture {
        Fixture::TruthRetain => config.listeners = [ListenerKind::Retain; 6],
        Fixture::Truth => {}
        Fixture::Deceptive => config.communication = [PolicyKind::DeceptiveControl; 6],
        Fixture::Silence => config.communication = [PolicyKind::Silent; 6],
        Fixture::Blame => {
            config.communication = [PolicyKind::IncentiveSensitive; 6];
            config.incentives = [Incentives::BLAME; 6];
        }
        Fixture::OriginalVote => config.judgments = [JudgmentKind::Original; 6],
        Fixture::Relabeled => {
            config.layout = [3, 4, 5, 0, 1, 2];
            config.speaking_order = config.layout;
        }
        Fixture::ReverseOrder => config.speaking_order = [5, 4, 3, 2, 1, 0],
        Fixture::HiddenTruth | Fixture::HiddenContradiction => {
            config.participation_briefing = false;
            config.communication = [PolicyKind::Silent; 6];
            let mut observations = [None; 6];
            if fixture == Fixture::HiddenTruth {
                observations[0] = Some(actor);
                config.communication[0] = PolicyKind::Truthful;
            } else {
                observations[0] = Some((actor + 5) % N);
                config.communication[0] = PolicyKind::DeceptiveControl;
            }
            config.observation_override = Some(observations);
        }
        _ => {
            config.participation_briefing = false;
            config.communication = [PolicyKind::Silent; 6];
            config.observation_override = Some([Some(actor), Some(wrong), None, None, None, None]);
            let truth = Injection {
                speaker: 0,
                speech: Speech::Assert(actor),
                intent: Intent::ReportBelief,
                recipients: vec![2, 3, 4, 5],
            };
            let mistake = Injection {
                speaker: 1,
                speech: Speech::Assert(wrong),
                intent: Intent::ReportBelief,
                recipients: vec![2, 3, 4, 5],
            };
            let lie = Injection {
                speaker: 0,
                speech: Speech::Assert(wrong),
                intent: Intent::ContradictBelief,
                recipients: vec![2, 3, 4, 5],
            };
            config.injections = match fixture {
                Fixture::Absent => vec![],
                Fixture::OneTruth => vec![truth],
                Fixture::OneMistake => vec![mistake],
                Fixture::OneLie => vec![lie],
                Fixture::Conflict => vec![truth, mistake],
                Fixture::Repeat => vec![truth.clone(), truth],
                Fixture::Uncertain => vec![Injection {
                    speaker: 2,
                    speech: Speech::Uncertain,
                    intent: Intent::ExpressUncertainty,
                    recipients: vec![2, 3, 4, 5],
                }],
                _ => unreachable!(),
            };
        }
    }
    config
}
pub fn judgment(report: &Report, id: u8) -> &Decision {
    report
        .decisions
        .iter()
        .find(|d| d.input.id == id && d.input.phase == Phase::Judgment)
        .expect("complete report has a ballot decision per individual")
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub candidate_changes: usize,
    pub score_changes: usize,
    pub margin_changes: usize,
    pub unchanged_candidates: usize,
    pub changes_to_correct: usize,
    pub changes_to_incorrect: usize,
    pub changes_to_unknown: usize,
    pub ignored_assessments: usize,
    pub selected_messages: usize,
    pub injected_messages: usize,
    pub sincere_mistakes: usize,
    pub intentional_contradictions: usize,
    pub false_assertions: usize,
    pub rejected_decisions: usize,
    pub abstentions: usize,
    pub votes_differ_from_original: usize,
    pub correct_verdicts: usize,
    pub incorrect_verdicts: usize,
    pub no_verdict: usize,
}
impl Counts {
    fn add(&mut self, report: &Report) {
        for update in &report.updates {
            self.candidate_changes += usize::from(update.candidate_changed);
            self.unchanged_candidates += usize::from(!update.candidate_changed);
            self.score_changes +=
                usize::from(update.input.prior.scores != update.assessment.after.scores);
            self.margin_changes +=
                usize::from(update.input.prior.margin != update.assessment.after.margin);
            if update.candidate_changed {
                match update.after_correct {
                    Some(true) => self.changes_to_correct += 1,
                    Some(false) => self.changes_to_incorrect += 1,
                    None => self.changes_to_unknown += 1,
                }
            }
            self.ignored_assessments += update
                .assessment
                .messages
                .iter()
                .filter(|m| m.handling.ignored())
                .count();
        }
        for testimony in &report.testimony {
            match testimony.origin {
                Origin::Selected { .. } => self.selected_messages += 1,
                Origin::Intervention { .. } => self.injected_messages += 1,
            }
            self.sincere_mistakes +=
                usize::from(testimony.classification == Classification::SincereMistake);
            self.intentional_contradictions +=
                usize::from(testimony.classification == Classification::IntentionalContradiction);
            self.false_assertions += usize::from(testimony.assertion_matches_truth == Some(false));
        }
        for decision in &report.decisions {
            self.rejected_decisions += usize::from(decision.rejection.is_some());
            if decision.rejection.is_none()
                && let Action::Vote(candidate) = decision.choice.action
            {
                self.abstentions += usize::from(candidate.is_none());
                self.votes_differ_from_original +=
                    usize::from(candidate != decision.input.original.belief);
            }
        }
        match report.correct {
            Some(true) => self.correct_verdicts += 1,
            Some(false) => self.incorrect_verdicts += 1,
            None => self.no_verdict += 1,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureEvidence {
    pub fixture: Fixture,
    pub counts: Counts,
    pub reports: Vec<Report>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pair {
    pub seed: u64,
    pub control: Fixture,
    pub treatment: Fixture,
    pub recipient: u8,
    pub matched_original_state: bool,
    pub identical_final_listener_inputs: bool,
    pub candidate_changed: bool,
    pub scores_changed: bool,
    pub margin_changed: bool,
    pub vote_changed: bool,
    pub verdict_changed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelabelCheck {
    pub seed: u64,
    pub beliefs_equivariant: bool,
    pub votes_equivariant: bool,
    pub verdict_equivariant: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub runs: usize,
    pub replay_matches: usize,
    pub fixtures: Vec<FixtureEvidence>,
    pub pairs: Vec<Pair>,
    pub relabel_checks: Vec<RelabelCheck>,
}
fn evidence(evaluation: &Evaluation, fixture: Fixture) -> &FixtureEvidence {
    evaluation
        .fixtures
        .iter()
        .find(|e| e.fixture == fixture)
        .unwrap()
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut evaluation = Evaluation {
        version: "evaluation-003-v1".into(),
        runs: 0,
        replay_matches: 0,
        fixtures: vec![],
        pairs: vec![],
        relabel_checks: vec![],
    };
    for fixture in FIXTURES {
        let mut result = FixtureEvidence {
            fixture,
            counts: Counts::default(),
            reports: vec![],
        };
        for seed in 0..16 {
            let report = run(fixture_config(fixture, seed))?;
            let replay = run(fixture_config(fixture, seed))?;
            if serde_json::to_vec(&report).map_err(|e| e.to_string())?
                != serde_json::to_vec(&replay).map_err(|e| e.to_string())?
            {
                return Err("003 serialized replay mismatch".into());
            }
            evaluation.runs += 1;
            evaluation.replay_matches += 1;
            result.counts.add(&report);
            result.reports.push(report);
        }
        evaluation.fixtures.push(result);
    }
    let comparisons = [
        (Fixture::TruthRetain, Fixture::Truth),
        (Fixture::Truth, Fixture::OriginalVote),
        (Fixture::Truth, Fixture::ReverseOrder),
        (Fixture::Absent, Fixture::OneTruth),
        (Fixture::Absent, Fixture::OneMistake),
        (Fixture::Absent, Fixture::OneLie),
        (Fixture::Absent, Fixture::Conflict),
        (Fixture::Absent, Fixture::Repeat),
        (Fixture::Absent, Fixture::Uncertain),
        (Fixture::OneTruth, Fixture::Repeat),
        (Fixture::HiddenTruth, Fixture::HiddenContradiction),
    ];
    for (control, treatment) in comparisons {
        let mut pairs = vec![];
        for (b, a) in evidence(&evaluation, control)
            .reports
            .iter()
            .zip(&evidence(&evaluation, treatment).reports)
        {
            let before = judgment(b, 5);
            let after = judgment(a, 5);
            pairs.push(Pair {
                seed: b.config.seed,
                control,
                treatment,
                recipient: 5,
                matched_original_state: before.input.original == after.input.original,
                identical_final_listener_inputs: b.updates[before.update].input
                    == a.updates[after.update].input,
                candidate_changed: before.input.current.candidate != after.input.current.candidate,
                scores_changed: before.input.current.scores != after.input.current.scores,
                margin_changed: before.input.current.margin != after.input.current.margin,
                vote_changed: before.choice.action != after.choice.action,
                verdict_changed: b.verdict != a.verdict,
            });
        }
        evaluation.pairs.extend(pairs);
    }
    let mut checks = vec![];
    for (base, mapped) in evidence(&evaluation, Fixture::Truth)
        .reports
        .iter()
        .zip(&evidence(&evaluation, Fixture::Relabeled).reports)
    {
        let layout = mapped.config.layout;
        let mut beliefs_equivariant = true;
        let mut votes_equivariant = true;
        for id in 0..N {
            let b = judgment(base, id);
            let a = judgment(mapped, layout[usize::from(id)]);
            beliefs_equivariant &= b.input.current.candidate.map(|c| layout[usize::from(c)])
                == a.input.current.candidate;
            beliefs_equivariant &= (0..N).all(|c| {
                b.input.current.scores[usize::from(c)]
                    == a.input.current.scores[usize::from(layout[usize::from(c)])]
            });
            let Action::Vote(bv) = b.choice.action else {
                unreachable!()
            };
            let Action::Vote(av) = a.choice.action else {
                unreachable!()
            };
            votes_equivariant &= bv.map(|c| layout[usize::from(c)]) == av;
        }
        checks.push(RelabelCheck {
            seed: base.config.seed,
            beliefs_equivariant,
            votes_equivariant,
            verdict_equivariant: base.verdict.map(|c| layout[usize::from(c)]) == mapped.verdict,
        });
    }
    evaluation.relabel_checks = checks;
    Ok(evaluation)
}
