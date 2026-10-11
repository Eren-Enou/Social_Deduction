use super::*;
use crate::{Phase, experiment005 as e5, experiment007 as e7};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub fn canonical_digest<T: Serialize>(value: &T) -> Result<String, String> {
    let mut bytes = b"canonical-010-v1\n".to_vec();
    bytes.extend(e7::canonical_bytes(value)?);
    Ok(e7::sha256(&bytes))
}
pub fn report_digests(r: &Report) -> Result<BTreeMap<String, String>, String> {
    let mut d = BTreeMap::new();
    d.insert("full_series".into(), canonical_digest(r)?);
    d.insert(
        "events_resources".into(),
        canonical_digest(&(&r.objective_history, &r.ledgers))?,
    );
    d.insert(
        "acquisitions_forecasts".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| (&e.investigation, &e.deliveries))
                .collect::<Vec<_>>(),
        )?,
    );
    d.insert(
        "method_memory_feedback".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| {
                    (
                        &e.method_before,
                        &e.method_ingestions,
                        &e.indirect_ingestions,
                        &e.method_feedback,
                        &e.method_after,
                    )
                })
                .collect::<Vec<_>>(),
        )?,
    );
    d.insert(
        "interpersonal_cognition".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| {
                    (
                        &e.memory_before,
                        &e.cognition,
                        &e.ingestions,
                        &e.verification,
                        &e.memory_after,
                    )
                })
                .collect::<Vec<_>>(),
        )?,
    );
    d.insert(
        "communication".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| (&e.communication, &e.reports, &e.ancestry))
                .collect::<Vec<_>>(),
        )?,
    );
    d.insert(
        "incidents_ballots_outcomes".into(),
        canonical_digest(&r.episodes.iter().map(|e| &e.incident).collect::<Vec<_>>())?,
    );
    Ok(d)
}
pub fn environment_digest(c: &Config) -> Result<String, String> {
    let mut c = c.clone();
    c.policy = [PolicyKind::NeverInvestigate; 6];
    c.feedback.clear();
    for i in &mut c.incidents {
        i.forced.clear();
        i.injected.clear();
    }
    canonical_digest(&c)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObserverForecastLabel {
    pub sealed_result: Option<u8>,
    pub correct: Option<bool>,
    pub missing: bool,
    pub actual_root: Root,
    pub process_accuracy: Probability,
    pub process_missing: Probability,
    pub after_shift: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastRecord {
    pub episode: u8,
    pub observer: u8,
    pub local: MethodExpectation,
    pub effective_nonmissing: Probability,
    pub affordable: bool,
    pub inspected: bool,
    pub voluntary: bool,
    pub injected: bool,
    pub observer_label: ObserverForecastLabel,
}
pub fn forecasts(r: &Report) -> Result<Vec<ForecastRecord>, String> {
    let mut records = vec![];
    for (episode, e) in r.episodes.iter().enumerate() {
        for d in &e.investigation {
            for local in &d.choice.expectations {
                let source = r.config.incidents[episode]
                    .artifacts
                    .iter()
                    .find(|s| s.descriptor.id == local.source)
                    .ok_or("forecast references absent artifact")?;
                let method = r
                    .config
                    .processes
                    .iter()
                    .find(|p| p.family == local.family)
                    .ok_or("forecast references absent process")?;
                let inspected = d.delivery.is_some()
                    && matches!(d.choice.action,InvestigationAction::Inspect(id)if id==local.source);
                records.push(ForecastRecord {
                    episode: e.id,
                    observer: d.input.local.id,
                    local: local.clone(),
                    effective_nonmissing: Probability {
                        numerator: local.effective_missing.denominator
                            - local.effective_missing.numerator,
                        denominator: local.effective_missing.denominator,
                    },
                    affordable: source.cost <= d.input.local.balance,
                    inspected,
                    voluntary: inspected && !d.forced,
                    injected: e.deliveries.iter().any(|f| {
                        f.finding.acquisition.observer == d.input.local.id
                            && f.finding.source.id == local.source
                            && matches!(f.origin, DeliveryOrigin::Intervention(_))
                    }),
                    observer_label: ObserverForecastLabel {
                        sealed_result: source.candidate,
                        correct: source.candidate.map(|c| c == e.incident.config.actor()),
                        missing: source.candidate.is_none(),
                        actual_root: source.actual_root,
                        process_accuracy: method.accuracy(e.id),
                        process_missing: method.missingness(e.id),
                        after_shift: method.shift_episode.is_some_and(|episode| e.id >= episode),
                    },
                });
            }
        }
    }
    Ok(records)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictorKind {
    Neutral,
    Advertised,
    Experienced,
    Effective,
    ObserverProcess,
}
pub const PREDICTORS: [PredictorKind; 5] = [
    PredictorKind::Neutral,
    PredictorKind::Advertised,
    PredictorKind::Experienced,
    PredictorKind::Effective,
    PredictorKind::ObserverProcess,
];
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub samples: u64,
    pub brier_sum: u64,
    pub bins: [e7::Bin; 5],
}
impl Score {
    pub fn add(&mut self, p: Probability, label: bool) {
        self.samples += 1;
        self.brier_sum += p.brier(label);
        let b = &mut self.bins[p.bin()];
        b.samples += 1;
        b.forecast_sum_ppm += p.ppm();
        b.correct += u64::from(label);
    }
    pub fn merge(&mut self, s: &Self) {
        self.samples += s.samples;
        self.brier_sum += s.brier_sum;
        for (a, b) in self.bins.iter_mut().zip(&s.bins) {
            a.samples += b.samples;
            a.forecast_sum_ppm += b.forecast_sum_ppm;
            a.correct += b.correct;
        }
    }
    pub fn brier(&self) -> Option<u64> {
        self.brier_sum.checked_div(self.samples)
    }
    pub fn ece(&self) -> Option<u64> {
        self.bins
            .iter()
            .map(|b| b.forecast_sum_ppm.abs_diff(b.correct * 1_000_000))
            .sum::<u64>()
            .checked_div(self.samples)
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityScore {
    pub accuracy: Score,
    pub missingness: Score,
}
impl QualityScore {
    pub fn merge(&mut self, s: &Self) {
        self.accuracy.merge(&s.accuracy);
        self.missingness.merge(&s.missingness);
    }
}
pub fn forecast_score(
    records: &[ForecastRecord],
    predictor: PredictorKind,
    selected: bool,
    assessment: bool,
    phase: Option<bool>,
    count: Option<u32>,
) -> QualityScore {
    forecast_score_scope(
        records, predictor, selected, false, assessment, phase, count,
    )
}
pub fn forecast_score_scope(
    records: &[ForecastRecord],
    predictor: PredictorKind,
    selected: bool,
    uninspected: bool,
    assessment: bool,
    phase: Option<bool>,
    count: Option<u32>,
) -> QualityScore {
    let mut s = QualityScore::default();
    for f in records {
        if (selected && !f.voluntary)
            || (uninspected && (f.inspected || f.injected))
            || (assessment && f.episode < 4)
            || phase.is_some_and(|after| f.observer_label.after_shift != after)
        {
            continue;
        }
        let (accuracy, missing) = match predictor {
            PredictorKind::Neutral => (
                Probability {
                    numerator: 1,
                    denominator: 2,
                },
                Probability {
                    numerator: 1,
                    denominator: 2,
                },
            ),
            PredictorKind::Advertised => (f.local.advertised_accuracy, f.local.advertised_missing),
            PredictorKind::Experienced => (f.local.learned_accuracy, f.local.learned_missing),
            PredictorKind::Effective => (f.local.effective_accuracy, f.local.effective_missing),
            PredictorKind::ObserverProcess => (
                f.observer_label.process_accuracy,
                f.observer_label.process_missing,
            ),
        };
        if count.is_none_or(|n| f.local.counts.graded().min(4) == n)
            && let Some(correct) = f.observer_label.correct
        {
            s.accuracy.add(accuracy, correct);
        }
        if count.is_none_or(|n| f.local.counts.observed().min(4) == n) {
            s.missingness.add(missing, f.observer_label.missing);
        }
    }
    s
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceTrace {
    pub observer: u8,
    pub input_digest: String,
    pub prior: e5::Belief,
    pub balance: u8,
    pub choice: InvestigationChoice,
    pub forced: bool,
    pub proposal: Option<InvestigationChoice>,
    pub rejection: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningTrace {
    pub observer: u8,
    pub acquisition: Acquisition,
    pub method: MethodId,
    pub origin: DeliveryOrigin,
    pub eligible: bool,
    pub candidate: Option<u8>,
    pub before: Counts,
    pub after: Counts,
    pub evicted: Vec<DirectExperience>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackTrace {
    pub observer: u8,
    pub plan: usize,
    pub received: e7::Token,
    pub acceptance: e7::Acceptance,
    pub factual_accuracy: Option<bool>,
    pub before: BTreeMap<MethodId, Counts>,
    pub after: BTreeMap<MethodId, Counts>,
    pub unverifiable_eviction: bool,
    pub evicted_tokens: Vec<e7::TokenEntry>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeTrace {
    pub id: u8,
    pub actor: u8,
    pub choices: Vec<ChoiceTrace>,
    pub learning: Vec<LearningTrace>,
    pub feedback: Vec<FeedbackTrace>,
    pub deliveries: Vec<crate::experiment009::DeliveryTrace>,
    pub messages: Vec<e5::Testimony>,
    pub ballots: [Option<u8>; 6],
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
}
pub fn traces(r: &Report) -> Result<Vec<EpisodeTrace>, String> {
    r.episodes
        .iter()
        .map(|e| {
            let choices = e
                .investigation
                .iter()
                .map(|d| {
                    Ok(ChoiceTrace {
                        observer: d.input.local.id,
                        input_digest: canonical_digest(&d.input)?,
                        prior: d.input.local.belief.clone(),
                        balance: d.input.local.balance,
                        choice: d.choice.clone(),
                        forced: d.forced,
                        proposal: d.proposal.clone(),
                        rejection: d.rejection.clone(),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let learning = e
                .method_ingestions
                .iter()
                .map(|i| LearningTrace {
                    observer: i.after.observer,
                    acquisition: i.input.finding.acquisition,
                    method: i.input.method.clone(),
                    origin: i.input.origin,
                    eligible: !i.duplicate,
                    candidate: i.input.finding.candidate,
                    before: i
                        .input
                        .memory
                        .summaries
                        .get(&i.input.method)
                        .copied()
                        .unwrap_or_default(),
                    after: i
                        .after
                        .summaries
                        .get(&i.input.method)
                        .copied()
                        .unwrap_or_default(),
                    evicted: i.evicted.clone(),
                })
                .collect();
            let feedback = e
                .method_feedback
                .iter()
                .map(|f| FeedbackTrace {
                    observer: f.observer,
                    plan: f.plan,
                    received: f.effect.input.received.clone(),
                    acceptance: f.effect.transport_effect.acceptance,
                    factual_accuracy: f.correct,
                    before: f.effect.input.memory.summaries.clone(),
                    after: f.effect.after.summaries.clone(),
                    unverifiable_eviction: f.effect.transport_effect.unverifiable_eviction,
                    evicted_tokens: f.effect.transport_effect.evicted_tokens.clone(),
                })
                .collect();
            let deliveries = e
                .deliveries
                .iter()
                .map(|d| crate::experiment009::DeliveryTrace {
                    origin: d.origin,
                    finding: d.finding.clone(),
                    root: d.actual_root,
                    correct: d.correct,
                    before: d.before_assessment.after.clone(),
                    after: d.after_assessment.after.clone(),
                    balances: (d.balance_before, d.balance_after),
                })
                .collect();
            let mut ballots = [None; 6];
            for d in &e.incident.decisions {
                if d.input.phase == Phase::Judgment
                    && d.rejection.is_none()
                    && let e5::Action::Vote(candidate) = d.choice.action
                {
                    ballots[usize::from(d.input.id)] = candidate;
                }
            }
            Ok(EpisodeTrace {
                id: e.id,
                actor: e.incident.config.actor(),
                choices,
                learning,
                feedback,
                deliveries,
                messages: e.incident.testimony.clone(),
                ballots,
                verdict: e.incident.verdict,
                correct: e.incident.correct,
            })
        })
        .collect()
}
