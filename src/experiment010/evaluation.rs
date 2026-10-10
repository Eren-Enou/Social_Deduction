use super::*;
use crate::{Rng, experiment007 as e7, experiment009 as e9};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const LEARNERS: [PolicyKind; 3] = [
    PolicyKind::ExperienceAdjusted,
    PolicyKind::ConservativeLearner,
    PolicyKind::ExploratoryLearner,
];
pub const MATCHED_POLICIES: [PolicyKind; 4] = [
    PolicyKind::Frozen009InformationValue,
    PolicyKind::ExperienceAdjusted,
    PolicyKind::ConservativeLearner,
    PolicyKind::ExploratoryLearner,
];
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub direct: u64,
    pub duplicates: u64,
    pub indirect: u64,
    pub evicted: u64,
    pub feedback: u64,
    pub false_feedback: u64,
    pub feedback_changes: u64,
    pub unverifiable_eviction: u64,
    pub exploration: u64,
    pub exploration_spent: u64,
    pub exploration_wrong: u64,
    pub exploration_missing: u64,
    pub exploration_null: u64,
    pub final_success: u64,
    pub final_failure: u64,
    pub final_nonmissing: u64,
    pub final_missing: u64,
    pub checkpoint_graded: u64,
    pub method_memory_bytes: u64,
    pub acceptance: BTreeMap<String, u64>,
}
impl LearningMetrics {
    fn collect(r: &Report) -> Result<Self, String> {
        let mut m = Self::default();
        for e in &r.episodes {
            for i in &e.method_ingestions {
                m.direct += 1;
                m.duplicates += u64::from(i.duplicate);
                m.evicted += i.evicted.len() as u64;
            }
            m.indirect += e.indirect_ingestions.len() as u64;
            for f in &e.method_feedback {
                m.feedback += 1;
                m.false_feedback += u64::from(f.correct == Some(false));
                m.feedback_changes +=
                    u64::from(f.effect.input.memory.summaries != f.effect.after.summaries);
                m.unverifiable_eviction +=
                    u64::from(f.effect.transport_effect.unverifiable_eviction);
                *m.acceptance
                    .entry(format!("{:?}", f.effect.transport_effect.acceptance))
                    .or_default() += 1;
            }
            for d in &e.investigation {
                if d.choice.exploration
                    && !d.forced
                    && let Some(index) = d.delivery
                {
                    let f = &e.deliveries[index];
                    m.exploration += 1;
                    m.exploration_spent += u64::from(f.balance_before - f.balance_after);
                    m.exploration_wrong += u64::from(
                        f.finding
                            .candidate
                            .is_some_and(|c| c != e.incident.config.actor()),
                    );
                    m.exploration_missing += u64::from(f.finding.candidate.is_none());
                    m.exploration_null +=
                        u64::from(f.after_assessment.after == f.before_assessment.after);
                }
            }
        }
        for memory in &r.final_method_memory {
            m.method_memory_bytes +=
                serde_json::to_vec(memory).map_err(|e| e.to_string())?.len() as u64;
            for c in memory.summaries.values() {
                m.final_success += u64::from(c.success);
                m.final_failure += u64::from(c.failure);
                m.final_nonmissing += u64::from(c.nonmissing);
                m.final_missing += u64::from(c.missing);
            }
            m.checkpoint_graded += memory
                .checkpoint
                .values()
                .map(|c| u64::from(c.graded()))
                .sum::<u64>();
        }
        Ok(m)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub population: String,
    pub family: String,
    pub seed: u64,
    pub policy: PolicyKind,
    pub stream: FeedbackStream,
    pub shifted: bool,
    pub metrics: e9::Metrics,
    pub assessment: e9::Metrics,
    pub learning: LearningMetrics,
    pub forecasts: Vec<ForecastRecord>,
    pub trace: Vec<EpisodeTrace>,
    pub digests: BTreeMap<String, String>,
    pub environment_digest: String,
}
impl Row {
    fn window(&self, assessment: bool) -> &e9::Metrics {
        if assessment {
            &self.assessment
        } else {
            &self.metrics
        }
    }
}
fn row(
    population: &str,
    family: String,
    seed: u64,
    policy: PolicyKind,
    stream: FeedbackStream,
    c: Config,
) -> Result<Row, String> {
    let environment_digest = environment_digest(&c)?;
    let shifted = c.processes.iter().any(|p| p.shift_episode.is_some());
    let a = run(c.clone())?;
    let b = run(c)?;
    let digests = report_digests(&a)?;
    if a != b || digests != report_digests(&b)? {
        return Err("010 full causal replay mismatch".into());
    }
    Ok(Row {
        population: population.into(),
        family,
        seed,
        policy,
        stream,
        shifted,
        metrics: incident_metrics(&a, false)?,
        assessment: incident_metrics(&a, true)?,
        learning: LearningMetrics::collect(&a)?,
        forecasts: forecasts(&a)?,
        trace: traces(&a)?,
        digests,
        environment_digest,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    pub mean: Option<i64>,
    pub lower: Option<i64>,
    pub upper: Option<i64>,
    pub worlds: usize,
    pub defined_resamples: usize,
}
pub fn paired_interval(values: &[i64]) -> Interval {
    if values.is_empty() {
        return Interval {
            mean: None,
            lower: None,
            upper: None,
            worlds: 0,
            defined_resamples: 0,
        };
    }
    let mut rng = Rng::new(10010);
    let mut samples = Vec::with_capacity(512);
    for _ in 0..512 {
        let sum: i64 = (0..values.len())
            .map(|_| values[(rng.draw() % values.len() as u64) as usize])
            .sum();
        samples.push(sum / values.len() as i64);
    }
    samples.sort();
    Interval {
        mean: Some(values.iter().sum::<i64>() / values.len() as i64),
        lower: Some(samples[12]),
        upper: Some(samples[499]),
        worlds: values.len(),
        defined_resamples: 512,
    }
}
fn delta(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    a.zip(b).map(|(a, b)| a - b)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utility {
    pub wrong_cost: i64,
    pub effort_price_millionths: i64,
    pub difference: Interval,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    pub policy: PolicyKind,
    pub baseline: PolicyKind,
    pub stream: FeedbackStream,
    pub regime: String,
    pub assessment: bool,
    pub worlds: usize,
    pub improved: usize,
    pub worsened: usize,
    pub unchanged: usize,
    pub communication_changed: usize,
    pub ballots_changed: usize,
    pub metrics: e9::Metrics,
    pub baseline_metrics: e9::Metrics,
    pub coverage: Interval,
    pub conditional_accuracy: Interval,
    pub wrong_risk: Interval,
    pub spent: Interval,
    pub correct_ballots: Interval,
    pub correct_verdicts: Interval,
    pub utilities: Vec<Utility>,
    pub verdict_transitions: BTreeMap<String, usize>,
}
fn cohort(r: &Row, stream: FeedbackStream, regime: &str) -> bool {
    r.population == "heldout"
        && r.stream == stream
        && match regime {
            "stationary" => !r.shifted,
            "shifted" => r.shifted,
            _ => true,
        }
}
fn comparison(
    rows: &[Row],
    policy: PolicyKind,
    baseline: PolicyKind,
    stream: FeedbackStream,
    regime: &str,
    assessment: bool,
) -> Result<Comparison, String> {
    let pairs: Vec<_> = rows
        .iter()
        .filter(|r| cohort(r, stream, regime) && r.policy == policy)
        .map(|a| {
            let b = rows
                .iter()
                .find(|b| cohort(b, stream, regime) && b.policy == baseline && b.seed == a.seed)
                .ok_or("missing paired baseline")?;
            if a.environment_digest != b.environment_digest {
                return Err("unmatched latent worlds".into());
            }
            Ok((a, b))
        })
        .collect::<Result<_, String>>()?;
    let mut a = e9::Metrics::default();
    let mut b = e9::Metrics::default();
    let mut improved = 0;
    let mut worsened = 0;
    let mut communication_changed = 0;
    let mut ballots_changed = 0;
    let mut verdict_transitions = BTreeMap::new();
    for (x, y) in &pairs {
        let xm = x.window(assessment);
        let ym = y.window(assessment);
        a.add(xm);
        b.add(ym);
        improved += usize::from(xm.correct_ballots > ym.correct_ballots);
        worsened += usize::from(xm.correct_ballots < ym.correct_ballots);
        let xt: Vec<_> = x
            .trace
            .iter()
            .filter(|t| !assessment || t.id >= 4)
            .collect();
        let yt: Vec<_> = y
            .trace
            .iter()
            .filter(|t| !assessment || t.id >= 4)
            .collect();
        communication_changed += usize::from(
            xt.iter()
                .map(|t| &t.messages)
                .ne(yt.iter().map(|t| &t.messages)),
        );
        ballots_changed += usize::from(
            xt.iter()
                .map(|t| t.ballots)
                .ne(yt.iter().map(|t| t.ballots)),
        );
        for (xt, yt) in xt.iter().zip(yt) {
            *verdict_transitions
                .entry(format!("{:?}->{:?}", yt.correct, xt.correct))
                .or_default() += 1;
        }
    }
    let interval = |f: fn(&e9::Metrics) -> Option<i64>| {
        paired_interval(
            &pairs
                .iter()
                .filter_map(|(x, y)| delta(f(x.window(assessment)), f(y.window(assessment))))
                .collect::<Vec<_>>(),
        )
    };
    let mut utilities = vec![];
    for wrong_cost in [1, 2, 4] {
        for effort_price_millionths in [0, 250_000, 1_000_000] {
            let value = |m: &e9::Metrics| {
                (m.correct_verdicts as i64 - wrong_cost * m.wrong_verdicts as i64) * 1_000_000
                    - effort_price_millionths * m.spent as i64
            };
            utilities.push(Utility {
                wrong_cost,
                effort_price_millionths,
                difference: paired_interval(
                    &pairs
                        .iter()
                        .map(|(x, y)| value(x.window(assessment)) - value(y.window(assessment)))
                        .collect::<Vec<_>>(),
                ),
            });
        }
    }
    Ok(Comparison {
        policy,
        baseline,
        stream,
        regime: regime.into(),
        assessment,
        worlds: pairs.len(),
        improved,
        worsened,
        unchanged: pairs.len() - improved - worsened,
        communication_changed,
        ballots_changed,
        coverage: interval(e9::Metrics::coverage),
        conditional_accuracy: interval(e9::Metrics::accuracy),
        wrong_risk: interval(e9::Metrics::wrong_risk),
        spent: interval(|m| Some(m.spent as i64 * 1_000_000)),
        correct_ballots: interval(|m| Some(m.correct_ballots as i64 * 1_000_000)),
        correct_verdicts: interval(|m| Some(m.correct_verdicts as i64 * 1_000_000)),
        metrics: a,
        baseline_metrics: b,
        utilities,
        verdict_transitions,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionGroup {
    pub policy: PolicyKind,
    pub stream: FeedbackStream,
    pub regime: String,
    pub assessment: bool,
    pub selected_only: bool,
    pub uninspected_only: bool,
    pub phase: Option<bool>,
    /// Accuracy uses graded counts; missingness uses directly observed counts, independently.
    pub experience_count_bin: Option<u32>,
    pub predictor: PredictorKind,
    pub score: QualityScore,
    pub accuracy_world_mean: Interval,
    pub missing_world_mean: Interval,
    pub accuracy_delta_neutral: Interval,
    pub accuracy_delta_advertised: Interval,
    pub missing_delta_neutral: Interval,
    pub missing_delta_advertised: Interval,
}
fn predictions(rows: &[Row]) -> Vec<PredictionGroup> {
    let mut groups = vec![];
    for policy in POLICIES {
        for stream in [FeedbackStream::Clean, FeedbackStream::MixedFault] {
            for regime in ["all", "stationary", "shifted"] {
                let rs: Vec<_> = rows
                    .iter()
                    .filter(|r| cohort(r, stream, regime) && r.policy == policy)
                    .collect();
                for assessment in [false, true] {
                    for (selected_only, uninspected_only) in
                        [(false, false), (true, false), (false, true)]
                    {
                        for (phase, count) in [
                            (None, None),
                            (Some(false), None),
                            (Some(true), None),
                            (None, Some(0)),
                            (None, Some(1)),
                            (None, Some(2)),
                            (None, Some(3)),
                            (None, Some(4)),
                        ] {
                            for predictor in PREDICTORS {
                                let mut score = QualityScore::default();
                                let mut av = vec![];
                                let mut mv = vec![];
                                let mut an = vec![];
                                let mut aa = vec![];
                                let mut mn = vec![];
                                let mut ma = vec![];
                                for r in &rs {
                                    let s = forecast_score_scope(
                                        &r.forecasts,
                                        predictor,
                                        selected_only,
                                        uninspected_only,
                                        assessment,
                                        phase,
                                        count,
                                    );
                                    let n = forecast_score_scope(
                                        &r.forecasts,
                                        PredictorKind::Neutral,
                                        selected_only,
                                        uninspected_only,
                                        assessment,
                                        phase,
                                        count,
                                    );
                                    let ad = forecast_score_scope(
                                        &r.forecasts,
                                        PredictorKind::Advertised,
                                        selected_only,
                                        uninspected_only,
                                        assessment,
                                        phase,
                                        count,
                                    );
                                    score.merge(&s);
                                    let accuracy = s.accuracy.brier().map(|v| v as i64);
                                    let missing = s.missingness.brier().map(|v| v as i64);
                                    if let Some(v) = accuracy {
                                        av.push(v);
                                    }
                                    if let Some(v) = missing {
                                        mv.push(v);
                                    }
                                    if let Some(v) =
                                        delta(accuracy, n.accuracy.brier().map(|v| v as i64))
                                    {
                                        an.push(v);
                                    }
                                    if let Some(v) =
                                        delta(accuracy, ad.accuracy.brier().map(|v| v as i64))
                                    {
                                        aa.push(v);
                                    }
                                    if let Some(v) =
                                        delta(missing, n.missingness.brier().map(|v| v as i64))
                                    {
                                        mn.push(v);
                                    }
                                    if let Some(v) =
                                        delta(missing, ad.missingness.brier().map(|v| v as i64))
                                    {
                                        ma.push(v);
                                    }
                                }
                                groups.push(PredictionGroup {
                                    policy,
                                    stream,
                                    regime: regime.into(),
                                    assessment,
                                    selected_only,
                                    uninspected_only,
                                    phase,
                                    experience_count_bin: count,
                                    predictor,
                                    score,
                                    accuracy_world_mean: paired_interval(&av),
                                    missing_world_mean: paired_interval(&mv),
                                    accuracy_delta_neutral: paired_interval(&an),
                                    accuracy_delta_advertised: paired_interval(&aa),
                                    missing_delta_neutral: paired_interval(&mn),
                                    missing_delta_advertised: paired_interval(&ma),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    groups
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub independent_worlds: usize,
    pub incidents: usize,
    pub replay_matches: usize,
    pub rows: Vec<Row>,
    pub comparisons: Vec<Comparison>,
    pub predictions: Vec<PredictionGroup>,
}
pub fn evaluate() -> Result<Evaluation, String> {
    evaluate_seeds(4000..4128, 0..4)
}
/// Engineering-sized runs use development seeds only and cannot be represented as the primary grid.
pub fn evaluate_seeds(
    heldout: std::ops::Range<u64>,
    diagnostic: std::ops::Range<u64>,
) -> Result<Evaluation, String> {
    let independent_worlds = (heldout.end - heldout.start) as usize;
    let primary = heldout == (4000..4128) && diagnostic == (0..4);
    let mut rows = vec![];
    for seed in heldout {
        let world = Generator.generate(seed);
        for stream in [FeedbackStream::Clean, FeedbackStream::MixedFault] {
            for policy in POLICIES {
                rows.push(row(
                    "heldout",
                    "Generated".into(),
                    seed,
                    policy,
                    stream,
                    generated_config(&world, policy, stream),
                )?);
            }
        }
    }
    for seed in diagnostic {
        for family in DIAGNOSTICS {
            for policy in POLICIES {
                rows.push(row(
                    "diagnostic",
                    format!("{family:?}"),
                    seed,
                    policy,
                    FeedbackStream::Clean,
                    diagnostic_config(family, seed, policy, false),
                )?);
            }
            for policy in MATCHED_POLICIES {
                rows.push(row(
                    "matched",
                    format!("{family:?}"),
                    seed,
                    policy,
                    FeedbackStream::Clean,
                    diagnostic_config(family, seed, policy, true),
                )?);
            }
        }
        for corrupt in [false, true] {
            for policy in MATCHED_POLICIES {
                for (variant, c) in fixed_current_configs(seed, policy, corrupt)
                    .into_iter()
                    .enumerate()
                {
                    rows.push(row(
                        "fixed_current",
                        format!(
                            "{}-{variant}",
                            if corrupt { "feedback" } else { "acquisition" }
                        ),
                        seed,
                        policy,
                        FeedbackStream::Clean,
                        c,
                    )?);
                }
            }
        }
    }
    let mut comparisons = vec![];
    for policy in LEARNERS {
        for baseline in POLICIES.into_iter().filter(|p| !LEARNERS.contains(p)) {
            for stream in [FeedbackStream::Clean, FeedbackStream::MixedFault] {
                for regime in ["all", "stationary", "shifted"] {
                    for assessment in [false, true] {
                        comparisons.push(comparison(
                            &rows, policy, baseline, stream, regime, assessment,
                        )?);
                    }
                }
            }
        }
    }
    let predictions = predictions(&rows);
    let incidents = rows.iter().map(|r| r.trace.len()).sum();
    Ok(Evaluation {
        version: if primary {
            "evaluation-010-v1"
        } else {
            "development-evaluation-010-v1"
        }
        .into(),
        independent_worlds,
        incidents,
        replay_matches: rows.len(),
        rows,
        comparisons,
        predictions,
    })
}
pub fn manifest(e: &Evaluation) -> Result<e7::ReplayManifest, String> {
    Ok(e7::ReplayManifest {
        version: "replay-manifest-010-v1".into(),
        series: e.rows.len(),
        incidents: e.incidents,
        evaluation_digest: canonical_digest(e)?,
        metric_digest: canonical_digest(&(&e.comparisons, &e.predictions))?,
        series_sections: e.rows.iter().map(|r| r.digests.clone()).collect(),
    })
}
