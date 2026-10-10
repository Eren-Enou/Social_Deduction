use super::*;
use crate::{Phase, Rng, experiment005 as e5, experiment007 as e7};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub fn canonical_digest<T: Serialize>(v: &T) -> Result<String, String> {
    let mut bytes = b"canonical-008-v1\n".to_vec();
    bytes.extend(e7::canonical_bytes(v)?);
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
        "investigation_deliveries".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| (&e.investigation, &e.deliveries))
                .collect::<Vec<_>>(),
        )?,
    );
    d.insert(
        "memory_cognition".into(),
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
        "communications".into(),
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metrics {
    pub opportunities: u64,
    pub eligible: u64,
    pub selected: u64,
    pub declined: u64,
    pub rejected: u64,
    pub communication_rejections: u64,
    pub spent: u64,
    pub conserved: u64,
    pub correct_findings: u64,
    pub wrong_findings: u64,
    pub missing_findings: u64,
    pub injected_findings: u64,
    pub candidate_changes: u64,
    pub support_changes: u64,
    pub acquisition_improved: u64,
    pub acquisition_worsened: u64,
    pub acquisition_null: u64,
    pub unnecessary: u64,
    pub useful_private: u64,
    pub used_groups: u64,
    pub ignored_findings: u64,
    pub contested_groups: u64,
    pub duplicate_exposures: u64,
    pub known_dependent: u64,
    pub hidden_dependent: u64,
    pub correct_ballots: u64,
    pub wrong_ballots: u64,
    pub abstentions: u64,
    pub correct_verdicts: u64,
    pub wrong_verdicts: u64,
    pub unresolved: u64,
    pub full_report_bytes: u64,
    pub final_memory_bytes: u64,
    pub inspections: u64,
    pub max_records: u64,
    pub max_findings: u64,
    pub max_messages: u64,
}
impl Metrics {
    pub fn add(&mut self, other: &Self) {
        macro_rules! sum {($($f:ident),*)=>{$(self.$f+=other.$f;)*};}
        sum!(
            opportunities,
            eligible,
            selected,
            declined,
            rejected,
            communication_rejections,
            spent,
            conserved,
            correct_findings,
            wrong_findings,
            missing_findings,
            injected_findings,
            candidate_changes,
            support_changes,
            acquisition_improved,
            acquisition_worsened,
            acquisition_null,
            unnecessary,
            useful_private,
            used_groups,
            ignored_findings,
            contested_groups,
            duplicate_exposures,
            known_dependent,
            hidden_dependent,
            correct_ballots,
            wrong_ballots,
            abstentions,
            correct_verdicts,
            wrong_verdicts,
            unresolved,
            full_report_bytes,
            final_memory_bytes,
            inspections
        );
        self.max_records = self.max_records.max(other.max_records);
        self.max_findings = self.max_findings.max(other.max_findings);
        self.max_messages = self.max_messages.max(other.max_messages);
    }
    pub fn coverage(&self) -> Option<i64> {
        ratio(
            self.correct_ballots + self.wrong_ballots,
            self.correct_ballots + self.wrong_ballots + self.abstentions,
        )
    }
    pub fn accuracy(&self) -> Option<i64> {
        ratio(
            self.correct_ballots,
            self.correct_ballots + self.wrong_ballots,
        )
    }
    pub fn wrong_risk(&self) -> Option<i64> {
        ratio(
            self.wrong_ballots,
            self.correct_ballots + self.wrong_ballots + self.abstentions,
        )
    }
}
fn ratio(a: u64, b: u64) -> Option<i64> {
    (b != 0).then(|| (a * 1_000_000 / b) as i64)
}
pub fn metrics(r: &Report) -> Result<Metrics, String> {
    let mut m = Metrics {
        spent: r.ledgers.iter().map(|l| u64::from(l.spent)).sum(),
        conserved: r.ledgers.iter().map(|l| u64::from(l.remaining)).sum(),
        full_report_bytes: serde_json::to_vec(r).map_err(|e| e.to_string())?.len() as u64,
        final_memory_bytes: serde_json::to_vec(&r.final_memory)
            .map_err(|e| e.to_string())?
            .len() as u64,
        ..Metrics::default()
    };
    for e in &r.episodes {
        m.communication_rejections += e
            .incident
            .decisions
            .iter()
            .filter(|d| d.input.phase == Phase::Discussion && d.rejection.is_some())
            .count() as u64;
        m.opportunities += e.investigation.len() as u64;
        for d in &e.investigation {
            m.inspections += d.assessment.inspections as u64;
            m.eligible += u64::from(d.input.balance > 0 && !d.input.catalog.is_empty());
            m.rejected += u64::from(d.rejection.is_some());
            m.selected += u64::from(
                d.rejection.is_none() && matches!(d.choice.action, InvestigationAction::Inspect(_)),
            );
            m.declined +=
                u64::from(d.rejection.is_none() && d.choice.action == InvestigationAction::Decline);
        }
        for d in &e.deliveries {
            m.inspections +=
                (d.before_assessment.inspections + d.after_assessment.inspections) as u64;
            if matches!(d.origin, DeliveryOrigin::Intervention(_)) {
                m.injected_findings += 1;
            }
            match d.correct {
                Some(true) => m.correct_findings += 1,
                Some(false) => m.wrong_findings += 1,
                None => m.missing_findings += 1,
            }
            let before = &d.before_assessment.after;
            let after = &d.after_assessment.after;
            m.candidate_changes += u64::from(before.candidate != after.candidate);
            m.support_changes += u64::from(before.scores != after.scores);
            let actor = e.incident.config.actor();
            let diff = i8::from(after.candidate == Some(actor))
                - i8::from(before.candidate == Some(actor));
            if diff > 0 {
                m.acquisition_improved += 1;
            } else if diff < 0 {
                m.acquisition_worsened += 1;
            } else {
                m.acquisition_null += 1;
            }
            m.unnecessary +=
                u64::from(before == after && matches!(d.origin, DeliveryOrigin::Selected(_)));
            m.useful_private += u64::from(
                before != after
                    && !e.communication.iter().any(|c| {
                        c.input.incident.id == d.finding.acquisition.observer
                            && c.decision >= 6
                            && c.decision < 12
                            && e.incident.decisions[c.decision].rejection.is_none()
                            && matches!(c.choice.choice.action, e5::Action::Speak { .. })
                    })
                    && !e
                        .reports
                        .iter()
                        .any(|p| p.finding.acquisition == d.finding.acquisition),
            );
            m.known_dependent += u64::from(
                matches!(d.actual_root, Root::Observation(_))
                    && d.finding.source.visible_parent.is_some(),
            );
            m.hidden_dependent += u64::from(
                matches!(d.actual_root, Root::Observation(_))
                    && d.finding.source.visible_parent.is_none(),
            );
            m.ignored_findings +=
                u64::from(d.finding.candidate.is_none() || before.scores == after.scores);
        }
        for c in &e.cognition {
            m.inspections += c.assessment.inspections as u64;
            m.used_groups += c
                .assessment
                .contributions
                .iter()
                .filter(|g| g.weight > 0)
                .count() as u64;
            m.contested_groups += c
                .assessment
                .contributions
                .iter()
                .filter(|g| g.contested)
                .count() as u64;
            m.duplicate_exposures += c
                .assessment
                .contributions
                .iter()
                .map(|g| g.observations.saturating_sub(1) as u64)
                .sum::<u64>();
            m.max_findings = m.max_findings.max(c.input.findings.len() as u64);
            m.max_messages = m.max_messages.max(c.input.incident.received.len() as u64);
        }
        for d in &e.incident.decisions {
            if d.input.phase == Phase::Judgment {
                match d.choice.action {
                    e5::Action::Vote(Some(c)) if c == e.incident.config.actor() => {
                        m.correct_ballots += 1
                    }
                    e5::Action::Vote(Some(_)) => m.wrong_ballots += 1,
                    _ => m.abstentions += 1,
                }
            }
        }
        match e.incident.correct {
            Some(true) => m.correct_verdicts += 1,
            Some(false) => m.wrong_verdicts += 1,
            None => m.unresolved += 1,
        }
        m.max_records = m.max_records.max(
            e.memory_after
                .iter()
                .map(|x| x.local.records.len() as u64)
                .max()
                .unwrap_or(0),
        );
    }
    Ok(m)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryTrace {
    pub origin: DeliveryOrigin,
    pub finding: Finding,
    pub root: Root,
    pub correct: Option<bool>,
    pub before: e5::Belief,
    pub after: e5::Belief,
    pub balances: (u8, u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeTrace {
    pub actor: u8,
    pub investigation: Vec<Investigation>,
    pub deliveries: Vec<DeliveryTrace>,
    pub reports: Vec<ReportedFinding>,
    pub messages: Vec<e5::Testimony>,
    pub ancestry: Vec<Ancestry>,
    pub ballots: [Option<u8>; 6],
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub population: String,
    pub family: String,
    pub seed: u64,
    pub policy: PolicyKind,
    pub injected: bool,
    pub metrics: Metrics,
    pub coverage_millionths: Option<i64>,
    pub conditional_accuracy_millionths: Option<i64>,
    pub incorrect_risk_millionths: Option<i64>,
    pub digests: BTreeMap<String, String>,
    pub trace: Vec<EpisodeTrace>,
    pub communication_digest: String,
    pub ballot_digest: String,
}
fn row(
    population: &str,
    family: String,
    seed: u64,
    policy: PolicyKind,
    injected: bool,
    c: Config,
) -> Result<Row, String> {
    let a = run(c.clone())?;
    let b = run(c)?;
    let digests = report_digests(&a)?;
    if digests != report_digests(&b)? || a != b {
        return Err("008 full causal replay mismatch".into());
    }
    let metrics = metrics(&a)?;
    let trace: Vec<_> = a
        .episodes
        .iter()
        .map(|e| EpisodeTrace {
            actor: e.incident.config.actor(),
            investigation: e.investigation.clone(),
            deliveries: e
                .deliveries
                .iter()
                .map(|d| DeliveryTrace {
                    origin: d.origin,
                    finding: d.finding.clone(),
                    root: d.actual_root,
                    correct: d.correct,
                    before: d.before_assessment.after.clone(),
                    after: d.after_assessment.after.clone(),
                    balances: (d.balance_before, d.balance_after),
                })
                .collect(),
            reports: e.reports.clone(),
            messages: e.incident.testimony.clone(),
            ancestry: e.ancestry.clone(),
            ballots: std::array::from_fn(|id| {
                e.incident
                    .decisions
                    .iter()
                    .find(|d| d.input.id == id as u8 && d.input.phase == Phase::Judgment)
                    .and_then(|d| {
                        if let e5::Action::Vote(c) = d.choice.action {
                            c
                        } else {
                            None
                        }
                    })
            }),
            verdict: e.incident.verdict,
            correct: e.incident.correct,
        })
        .collect();
    let communication_digest = canonical_digest(
        &trace
            .iter()
            .map(|t| (&t.messages, &t.reports))
            .collect::<Vec<_>>(),
    )?;
    let ballot_digest = canonical_digest(&trace.iter().map(|t| t.ballots).collect::<Vec<_>>())?;
    Ok(Row {
        population: population.into(),
        family,
        seed,
        policy,
        injected,
        coverage_millionths: metrics.coverage(),
        conditional_accuracy_millionths: metrics.accuracy(),
        incorrect_risk_millionths: metrics.wrong_risk(),
        metrics,
        digests,
        trace,
        communication_digest,
        ballot_digest,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub population: String,
    pub policy: PolicyKind,
    pub injected: bool,
    pub series: usize,
    pub metrics: Metrics,
    pub coverage_millionths: Option<i64>,
    pub conditional_accuracy_millionths: Option<i64>,
    pub incorrect_risk_millionths: Option<i64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    pub lower: Option<i64>,
    pub upper: Option<i64>,
    pub defined_resamples: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utility {
    pub wrong_cost: i64,
    pub effort_price_millionths: i64,
    pub paired_mean_millionths: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    pub policy: PolicyKind,
    pub injected: bool,
    pub baseline: PolicyKind,
    pub worlds: usize,
    pub improved: usize,
    pub worsened: usize,
    pub unchanged: usize,
    pub communication_changed: usize,
    pub ballots_changed: usize,
    pub verdict_transitions: BTreeMap<String, usize>,
    pub spent_delta: i64,
    pub correct_ballot_delta: i64,
    pub coverage_delta_millionths: Option<i64>,
    pub accuracy_delta_millionths: Option<i64>,
    pub wrong_risk_delta_millionths: Option<i64>,
    pub coverage_interval: Interval,
    pub accuracy_interval: Interval,
    pub wrong_risk_interval: Interval,
    pub utilities: Vec<Utility>,
}
fn difference(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    a.zip(b).map(|(a, b)| a - b)
}
fn comparison(
    rows: &[Row],
    policy: PolicyKind,
    injected: bool,
    baseline: PolicyKind,
) -> Comparison {
    let pairs: Vec<_> = rows
        .iter()
        .filter(|r| r.population == "heldout" && r.policy == policy && r.injected == injected)
        .map(|a| {
            (
                a,
                rows.iter()
                    .find(|b| {
                        b.population == "heldout"
                            && b.seed == a.seed
                            && b.policy == baseline
                            && !b.injected
                    })
                    .unwrap(),
            )
        })
        .collect();
    let mut a = Metrics::default();
    let mut b = Metrics::default();
    let mut transitions = BTreeMap::new();
    let mut improved = 0;
    let mut worsened = 0;
    let mut communication_changed = 0;
    let mut ballots_changed = 0;
    for (x, y) in &pairs {
        a.add(&x.metrics);
        b.add(&y.metrics);
        if x.metrics.correct_ballots > y.metrics.correct_ballots {
            improved += 1;
        } else if x.metrics.correct_ballots < y.metrics.correct_ballots {
            worsened += 1;
        }
        communication_changed += usize::from(x.communication_digest != y.communication_digest);
        ballots_changed += usize::from(x.ballot_digest != y.ballot_digest);
        for (xt, yt) in x.trace.iter().zip(&y.trace) {
            let key = format!("{:?}->{:?}", yt.correct, xt.correct);
            *transitions.entry(key).or_insert(0) += 1;
        }
    }
    let mut samples: [Vec<i64>; 3] = std::array::from_fn(|_| vec![]);
    let mut rng = Rng::new(8008);
    for _ in 0..512 {
        let mut x = Metrics::default();
        let mut y = Metrics::default();
        for _ in 0..pairs.len() {
            let (a, b) = pairs[(rng.draw() % pairs.len() as u64) as usize];
            x.add(&a.metrics);
            y.add(&b.metrics);
        }
        for (values, delta) in samples.iter_mut().zip([
            difference(x.coverage(), y.coverage()),
            difference(x.accuracy(), y.accuracy()),
            difference(x.wrong_risk(), y.wrong_risk()),
        ]) {
            if let Some(delta) = delta {
                values.push(delta);
            }
        }
    }
    let intervals: Vec<_> = samples
        .into_iter()
        .map(|mut values| {
            values.sort();
            let n = values.len();
            Interval {
                lower: values.get(12).copied(),
                upper: values.get(499).copied(),
                defined_resamples: n,
            }
        })
        .collect();
    let mut utilities = vec![];
    for wrong_cost in [1, 2, 4] {
        for effort_price_millionths in [0, 250_000, 1_000_000] {
            let value = |m: &Metrics| {
                (m.correct_verdicts as i64 - wrong_cost * m.wrong_verdicts as i64) * 1_000_000
                    - effort_price_millionths * m.spent as i64
            };
            utilities.push(Utility {
                wrong_cost,
                effort_price_millionths,
                paired_mean_millionths: (value(&a) - value(&b)) / pairs.len() as i64,
            });
        }
    }
    Comparison {
        policy,
        injected,
        baseline,
        worlds: pairs.len(),
        improved,
        worsened,
        unchanged: pairs.len() - improved - worsened,
        communication_changed,
        ballots_changed,
        verdict_transitions: transitions,
        spent_delta: a.spent as i64 - b.spent as i64,
        correct_ballot_delta: a.correct_ballots as i64 - b.correct_ballots as i64,
        coverage_delta_millionths: difference(a.coverage(), b.coverage()),
        accuracy_delta_millionths: difference(a.accuracy(), b.accuracy()),
        wrong_risk_delta_millionths: difference(a.wrong_risk(), b.wrong_risk()),
        coverage_interval: intervals[0].clone(),
        accuracy_interval: intervals[1].clone(),
        wrong_risk_interval: intervals[2].clone(),
        utilities,
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub independent_worlds: usize,
    pub incidents: usize,
    pub replay_matches: usize,
    pub rows: Vec<Row>,
    pub groups: Vec<Group>,
    pub comparisons: Vec<Comparison>,
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut rows = vec![];
    for family in DIAGNOSTICS {
        for seed in 0..4 {
            for policy in POLICIES {
                rows.push(row(
                    "diagnostic",
                    format!("{family:?}"),
                    seed,
                    policy,
                    family == Diagnostic::Injected,
                    diagnostic_config(family, seed, policy),
                )?);
            }
        }
    }
    for seed in 2000..2032 {
        let world = Generator.generate(seed);
        for policy in POLICIES {
            rows.push(row(
                "heldout",
                "generated".into(),
                seed,
                policy,
                false,
                generated_config(&world, policy, false),
            )?);
        }
        rows.push(row(
            "heldout",
            "generated".into(),
            seed,
            PolicyKind::NeverInvestigate,
            true,
            generated_config(&world, PolicyKind::NeverInvestigate, true),
        )?);
    }
    let mut groups = vec![];
    for population in ["diagnostic", "heldout"] {
        for policy in POLICIES {
            for injected in [false, true] {
                let matching: Vec<_> = rows
                    .iter()
                    .filter(|r| {
                        r.population == population && r.policy == policy && r.injected == injected
                    })
                    .collect();
                if matching.is_empty() {
                    continue;
                }
                let mut metrics = Metrics::default();
                for row in &matching {
                    metrics.add(&row.metrics);
                }
                groups.push(Group {
                    population: population.into(),
                    policy,
                    injected,
                    series: matching.len(),
                    coverage_millionths: metrics.coverage(),
                    conditional_accuracy_millionths: metrics.accuracy(),
                    incorrect_risk_millionths: metrics.wrong_risk(),
                    metrics,
                });
            }
        }
    }
    let mut comparisons = vec![];
    for policy in POLICIES {
        for baseline in [PolicyKind::NeverInvestigate, PolicyKind::AlwaysInvestigate] {
            if policy != baseline {
                comparisons.push(comparison(&rows, policy, false, baseline));
            }
        }
    }
    comparisons.push(comparison(
        &rows,
        PolicyKind::NeverInvestigate,
        true,
        PolicyKind::NeverInvestigate,
    ));
    Ok(Evaluation {
        version: "evaluation-008-v1".into(),
        independent_worlds: 32,
        incidents: rows.iter().map(|r| r.trace.len()).sum(),
        replay_matches: rows.len(),
        rows,
        groups,
        comparisons,
    })
}
pub fn manifest(e: &Evaluation) -> Result<e7::ReplayManifest, String> {
    Ok(e7::ReplayManifest {
        version: "replay-manifest-008-v1".into(),
        series: e.rows.len(),
        incidents: e.incidents,
        evaluation_digest: canonical_digest(e)?,
        metric_digest: canonical_digest(&(&e.groups, &e.comparisons))?,
        series_sections: e.rows.iter().map(|r| r.digests.clone()).collect(),
    })
}
