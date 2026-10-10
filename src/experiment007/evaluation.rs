use super::*;
use crate::{Phase, Rng, experiment005 as e5, experiment006 as e6};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reliability {
    pub worlds: u64,
    pub eligible: u64,
    pub correct_votes: u64,
    pub wrong_votes: u64,
    pub abstentions: u64,
    pub correct_verdicts: u64,
    pub wrong_verdicts: u64,
    pub unresolved: u64,
}
impl Reliability {
    fn from_report(r: &Report) -> Self {
        let last = &r.episodes.last().unwrap().incident;
        let mut c = Self {
            worlds: 1,
            eligible: 6,
            ..Self::default()
        };
        match last.correct {
            Some(true) => c.correct_verdicts = 1,
            Some(false) => c.wrong_verdicts = 1,
            None => c.unresolved = 1,
        }
        for d in last
            .decisions
            .iter()
            .filter(|d| d.input.phase == Phase::Judgment)
        {
            match d.choice.action {
                e5::Action::Vote(Some(id)) if id == last.config.actor() => c.correct_votes += 1,
                e5::Action::Vote(Some(_)) => c.wrong_votes += 1,
                e5::Action::Vote(None) => c.abstentions += 1,
                _ => {}
            }
        }
        c
    }
    fn add(&mut self, c: &Self) {
        self.worlds += c.worlds;
        self.eligible += c.eligible;
        self.correct_votes += c.correct_votes;
        self.wrong_votes += c.wrong_votes;
        self.abstentions += c.abstentions;
        self.correct_verdicts += c.correct_verdicts;
        self.wrong_verdicts += c.wrong_verdicts;
        self.unresolved += c.unresolved;
    }
    pub fn metrics(&self) -> BTreeMap<String, Option<i64>> {
        let mut m = BTreeMap::new();
        for (name, n, d) in [
            ("vote_correct_all", self.correct_votes, self.eligible),
            ("vote_wrong_all", self.wrong_votes, self.eligible),
            (
                "vote_coverage",
                self.correct_votes + self.wrong_votes,
                self.eligible,
            ),
            (
                "vote_accuracy",
                self.correct_votes,
                self.correct_votes + self.wrong_votes,
            ),
            ("verdict_correct_all", self.correct_verdicts, self.worlds),
            ("verdict_wrong_all", self.wrong_verdicts, self.worlds),
            (
                "verdict_coverage",
                self.correct_verdicts + self.wrong_verdicts,
                self.worlds,
            ),
            (
                "verdict_accuracy",
                self.correct_verdicts,
                self.correct_verdicts + self.wrong_verdicts,
            ),
        ] {
            m.insert(name.into(), ratio(n * UNITS, d).map(|n| n as i64));
        }
        for cost in [1, 2, 4] {
            m.insert(
                format!("utility_wrong_cost_{cost}"),
                if self.worlds == 0 {
                    None
                } else {
                    Some(
                        (self.correct_verdicts as i64 - cost * self.wrong_verdicts as i64)
                            * UNITS as i64
                            / self.worlds as i64,
                    )
                },
            );
        }
        m
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabeledForecast {
    pub forecast: Forecast,
    pub message: Option<usize>,
    pub correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackTrace {
    pub after_episode: u8,
    pub actual_about: u8,
    pub source: String,
    pub fault: String,
    pub actual_actor: u8,
    pub factual_accuracy: Option<bool>,
    pub authentic: bool,
    pub recipient: u8,
    pub token: Token,
    pub acceptance: Acceptance,
    pub credits_before: [e6::Credit; 6],
    pub credits_after: [e6::Credit; 6],
    pub record_keys: Vec<e6::Reference>,
    pub receipts_after: Vec<Receipt>,
    pub unverifiable_eviction: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionTrace {
    pub id: u8,
    pub phase: Phase,
    pub belief: e5::Belief,
    pub choice: e5::Choice,
    pub rejection: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentTrace {
    pub episode: u8,
    pub actor: u8,
    pub observations: [Option<u8>; 6],
    pub later: Vec<e5::LaterObservation>,
    pub testimony: Vec<e5::Testimony>,
    pub decisions: Vec<DecisionTrace>,
    pub final_cognition: Vec<e6::Cognition>,
    pub evictions: Vec<(u8, Vec<e6::Reference>)>,
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub family: String,
    pub seed: u64,
    pub generated: bool,
    pub changed: bool,
    pub corrections: bool,
    pub acceptance: VerificationKind,
    pub listener: e6::ListenerKind,
    pub incident_count: usize,
    pub digests: BTreeMap<String, String>,
    pub replay_matched: bool,
    pub rejected: usize,
    pub reliability: Reliability,
    pub forecasts: Vec<LabeledForecast>,
    pub world_losses: Option<[u64; 2]>,
    pub history: Vec<IncidentTrace>,
    pub feedback: Vec<FeedbackTrace>,
}
fn row(family: String, seed: u64, world: Option<&World>, config: Config) -> Result<Row, String> {
    let r = run(config.clone())?;
    let digests = report_digests(&r)?;
    if digests != report_digests(&run(config.clone())?)? {
        return Err("007 canonical replay mismatch".into());
    }
    let last = &r.episodes.last().unwrap().incident;
    let forecasts: Vec<LabeledForecast> = r
        .forecasts
        .iter()
        .map(|forecast| {
            let message = last.testimony.iter().find(|m| {
                m.message.speaker == forecast.input.source
                    && matches!(m.message.body, e5::Body::Assert(_))
            });
            LabeledForecast {
                forecast: forecast.clone(),
                message: message.map(|m| m.message.id),
                correct: message.map(|m| m.message.body.candidate() == Some(last.config.actor())),
            }
        })
        .collect();
    let known: Vec<_> = forecasts
        .iter()
        .filter_map(|f| {
            f.correct.map(|c| {
                (
                    f.forecast.frequency.brier(c),
                    f.forecast.frequency.log_loss(c),
                )
            })
        })
        .collect();
    let world_losses = if known.is_empty() {
        None
    } else {
        Some([
            known.iter().map(|v| v.0).sum::<u64>() / known.len() as u64,
            known.iter().map(|v| v.1).sum::<u64>() / known.len() as u64,
        ])
    };
    let history = r
        .episodes
        .iter()
        .map(|e| IncidentTrace {
            episode: e.id,
            actor: e.incident.config.actor(),
            observations: e.incident.config.observations,
            later: e.incident.config.later.clone(),
            testimony: e.incident.testimony.clone(),
            decisions: e
                .incident
                .decisions
                .iter()
                .map(|d| DecisionTrace {
                    id: d.input.id,
                    phase: d.input.phase,
                    belief: d.input.current.clone(),
                    choice: d.choice.clone(),
                    rejection: d.rejection.clone(),
                })
                .collect(),
            final_cognition: e
                .cognition
                .iter()
                .filter(|c| {
                    e.incident
                        .decisions
                        .iter()
                        .any(|d| d.input.phase == Phase::Judgment && d.update == c.update)
                })
                .cloned()
                .collect(),
            evictions: e
                .ingestions
                .iter()
                .map(|i| (i.input.observer, i.evicted.clone()))
                .collect(),
            verdict: e.incident.verdict,
            correct: e.incident.correct,
        })
        .collect();
    let feedback = r
        .feedback
        .iter()
        .map(|f| FeedbackTrace {
            after_episode: f.after_episode,
            actual_about: f.actual_about,
            source: f.source.clone(),
            fault: f.fault.clone(),
            actual_actor: f.actual_actor,
            factual_accuracy: f.factual_accuracy,
            authentic: f.authentic,
            recipient: f.recipient,
            token: f.input.received.clone(),
            acceptance: f.effect.acceptance,
            credits_before: f.input.memory.local.credits,
            credits_after: f.effect.after.local.credits,
            record_keys: f.input.memory.local.records.iter().map(|r| r.key).collect(),
            receipts_after: f.effect.after.receipts.clone(),
            unverifiable_eviction: f.effect.unverifiable_eviction,
        })
        .collect();
    Ok(Row {
        family,
        seed,
        generated: world.is_some(),
        changed: world.is_some_and(|w| w.changed),
        corrections: world.is_some_and(|w| w.corrections),
        acceptance: config.acceptance,
        listener: config.listener,
        incident_count: r.episodes.len(),
        digests,
        replay_matched: true,
        rejected: r
            .episodes
            .iter()
            .flat_map(|e| &e.incident.decisions)
            .filter(|d| d.rejection.is_some())
            .count(),
        reliability: Reliability::from_report(&r),
        forecasts,
        world_losses,
        history,
        feedback,
    })
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastMetrics {
    pub frequency: Calibration,
    pub neutral: Calibration,
    pub ordinal_decisions: u64,
    pub ordinal_correct: u64,
    pub scores: [Bin; 5],
}
impl ForecastMetrics {
    fn add(&mut self, f: &LabeledForecast) {
        if let Some(correct) = f.correct {
            self.frequency.add(f.forecast.frequency, correct);
            self.neutral.add(f.forecast.neutral, correct);
            if let Some(p) = f.forecast.ordinal {
                self.ordinal_decisions += 1;
                self.ordinal_correct += u64::from(p == correct);
            }
            let bin = &mut self.scores[(f.forecast.input.score + 2) as usize];
            bin.samples += 1;
            bin.correct += u64::from(correct);
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stratum {
    pub label: String,
    pub metrics: ForecastMetrics,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub family: String,
    pub generated: bool,
    pub acceptance: VerificationKind,
    pub listener: e6::ListenerKind,
    pub reliability: Reliability,
    pub metrics: BTreeMap<String, Option<i64>>,
    pub forecasts: ForecastMetrics,
    pub strata: Vec<Stratum>,
    pub intervals: BTreeMap<String, Option<[i64; 2]>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    pub kind: String,
    pub family: String,
    pub acceptance: Option<VerificationKind>,
    pub listener: Option<e6::ListenerKind>,
    pub worlds: usize,
    pub identical_current_wire: usize,
    pub identical_current_private_inputs: usize,
    pub votes_changed: usize,
    pub verdicts_changed: usize,
    pub delta_metrics: BTreeMap<String, Option<i64>>,
    pub intervals: BTreeMap<String, Option<[i64; 2]>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub implementation: String,
    pub development_seeds: [u64; 2],
    pub held_out_seeds: [u64; 2],
    pub diagnostic_seeds: [u64; 2],
    pub incidents: usize,
    pub replay_matches: usize,
    pub rows: Vec<Row>,
    pub groups: Vec<Group>,
    pub comparisons: Vec<Comparison>,
}
fn aggregate(rows: &[&Row]) -> Reliability {
    let mut c = Reliability::default();
    for r in rows {
        c.add(&r.reliability);
    }
    c
}
fn metrics(rows: &[&Row]) -> BTreeMap<String, Option<i64>> {
    let mut m = aggregate(rows).metrics();
    let losses: Vec<_> = rows.iter().filter_map(|r| r.world_losses).collect();
    m.insert(
        "brier_world_mean".into(),
        ratio(losses.iter().map(|v| v[0]).sum(), losses.len() as u64).map(|v| v as i64),
    );
    m.insert(
        "log_loss_world_mean".into(),
        ratio(losses.iter().map(|v| v[1]).sum(), losses.len() as u64).map(|v| v as i64),
    );
    m.insert(
        "brier_frequency_minus_neutral".into(),
        m["brier_world_mean"].map(|v| v - 250_000),
    );
    m
}
fn difference(
    a: &BTreeMap<String, Option<i64>>,
    b: &BTreeMap<String, Option<i64>>,
) -> BTreeMap<String, Option<i64>> {
    b.iter()
        .map(|(k, v)| (k.clone(), v.zip(a[k]).map(|(b, a)| b - a)))
        .collect()
}
fn bootstrap(a: Option<&[&Row]>, b: &[&Row]) -> BTreeMap<String, Option<[i64; 2]>> {
    let mut rng = Rng::new(7007);
    let mut samples: BTreeMap<String, Vec<Option<i64>>> = BTreeMap::new();
    for _ in 0..512 {
        let indices: Vec<_> = (0..b.len())
            .map(|_| (rng.draw() % b.len() as u64) as usize)
            .collect();
        let br: Vec<_> = indices.iter().map(|&i| b[i]).collect();
        let mut values = metrics(&br);
        if let Some(a) = a {
            let ar: Vec<_> = indices.iter().map(|&i| a[i]).collect();
            values = difference(&metrics(&ar), &values);
        }
        for (k, v) in values {
            samples.entry(k).or_default().push(v);
        }
    }
    samples
        .into_iter()
        .map(|(k, values)| {
            let value = if values.iter().any(Option::is_none) {
                None
            } else {
                let mut v: Vec<_> = values.into_iter().flatten().collect();
                v.sort_unstable();
                Some([v[12], v[499]])
            };
            (k, value)
        })
        .collect()
}
fn matched(
    a: &[&Row],
    b: &[&Row],
    kind: &str,
    family: &str,
    acceptance: Option<VerificationKind>,
    listener: Option<e6::ListenerKind>,
) -> Comparison {
    let mut c = Comparison {
        kind: kind.into(),
        family: family.into(),
        acceptance,
        listener,
        worlds: a.len(),
        identical_current_wire: 0,
        identical_current_private_inputs: 0,
        votes_changed: 0,
        verdicts_changed: 0,
        delta_metrics: difference(&metrics(a), &metrics(b)),
        intervals: bootstrap(Some(a), b),
    };
    for (a, b) in a.iter().zip(b) {
        let x = a.history.last().unwrap();
        let y = b.history.last().unwrap();
        c.identical_current_wire += usize::from(
            x.testimony
                .iter()
                .map(|m| (&m.message, &m.recipients))
                .eq(y.testimony.iter().map(|m| (&m.message, &m.recipients))),
        );
        c.identical_current_private_inputs += usize::from(
            x.actor == y.actor && x.observations == y.observations && x.later == y.later,
        );
        c.votes_changed += x
            .decisions
            .iter()
            .filter(|d| d.phase == Phase::Judgment)
            .zip(y.decisions.iter().filter(|d| d.phase == Phase::Judgment))
            .filter(|(a, b)| a.choice.action != b.choice.action)
            .count();
        c.verdicts_changed += usize::from(x.verdict != y.verdict);
    }
    c
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut e = evaluate_with(&mut Generator)?;
    e.implementation = "builtin-007-v1".into();
    Ok(e)
}
pub trait EvaluationPolicy {
    fn summarize(&mut self, evaluation: &mut Evaluation);
}
pub struct PreregisteredEvaluation;
pub fn evaluate_with(generator: &mut dyn WorldGenerator) -> Result<Evaluation, String> {
    evaluate_with_components(generator, &mut PreregisteredEvaluation)
}
pub fn evaluate_with_components(
    generator: &mut dyn WorldGenerator,
    evaluation: &mut dyn EvaluationPolicy,
) -> Result<Evaluation, String> {
    let mut e = Evaluation {
        version: "evaluation-007-v1".into(),
        implementation: "caller-supplied-unversioned".into(),
        development_seeds: [0, 15],
        held_out_seeds: [1000, 1063],
        diagnostic_seeds: [0, 3],
        incidents: 0,
        replay_matches: 0,
        rows: vec![],
        groups: vec![],
        comparisons: vec![],
    };
    for seed in 1000..1064 {
        let world = generator.generate(seed);
        for fault in FAULTS {
            for acceptance in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
                for listener in [e6::ListenerKind::Frozen005, e6::ListenerKind::Credibility] {
                    e.rows.push(row(
                        format!("{fault:?}"),
                        seed,
                        Some(&world),
                        generated_config(&world, fault, acceptance, listener),
                    )?);
                }
            }
        }
    }
    for d in DIAGNOSTICS {
        for seed in 0..4 {
            for acceptance in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
                for listener in [e6::ListenerKind::Frozen005, e6::ListenerKind::Credibility] {
                    e.rows.push(row(
                        format!("{d:?}"),
                        seed,
                        None,
                        diagnostic_config(d, seed, acceptance, listener),
                    )?);
                }
            }
        }
    }
    e.incidents = e.rows.iter().map(|r| r.incident_count).sum();
    e.replay_matches = e.rows.iter().filter(|r| r.replay_matched).count();
    evaluation.summarize(&mut e);
    Ok(e)
}
impl EvaluationPolicy for PreregisteredEvaluation {
    fn summarize(&mut self, e: &mut Evaluation) {
        for generated in [true, false] {
            let families: Vec<String> = if generated {
                FAULTS.iter().map(|f| format!("{f:?}")).collect()
            } else {
                DIAGNOSTICS.iter().map(|f| format!("{f:?}")).collect()
            };
            for family in families {
                for acceptance in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
                    for listener in [e6::ListenerKind::Frozen005, e6::ListenerKind::Credibility] {
                        let rows: Vec<_> = e
                            .rows
                            .iter()
                            .filter(|r| {
                                r.generated == generated
                                    && r.family == family
                                    && r.acceptance == acceptance
                                    && r.listener == listener
                            })
                            .collect();
                        let mut forecasts = ForecastMetrics::default();
                        let mut strata: BTreeMap<String, ForecastMetrics> = BTreeMap::new();
                        for r in &rows {
                            for f in &r.forecasts {
                                forecasts.add(f);
                                for label in [
                                    format!(
                                        "verified_count_{}",
                                        (u32::from(f.forecast.input.accurate)
                                            + u32::from(f.forecast.input.inaccurate))
                                        .min(4)
                                    ),
                                    format!("changed_{}", r.changed),
                                    format!("corrections_{}", r.corrections),
                                ] {
                                    strata.entry(label).or_default().add(f);
                                }
                            }
                        }
                        e.groups.push(Group {
                            family: family.clone(),
                            generated,
                            acceptance,
                            listener,
                            reliability: aggregate(&rows),
                            metrics: metrics(&rows),
                            forecasts,
                            strata: strata
                                .into_iter()
                                .map(|(label, metrics)| Stratum { label, metrics })
                                .collect(),
                            intervals: if generated {
                                bootstrap(None, &rows)
                            } else {
                                BTreeMap::new()
                            },
                        });
                    }
                }
            }
        }
        for fault in FAULTS {
            let family = format!("{fault:?}");
            for acceptance in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
                let a: Vec<_> = e
                    .rows
                    .iter()
                    .filter(|r| {
                        r.generated
                            && r.family == family
                            && r.acceptance == acceptance
                            && r.listener == e6::ListenerKind::Frozen005
                    })
                    .collect();
                let b: Vec<_> = e
                    .rows
                    .iter()
                    .filter(|r| {
                        r.generated
                            && r.family == family
                            && r.acceptance == acceptance
                            && r.listener == e6::ListenerKind::Credibility
                    })
                    .collect();
                e.comparisons
                    .push(matched(&a, &b, "listener", &family, Some(acceptance), None));
            }
            for listener in [e6::ListenerKind::Frozen005, e6::ListenerKind::Credibility] {
                let a: Vec<_> = e
                    .rows
                    .iter()
                    .filter(|r| {
                        r.generated
                            && r.family == family
                            && r.acceptance == VerificationKind::Frozen006
                            && r.listener == listener
                    })
                    .collect();
                let b: Vec<_> = e
                    .rows
                    .iter()
                    .filter(|r| {
                        r.generated
                            && r.family == family
                            && r.acceptance == VerificationKind::ConflictAware
                            && r.listener == listener
                    })
                    .collect();
                e.comparisons
                    .push(matched(&a, &b, "acceptance", &family, None, Some(listener)));
            }
            if fault != Fault::Clean {
                for acceptance in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
                    for listener in [e6::ListenerKind::Frozen005, e6::ListenerKind::Credibility] {
                        let a: Vec<_> = e
                            .rows
                            .iter()
                            .filter(|r| {
                                r.generated
                                    && r.family == "Clean"
                                    && r.acceptance == acceptance
                                    && r.listener == listener
                            })
                            .collect();
                        let b: Vec<_> = e
                            .rows
                            .iter()
                            .filter(|r| {
                                r.generated
                                    && r.family == family
                                    && r.acceptance == acceptance
                                    && r.listener == listener
                            })
                            .collect();
                        e.comparisons.push(matched(
                            &a,
                            &b,
                            "verification_stream",
                            &family,
                            Some(acceptance),
                            Some(listener),
                        ));
                    }
                }
            }
        }
    }
}
