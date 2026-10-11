use super::*;
use crate::{Phase, Rng, experiment005 as e5, experiment007 as e7, experiment010 as e10};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTrace {
    pub reference: Reference,
    pub observer: u8,
    pub input_digest: String,
    pub choice: RequestChoice,
    pub rejection: Option<String>,
    pub delivered: bool,
    pub recipient_counts: Option<Counts>,
    pub evicted: Vec<Reference>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplyTrace {
    pub reference: Reference,
    pub observer: u8,
    pub input_digest: String,
    pub personal: BTreeMap<MethodId, Counts>,
    pub received_requests: Vec<Request>,
    pub choice: ResponseChoice,
    pub proposal: Option<ResponseChoice>,
    pub origin: Origin,
    pub rejection: Option<String>,
    pub delivered: bool,
    pub actual_origins: Vec<Reference>,
    pub actual_depth: u8,
    pub evicted: Vec<Reference>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityTrace {
    pub reference: Reference,
    pub agent: u8,
    pub window: Window,
    pub before: u8,
    pub after: u8,
    pub submitted: bool,
    pub rejected: bool,
    pub exhausted: bool,
    pub foregone: Option<CommunicationChoice>,
    pub foregone_input_digest: Option<String>,
}
fn inquiry_traces(e: &Episode, row: &mut Row) -> Result<(), String> {
    for q in e.requests.iter().filter(|q| q.delivered.is_some()) {
        let request = q.choice.selected.as_ref().unwrap();
        increment(
            &mut row.counters,
            "current_incident_unanswered_queries",
            u64::from(!e.responses.iter().any(|a| {
                !a.delivered.is_empty()
                    && a.choice
                        .selected
                        .as_ref()
                        .is_some_and(|a| a.request == request.reference)
            })),
        );
    }
    increment(
        &mut row.counters,
        "ordinary_statements",
        e.incident.testimony.len() as u64,
    );
    increment(
        &mut row.counters,
        "response_late_deliveries",
        e.responses
            .iter()
            .filter(|d| !d.delivered.is_empty() && d.input.window == Window::Final)
            .count() as u64,
    );

    for q in &e.requests {
        increment(&mut row.counters, "query_opportunities", 1);
        increment(
            &mut row.counters,
            "query_declined",
            u64::from(q.choice.selected.is_none()),
        );
        increment(
            &mut row.counters,
            "query_selected",
            u64::from(q.choice.selected.is_some()),
        );
        increment(
            &mut row.counters,
            "query_rejected",
            u64::from(q.rejection.is_some()),
        );
        increment(
            &mut row.counters,
            "query_delivered",
            u64::from(q.delivered.is_some()),
        );
        if let Some(selected) = &q.choice.selected {
            let known =
                q.input.local.social.received.iter().any(|m| {
                    m.speaker == selected.recipient && m.payload.family == selected.family
                });
            increment(
                &mut row.counters,
                if known {
                    "known_possible_expertise"
                } else {
                    "unknown_possible_expertise"
                },
                1,
            );
            if let Some(c) = q.recipient_counts {
                increment(
                    &mut row.counters,
                    if c.observed() > 0 {
                        "recipient_observed"
                    } else {
                        "recipient_unobserved"
                    },
                    1,
                );
                increment(
                    &mut row.counters,
                    "recipient_graded",
                    u64::from(c.graded() > 0),
                );
            }
            increment(
                &mut row.counters,
                &format!("query_type_{:?}", selected.kind),
                1,
            );
        }
        increment(
            &mut row.counters,
            "query_actions_consumed",
            u64::from(q.choice.selected.is_some()),
        );
        increment(
            &mut row.counters,
            "query_wire_bytes",
            q.choice
                .selected
                .as_ref()
                .map(|q| serde_json::to_vec(q).unwrap().len() as u64)
                .unwrap_or(0),
        );
        row.queries.push(QueryTrace {
            reference: q.input.local.reference,
            observer: q.input.id(),
            input_digest: canonical_digest(&q.input)?,
            choice: q.choice.clone(),
            rejection: q.rejection.clone(),
            delivered: q.delivered.is_some(),
            recipient_counts: q.recipient_counts,
            evicted: q
                .delivered
                .iter()
                .chain(&q.sent)
                .flat_map(|d| d.evicted.clone())
                .collect(),
        });
    }
    for a in &e.responses {
        increment(&mut row.counters, "response_opportunities", 1);
        increment(
            &mut row.counters,
            "response_absent",
            u64::from(a.choice.selected.is_none()),
        );
        increment(
            &mut row.counters,
            "response_rejected",
            u64::from(a.rejection.is_some()),
        );
        increment(
            &mut row.counters,
            "response_delivered",
            u64::from(!a.delivered.is_empty()),
        );
        if let Some(wire) = &a.choice.selected {
            increment(&mut row.counters, "response_submitted", 1);
            increment(
                &mut row.counters,
                "response_wire_bytes",
                serde_json::to_vec(wire).map_err(|e| e.to_string())?.len() as u64,
            );
            let key = match &wire.answer {
                Answer::InsufficientInformation => "response_insufficient",
                Answer::Refuse => "response_refused",
                Answer::Assessment { payload, .. } => {
                    if a.choice.intent == MethodIntent::Contrary {
                        "response_contrary"
                    } else if a.choice.intent == MethodIntent::Unsupported {
                        "response_unsupported"
                    } else if payload.rating == Rating::Uncertain {
                        "response_uncertain"
                    } else {
                        "response_sincere"
                    }
                }
            };
            increment(&mut row.counters, key, 1);
        }
        increment(
            &mut row.counters,
            "inquiry_evictions",
            a.delivered.iter().map(|d| d.evicted.len() as u64).sum(),
        );
        increment(
            &mut row.counters,
            "reply_actions_consumed",
            u64::from(a.choice.selected.is_some() && a.input.capacity > 0),
        );
        row.replies.push(ReplyTrace {
            reference: a.input.local.reference,
            observer: a.input.id(),
            input_digest: canonical_digest(&a.input)?,
            personal: a.input.local.personal.summaries.clone(),
            received_requests: a.input.memory.received.clone(),
            choice: a.choice.clone(),
            proposal: a.proposal.clone(),
            origin: a.origin,
            rejection: a.rejection.clone(),
            delivered: !a.delivered.is_empty(),
            actual_origins: a
                .social
                .as_ref()
                .map(|s| s.actual_origins.clone())
                .unwrap_or_default(),
            actual_depth: a.social.as_ref().map(|s| s.actual_depth).unwrap_or(0),
            evicted: a.delivered.iter().flat_map(|d| d.evicted.clone()).collect(),
        });
    }
    for c in &e.capacity {
        increment(
            &mut row.counters,
            "outbound_actions",
            u64::from(c.after - c.before),
        );
        increment(
            &mut row.counters,
            "exhausted_final",
            u64::from(c.exhausted && c.window == Window::Final),
        );
        increment(
            &mut row.counters,
            "foregone_incident_statements",
            u64::from(
                c.foregone
                    .as_ref()
                    .is_some_and(|p| matches!(p.choice.action, e5::Action::Speak { .. })),
            ),
        );
        row.capacity.push(CapacityTrace {
            reference: c.reference,
            agent: c.agent,
            window: c.window,
            before: c.before,
            after: c.after,
            submitted: c.submitted,
            rejected: c.rejected,
            exhausted: c.exhausted,
            foregone: c.foregone.clone(),
            foregone_input_digest: c
                .foregone_input
                .as_ref()
                .map(canonical_digest)
                .transpose()?,
        });
    }
    Ok(())
}

pub fn canonical_digest<T: Serialize>(value: &T) -> Result<String, String> {
    let mut bytes = b"canonical-012-v1\n".to_vec();
    bytes.extend(e7::canonical_bytes(value)?);
    Ok(e7::sha256(&bytes))
}
pub fn report_digests(r: &Report) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    out.insert(
        "inquiry_capacity".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| {
                    (
                        &e.inquiry_before,
                        &e.requests,
                        &e.responses,
                        &e.capacity,
                        &e.before_answers,
                        &e.inquiry_after,
                    )
                })
                .collect::<Vec<_>>(),
        )?,
    );
    out.insert("full_series".into(), canonical_digest(r)?);
    out.insert(
        "events".into(),
        canonical_digest(&(
            &r.objective_history,
            r.episodes.iter().map(|e| &e.incident).collect::<Vec<_>>(),
        ))?,
    );
    out.insert(
        "communication".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| (&e.social_decisions, &e.communication, &e.ancestry))
                .collect::<Vec<_>>(),
        )?,
    );
    out.insert(
        "social_memory".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| (&e.social_before, &e.social_after, &e.investigation))
                .collect::<Vec<_>>(),
        )?,
    );
    out.insert(
        "personal_feedback".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| {
                    (
                        &e.method_before,
                        &e.method_ingestions,
                        &e.method_feedback,
                        &e.method_after,
                        &e.memory_before,
                        &e.ingestions,
                        &e.verification,
                        &e.memory_after,
                    )
                })
                .collect::<Vec<_>>(),
        )?,
    );
    out.insert(
        "investigation_resources".into(),
        canonical_digest(&(
            &r.ledgers,
            r.episodes
                .iter()
                .map(|e| (&e.investigation, &e.deliveries))
                .collect::<Vec<_>>(),
        ))?,
    );
    out.insert(
        "outcomes".into(),
        canonical_digest(
            &r.episodes
                .iter()
                .map(|e| (&e.incident.tally, e.incident.verdict, e.incident.correct))
                .collect::<Vec<_>>(),
        )?,
    );
    Ok(out)
}
pub fn environment_digest(c: &Config) -> Result<String, String> {
    e10::environment_digest(&c.base.base)
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub ballots: [u64; 3],
    pub verdicts: [u64; 3],
    pub spent: u64,
    pub retained: u64,
}
fn outcome(r: &Report, assessment: bool) -> Outcome {
    let mut out = Outcome {
        retained: r.ledgers.iter().map(|l| u64::from(l.remaining)).sum(),
        ..Outcome::default()
    };
    for e in r.episodes.iter().filter(|e| !assessment || e.id >= 4) {
        for d in e
            .incident
            .decisions
            .iter()
            .filter(|d| d.input.phase == Phase::Judgment)
        {
            let c = match d.choice.action {
                e5::Action::Vote(Some(x)) if x == e.incident.config.actor() => 0,
                e5::Action::Vote(Some(_)) => 1,
                _ => 2,
            };
            out.ballots[c] += 1;
        }
        out.verdicts[match e.incident.correct {
            Some(true) => 0,
            Some(false) => 1,
            None => 2,
        }] += 1;
        out.spent += e
            .deliveries
            .iter()
            .map(|d| u64::from(d.balance_before - d.balance_after))
            .sum::<u64>();
    }
    out
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Forecast {
    pub before_answer_accuracy: e7::Probability,
    pub before_answer_missing: e7::Probability,
    pub episode: u8,
    pub observer: u8,
    pub source: u8,
    pub family: MethodId,
    pub personal: Counts,
    pub personal_accuracy: e7::Probability,
    pub personal_missing: e7::Probability,
    pub effective_accuracy: e7::Probability,
    pub effective_missing: e7::Probability,
    pub positive: u8,
    pub negative: u8,
    pub interpretations: Vec<Interpretation>,
    pub selected: bool,
    pub forced: bool,
    pub correct: Option<bool>,
    pub missing: bool,
    pub actual_accuracy: e7::Probability,
    pub input_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeechTrace {
    pub reference: Reference,
    pub observer: u8,
    pub input_digest: String,
    pub personal: BTreeMap<MethodId, Counts>,
    pub choice: SharingChoice,
    pub origin: Origin,
    pub rejection: Option<String>,
    pub wire: Option<Message>,
    pub actual_origins: Vec<Reference>,
    pub actual_depth: u8,
    pub recipients: Vec<u8>,
    pub evictions: Vec<(u8, Reference)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceTrace {
    pub before_answer_action: InvestigationAction,
    pub before_answer_digest: String,
    pub episode: u8,
    pub observer: u8,
    pub input_digest: String,
    pub personal_action: InvestigationAction,
    pub selected_action: InvestigationAction,
    pub forced: bool,
    pub balance: u8,
    pub considered: Vec<crate::experiment009::Estimate>,
    pub finding: Option<Finding>,
    pub correct: Option<bool>,
    pub before: crate::experiment005::Belief,
    pub after: crate::experiment005::Belief,
    pub paid: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackTrace {
    pub episode: u8,
    pub observer: u8,
    pub input_digest: String,
    pub token: e7::Token,
    pub actual_correct: Option<bool>,
    pub before: BTreeMap<MethodId, Counts>,
    pub after: BTreeMap<MethodId, Counts>,
    pub result: e7::Acceptance,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchedInterpretation {
    pub episode: u8,
    pub observer: u8,
    pub input_digest: String,
    pub variants: Vec<(SocialKind, InvestigationChoice)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub ordinary_messages: Vec<(u8, e5::Message, Vec<u8>)>,
    pub queries: Vec<QueryTrace>,
    pub replies: Vec<ReplyTrace>,
    pub capacity: Vec<CapacityTrace>,
    pub population: String,
    pub case: String,
    pub seed: u64,
    pub shifted: bool,
    pub environment: String,
    pub digests: BTreeMap<String, String>,
    pub full: Outcome,
    pub assessment: Outcome,
    pub counters: BTreeMap<String, u64>,
    pub forecasts: Vec<Forecast>,
    pub speech: Vec<SpeechTrace>,
    pub choices: Vec<ChoiceTrace>,
    pub feedback: Vec<FeedbackTrace>,
    pub matched: Vec<MatchedInterpretation>,
    pub ballots: Vec<(u8, u8, Option<u8>)>,
    pub outcomes: Vec<(u8, u8, Option<u8>, Option<bool>)>,
}
fn increment(c: &mut BTreeMap<String, u64>, key: &str, n: u64) {
    *c.entry(key.into()).or_default() += n;
}
pub fn row(population: &str, case: &str, seed: u64, c: Config) -> Result<Row, String> {
    let r = run(c.clone())?;
    let replay = run(c)?;
    if r != replay {
        return Err("complete012 replay differs".into());
    }
    drop(replay);
    let mut row = Row {
        population: population.into(),
        case: case.into(),
        seed,
        shifted: r.config.processes.iter().any(|p| p.shift_episode.is_some()),
        environment: environment_digest(&r.config)?,
        digests: report_digests(&r)?,
        full: outcome(&r, false),
        assessment: outcome(&r, true),
        counters: BTreeMap::new(),
        ordinary_messages: vec![],
        queries: vec![],
        replies: vec![],
        capacity: vec![],
        forecasts: vec![],
        speech: vec![],
        choices: vec![],
        feedback: vec![],
        matched: vec![],
        ballots: vec![],
        outcomes: vec![],
    };
    let roots: BTreeMap<_, _> = r
        .episodes
        .iter()
        .flat_map(|e| {
            e.social_decisions
                .iter()
                .chain(e.responses.iter().filter_map(|d| d.social.as_ref()))
        })
        .filter_map(|s| {
            s.message
                .as_ref()
                .map(|m| (m.reference, s.actual_origins.clone()))
        })
        .collect();
    for e in &r.episodes {
        inquiry_traces(e, &mut row)?;
        row.ordinary_messages.extend(
            e.incident
                .testimony
                .iter()
                .map(|t| (e.id, t.message.clone(), t.recipients.clone())),
        );
        for d in &e.investigation {
            if !d.forced {
                increment(
                    &mut row.counters,
                    "answer_changed_choices",
                    u64::from(d.before_answer_choice.action != d.choice.action),
                );
                increment(
                    &mut row.counters,
                    "answer_triggered",
                    u64::from(
                        d.before_answer_choice.action == InvestigationAction::Decline
                            && matches!(d.choice.action, InvestigationAction::Inspect(_)),
                    ),
                );
                increment(
                    &mut row.counters,
                    "answer_avoided",
                    u64::from(
                        matches!(
                            d.before_answer_choice.action,
                            InvestigationAction::Inspect(_)
                        ) && d.choice.action == InvestigationAction::Decline,
                    ),
                );
                increment(
                    &mut row.counters,
                    "answer_redirected",
                    u64::from(
                        matches!(
                            d.before_answer_choice.action,
                            InvestigationAction::Inspect(_)
                        ) && matches!(d.choice.action, InvestigationAction::Inspect(_))
                            && d.before_answer_choice.action != d.choice.action,
                    ),
                );
            }
        }
        row.outcomes.push((
            e.id,
            e.incident.config.actor(),
            e.incident.verdict,
            e.incident.correct,
        ));
        increment(
            &mut row.counters,
            "ordinary_communication_rejections",
            e.incident
                .decisions
                .iter()
                .filter(|d| d.input.phase == Phase::Discussion && d.rejection.is_some())
                .count() as u64,
        );
        for family in &r.config.processes {
            let estimates: Vec<_> = e
                .investigation
                .iter()
                .flat_map(|d| &d.choice.social)
                .filter(|s| s.input.family == family.family)
                .map(|s| (s.effective_accuracy, s.effective_missing))
                .collect();
            increment(
                &mut row.counters,
                "recipient_assessment_disagreement",
                u64::from(
                    estimates
                        .first()
                        .is_some_and(|first| estimates.iter().any(|e| e != first)),
                ),
            );
        }
        for s in e
            .social_decisions
            .iter()
            .chain(e.responses.iter().filter_map(|d| d.social.as_ref()))
        {
            increment(&mut row.counters, "speech_opportunities", 1);
            increment(&mut row.counters, "eligible", u64::from(s.choice.eligible));
            increment(
                &mut row.counters,
                "withheld",
                u64::from(s.choice.intent == MethodIntent::Withheld),
            );
            increment(
                &mut row.counters,
                "method_rejections",
                u64::from(s.rejection.is_some()),
            );
            if let Some(m) = &s.message {
                increment(
                    &mut row.counters,
                    if s.origin == Origin::Selected {
                        "selected_reports"
                    } else {
                        "injected_reports"
                    },
                    1,
                );
                increment(
                    &mut row.counters,
                    &format!("intent_{:?}", s.choice.intent),
                    1,
                );
                increment(
                    &mut row.counters,
                    "corrections",
                    u64::from(m.revision.is_some() && !m.retract),
                );
                increment(&mut row.counters, "retractions", u64::from(m.retract));
                increment(
                    &mut row.counters,
                    "delivered",
                    s.deliveries.iter().filter(|d| !d.own).count() as u64,
                );
                increment(
                    &mut row.counters,
                    "undelivered_possible",
                    6 - s.deliveries.iter().filter(|d| !d.own).count() as u64,
                );
                increment(
                    &mut row.counters,
                    "wire_bytes",
                    serde_json::to_vec(m).map_err(|e| e.to_string())?.len() as u64,
                );
            }
            increment(
                &mut row.counters,
                "social_evictions",
                s.deliveries.iter().map(|d| d.evicted.len() as u64).sum(),
            );
            row.speech.push(SpeechTrace {
                reference: s.input.reference,
                observer: s.input.incident.incident.id,
                input_digest: canonical_digest(&s.input)?,
                personal: s.input.personal.summaries.clone(),
                choice: s.choice.clone(),
                origin: s.origin,
                rejection: s.rejection.clone(),
                wire: s.message.clone(),
                actual_origins: s.actual_origins.clone(),
                actual_depth: s.actual_depth,
                recipients: s
                    .deliveries
                    .iter()
                    .filter(|d| !d.own)
                    .map(|d| d.after.observer)
                    .collect(),
                evictions: s
                    .deliveries
                    .iter()
                    .flat_map(|d| d.evicted.iter().map(move |r| (d.after.observer, *r)))
                    .collect(),
            });
        }
        for d in &e.investigation {
            let delivery = d.delivery.map(|index| &e.deliveries[index]);
            increment(
                &mut row.counters,
                "investigation_changed_by_social",
                u64::from(d.choice.action != d.choice.personal_proposal.action && !d.forced),
            );
            increment(
                &mut row.counters,
                "triggered_by_social",
                u64::from(
                    d.choice.personal_proposal.action == InvestigationAction::Decline
                        && matches!(d.choice.action, InvestigationAction::Inspect(_))
                        && !d.forced,
                ),
            );
            increment(
                &mut row.counters,
                "avoided_by_social",
                u64::from(
                    matches!(
                        d.choice.personal_proposal.action,
                        InvestigationAction::Inspect(_)
                    ) && d.choice.action == InvestigationAction::Decline
                        && !d.forced,
                ),
            );
            increment(
                &mut row.counters,
                "investigation_rejections",
                u64::from(d.rejection.is_some()),
            );
            if let Some(a) = delivery {
                increment(
                    &mut row.counters,
                    match a.correct {
                        Some(true) => "correct_findings",
                        Some(false) => "wrong_findings",
                        None => "missing_findings",
                    },
                    1,
                );
            }
            row.choices.push(ChoiceTrace {
                before_answer_action: d.before_answer_choice.action,
                before_answer_digest: canonical_digest(&d.before_answer_input)?,
                episode: e.id,
                observer: d.input.local.id,
                input_digest: canonical_digest(&d.input)?,
                personal_action: d.choice.personal_proposal.action,
                selected_action: d.choice.action,
                forced: d.forced,
                balance: d.input.local.balance,
                considered: d.choice.considered.clone(),
                finding: delivery.map(|d| d.finding.clone()),
                correct: delivery.and_then(|d| d.correct),
                before: d.assessment.after.clone(),
                after: delivery
                    .map(|d| d.after_assessment.after.clone())
                    .unwrap_or_else(|| d.assessment.after.clone()),
                paid: delivery
                    .map(|d| d.balance_before - d.balance_after)
                    .unwrap_or(0),
            });
            for (method, estimate) in d.input.methods.iter().zip(&d.choice.social) {
                let source = r.config.incidents[usize::from(e.id)]
                    .artifacts
                    .iter()
                    .find(|a| a.descriptor.id == method.source.descriptor.id)
                    .unwrap();
                let process = r
                    .config
                    .processes
                    .iter()
                    .find(|p| p.family == method.family)
                    .unwrap();
                let selected = delivery
                    .is_some_and(|a| a.finding.source.id == source.descriptor.id)
                    && !d.forced;
                if let Some(label) = source.candidate.map(|x| x == e.incident.config.actor()) {
                    let personal = estimate.personal_accuracy.brier(label);
                    let effective = estimate.effective_accuracy.brier(label);
                    increment(
                        &mut row.counters,
                        "social_forecast_worse_exposures",
                        u64::from(effective > personal),
                    );
                    increment(
                        &mut row.counters,
                        "social_forecast_better_exposures",
                        u64::from(effective < personal),
                    );
                }
                increment(
                    &mut row.counters,
                    "duplicate_interpretation_exposures",
                    estimate
                        .interpretations
                        .iter()
                        .filter(|i| i.discounted)
                        .count() as u64,
                );
                let used: Vec<_> = estimate
                    .interpretations
                    .iter()
                    .filter(|i| i.weight > 0)
                    .collect();
                for (index, a) in used.iter().enumerate() {
                    for b in &used[index + 1..] {
                        if a.group != b.group
                            && roots.get(&a.message).is_some_and(|root| {
                                roots
                                    .get(&b.message)
                                    .is_some_and(|other| root.iter().any(|r| other.contains(r)))
                            })
                        {
                            increment(&mut row.counters, "false_independent_group_exposures", 1);
                            if estimate
                                .input
                                .memory
                                .received
                                .iter()
                                .filter(|m| m.reference == a.message || m.reference == b.message)
                                .all(|m| m.payload.parent.is_none())
                            {
                                increment(
                                    &mut row.counters,
                                    "hidden_false_corroboration_exposures",
                                    1,
                                );
                            }
                        }
                    }
                }
                let before = d
                    .before_answer_choice
                    .social
                    .iter()
                    .find(|x| x.input.family == method.family)
                    .unwrap();
                row.forecasts.push(Forecast {
                    before_answer_accuracy: before.effective_accuracy,
                    before_answer_missing: before.effective_missing,
                    episode: e.id,
                    observer: d.input.local.id,
                    source: source.descriptor.id,
                    family: method.family.clone(),
                    personal: estimate.input.personal,
                    personal_accuracy: estimate.personal_accuracy,
                    personal_missing: estimate.personal_missing,
                    effective_accuracy: estimate.effective_accuracy,
                    effective_missing: estimate.effective_missing,
                    positive: estimate.positive,
                    negative: estimate.negative,
                    interpretations: estimate.interpretations.clone(),
                    selected,
                    forced: delivery.is_some_and(|a| a.finding.source.id == source.descriptor.id)
                        && d.forced,
                    correct: source.candidate.map(|x| x == e.incident.config.actor()),
                    missing: source.candidate.is_none(),
                    actual_accuracy: process.accuracy(e.id),
                    input_digest: canonical_digest(&estimate.input)?,
                });
                if selected
                    && estimate.effective_accuracy != estimate.personal_accuracy
                    && source.candidate.is_some()
                {
                    let predicts_correct = estimate.effective_accuracy.numerator * 2
                        > estimate.effective_accuracy.denominator;
                    increment(
                        &mut row.counters,
                        "selected_social_direct_contradictions",
                        u64::from(
                            predicts_correct
                                != source
                                    .candidate
                                    .is_some_and(|x| x == e.incident.config.actor()),
                        ),
                    );
                }
            }
            if population == "diagnostic" && e.id == 2 && d.input.local.id == 5 {
                let variants = LISTENERS
                    .into_iter()
                    .map(|listener| {
                        let mut v = d.input.clone();
                        v.listener = listener;
                        let choice =
                            Policy::new(e10::PolicyKind::ExperienceAdjusted, r.config.session, 5)
                                .choose(&v);
                        (listener, choice)
                    })
                    .collect();
                row.matched.push(MatchedInterpretation {
                    episode: e.id,
                    observer: 5,
                    input_digest: canonical_digest(&(&d.input.personal, &d.input.social))?,
                    variants,
                });
            }
        }
        for f in &e.method_feedback {
            row.feedback.push(FeedbackTrace {
                episode: e.id,
                observer: f.observer,
                input_digest: canonical_digest(&f.effect.input)?,
                token: f.effect.input.received.clone(),
                actual_correct: f.correct,
                before: f.effect.input.memory.summaries.clone(),
                after: f.effect.after.summaries.clone(),
                result: f.effect.transport_effect.acceptance,
            });
        }
        for d in e
            .incident
            .decisions
            .iter()
            .filter(|d| d.input.phase == Phase::Judgment)
        {
            row.ballots.push((
                e.id,
                d.input.id,
                match d.choice.action {
                    e5::Action::Vote(c) => c,
                    _ => None,
                },
            ));
        }
    }
    increment(
        &mut row.counters,
        "final_social_bytes",
        serde_json::to_vec(&r.final_social_memory)
            .map_err(|e| e.to_string())?
            .len() as u64,
    );
    increment(
        &mut row.counters,
        "full_report_bytes",
        serde_json::to_vec(&r).map_err(|e| e.to_string())?.len() as u64,
    );
    increment(
        &mut row.counters,
        "direct_grade_samples",
        r.final_method_memory
            .iter()
            .flat_map(|m| m.summaries.values())
            .map(|c| u64::from(c.graded()))
            .sum(),
    );
    increment(
        &mut row.counters,
        "final_inquiry_bytes",
        serde_json::to_vec(&r.final_inquiry_memory)
            .map_err(|e| e.to_string())?
            .len() as u64,
    );
    Ok(row)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interval {
    pub defined_worlds: usize,
    pub mean: Option<i64>,
    pub low: Option<i64>,
    pub high: Option<i64>,
}
pub fn paired_interval(values: &[i64]) -> Interval {
    if values.is_empty() {
        return Interval {
            defined_worlds: 0,
            mean: None,
            low: None,
            high: None,
        };
    }
    let mean = values.iter().sum::<i64>() / values.len() as i64;
    let mut rng = Rng::new(12012);
    let mut samples = Vec::with_capacity(512);
    for _ in 0..512 {
        samples.push(
            (0..values.len())
                .map(|_| values[(rng.draw() % values.len() as u64) as usize])
                .sum::<i64>()
                / values.len() as i64,
        );
    }
    samples.sort();
    Interval {
        defined_worlds: values.len(),
        mean: Some(mean),
        low: Some(samples[12]),
        high: Some(samples[499]),
    }
}
fn ratio(a: u64, b: u64) -> Option<i64> {
    (a * 1_000_000).checked_div(b).map(|v| v as i64)
}
fn measures(o: &Outcome) -> BTreeMap<String, Option<i64>> {
    let total = o.ballots.iter().sum();
    let defined = o.ballots[0] + o.ballots[1];
    let verdicts = o.verdicts.iter().sum();
    let mut out = BTreeMap::from([
        (
            "correct_ballots".into(),
            Some(o.ballots[0] as i64 * 1_000_000),
        ),
        (
            "correct_verdicts".into(),
            Some(o.verdicts[0] as i64 * 1_000_000),
        ),
        ("coverage".into(), ratio(defined, total)),
        ("conditional_accuracy".into(), ratio(o.ballots[0], defined)),
        ("wrong_risk".into(), ratio(o.ballots[1], total)),
        (
            "verdict_coverage".into(),
            ratio(o.verdicts[0] + o.verdicts[1], verdicts),
        ),
        ("spent".into(), Some(o.spent as i64 * 1_000_000)),
    ]);
    for cost in [1, 2, 4] {
        for price in [0, 250000, 1000000] {
            out.insert(
                format!("utility_c{cost}_p{price}"),
                Some(
                    (o.verdicts[0] as i64 - cost * o.verdicts[1] as i64) * 1_000_000
                        - price * o.spent as i64,
                ),
            );
        }
    }
    out
}
pub fn forecast_scores(
    row: &Row,
    assessment: bool,
    scope: &str,
    personal: bool,
) -> e10::QualityScore {
    let mut result = e10::QualityScore::default();
    for f in row.forecasts.iter().filter(|f| {
        (!assessment || f.episode >= 4)
            && match scope {
                "selected" => f.selected,
                "uninspected" => !f.selected && !f.forced,
                _ => true,
            }
    }) {
        if let Some(label) = f.correct {
            result.accuracy.add(
                if personal {
                    f.personal_accuracy
                } else {
                    f.effective_accuracy
                },
                label,
            );
        }
        result.missingness.add(
            if personal {
                f.personal_missing
            } else {
                f.effective_missing
            },
            f.missing,
        );
    }
    result
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    pub a: String,
    pub b: String,
    pub regime: String,
    pub assessment: bool,
    pub worlds: usize,
    pub changed_reports: usize,
    pub changed_ballots: usize,
    pub correct_ballot_worlds: [usize; 3],
    pub estimates: BTreeMap<String, Interval>,
}
pub fn comparisons(rows: &[Row]) -> Vec<Comparison> {
    use Treatment::*;
    let contrasts = [
        (Targeted, Ordinary),
        (Targeted, Unsolicited),
        (Targeted, Unanswered),
        (Targeted, Random),
        (Targeted, Selective),
        (Targeted, Misleading),
        (Targeted, Ignored),
        (Unanswered, Ordinary),
        (Unsolicited, Ordinary),
        (Selective, Ordinary),
        (Misleading, Ordinary),
    ];
    let mut result = vec![];
    for (a, b) in contrasts {
        for regime in ["all", "stationary", "shifted"] {
            for assessment in [false, true] {
                let an = format!("{a:?}");
                let bn = format!("{b:?}");
                let mut values: BTreeMap<String, Vec<i64>> = BTreeMap::new();
                let mut item = Comparison {
                    a: an.clone(),
                    b: bn.clone(),
                    regime: regime.into(),
                    assessment,
                    worlds: 0,
                    changed_reports: 0,
                    changed_ballots: 0,
                    correct_ballot_worlds: [0; 3],
                    estimates: BTreeMap::new(),
                };
                for x in rows.iter().filter(|r| {
                    r.population == "heldout"
                        && r.case == an
                        && (regime == "all" || (regime == "shifted") == r.shifted)
                }) {
                    let y = rows
                        .iter()
                        .find(|r| r.population == "heldout" && r.seed == x.seed && r.case == bn)
                        .unwrap();
                    assert_eq!(x.environment, y.environment, "paired latent worlds differ");
                    item.worlds += 1;
                    item.changed_reports += usize::from(
                        x.ordinary_messages != y.ordinary_messages
                            || x.queries
                                .iter()
                                .filter_map(|q| q.choice.selected.as_ref())
                                .collect::<Vec<_>>()
                                != y.queries
                                    .iter()
                                    .filter_map(|q| q.choice.selected.as_ref())
                                    .collect::<Vec<_>>()
                            || x.replies
                                .iter()
                                .filter_map(|q| q.choice.selected.as_ref())
                                .collect::<Vec<_>>()
                                != y.replies
                                    .iter()
                                    .filter_map(|q| q.choice.selected.as_ref())
                                    .collect::<Vec<_>>()
                            || x.speech
                                .iter()
                                .filter_map(|s| s.wire.as_ref())
                                .collect::<Vec<_>>()
                                != y.speech
                                    .iter()
                                    .filter_map(|s| s.wire.as_ref())
                                    .collect::<Vec<_>>(),
                    );
                    item.changed_ballots += usize::from(x.ballots != y.ballots);
                    let xo = if assessment { &x.assessment } else { &x.full };
                    let yo = if assessment { &y.assessment } else { &y.full };
                    item.correct_ballot_worlds[match xo.ballots[0].cmp(&yo.ballots[0]) {
                        std::cmp::Ordering::Greater => 0,
                        std::cmp::Ordering::Less => 1,
                        _ => 2,
                    }] += 1;
                    for (key, v) in measures(xo) {
                        if let (Some(a), Some(b)) = (v, measures(yo)[&key]) {
                            values.entry(key).or_default().push(a - b);
                        }
                    }
                    for scope in ["all", "selected", "uninspected"] {
                        let xs = forecast_scores(x, assessment, scope, false);
                        let ys = forecast_scores(y, assessment, scope, false);
                        for (name, a, b) in [
                            ("accuracy", xs.accuracy.brier(), ys.accuracy.brier()),
                            ("missing", xs.missingness.brier(), ys.missingness.brier()),
                        ] {
                            if let (Some(a), Some(b)) = (a, b) {
                                values
                                    .entry(format!("brier_{name}_{scope}"))
                                    .or_default()
                                    .push(a as i64 - b as i64);
                            }
                        }
                    }
                }
                item.estimates = values
                    .into_iter()
                    .map(|(k, v)| (k, paired_interval(&v)))
                    .collect();
                result.push(item);
            }
        }
    }
    result
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub independent_worlds: usize,
    pub incidents: usize,
    pub replay_matches: usize,
    pub rows: Vec<Row>,
    pub comparisons: Vec<Comparison>,
    pub prediction_groups: BTreeMap<String, e10::QualityScore>,
}
pub fn prediction_groups(rows: &[Row]) -> BTreeMap<String, e10::QualityScore> {
    let mut groups: BTreeMap<String, e10::QualityScore> = BTreeMap::new();
    for r in rows.iter().filter(|r| r.population == "heldout") {
        for f in &r.forecasts {
            for assessment in [false, true] {
                if assessment && f.episode < 4 {
                    continue;
                }
                for scope in ["all", "selected", "uninspected"] {
                    if (scope == "selected" && !f.selected)
                        || (scope == "uninspected" && (f.selected || f.forced))
                    {
                        continue;
                    }
                    for grade in ["all".to_owned(), f.personal.graded().min(4).to_string()] {
                        for regime in ["all", if r.shifted { "shifted" } else { "stationary" }] {
                            for personal in [false, true] {
                                let key = format!(
                                    "{}/{regime}/{}/{scope}/{grade}/{}",
                                    r.case,
                                    if assessment { "assessment" } else { "full" },
                                    if personal { "personal" } else { "effective" }
                                );
                                let g = groups.entry(key).or_default();
                                if let Some(label) = f.correct {
                                    g.accuracy.add(
                                        if personal {
                                            f.personal_accuracy
                                        } else {
                                            f.effective_accuracy
                                        },
                                        label,
                                    );
                                }
                                g.missingness.add(
                                    if personal {
                                        f.personal_missing
                                    } else {
                                        f.effective_missing
                                    },
                                    f.missing,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    groups
}
pub fn evaluate_seeds(
    worlds: std::ops::Range<u64>,
    diagnostics: std::ops::Range<u64>,
) -> Result<Evaluation, String> {
    let independent_worlds = (worlds.end - worlds.start) as usize;
    let mut rows = vec![];
    for seed in worlds {
        for treatment in TREATMENTS {
            rows.push(row(
                "heldout",
                &format!("{treatment:?}"),
                seed,
                generated(seed, treatment),
            )?);
        }
    }
    for seed in diagnostics {
        for family in DIAGNOSTICS {
            for listener in LISTENERS {
                rows.push(row(
                    "diagnostic",
                    &format!("{family:?}/{listener:?}"),
                    seed,
                    diagnostic(family, seed, listener),
                )?);
            }
        }
        for (index, c) in origin_pair(seed)?.into_iter().enumerate() {
            rows.push(row(
                "message_origin",
                if index == 0 { "selected" } else { "injected" },
                seed,
                c,
            )?);
        }
        let mut absent = diagnostic(
            Diagnostic::AccurateRequested,
            seed,
            SocialKind::ProvenanceAwareSocial,
        );
        absent.request = [RequestKind::NeverAsk; 6];
        rows.push(row("request_control", "no_request", seed, absent)?);
        let mut no_reply = diagnostic(
            Diagnostic::AccurateRequested,
            seed,
            SocialKind::ProvenanceAwareSocial,
        );
        no_reply.response = [ResponseKind::NoReply; 6];
        rows.push(row("response_control", "silence", seed, no_reply)?);
    }
    let comparisons = comparisons(&rows);
    let prediction_groups = prediction_groups(&rows);
    Ok(Evaluation {
        version: "evaluation-012-v1".into(),
        independent_worlds,
        incidents: rows.len() * 8,
        replay_matches: rows.len(),
        rows,
        comparisons,
        prediction_groups,
    })
}
pub fn evaluate() -> Result<Evaluation, String> {
    evaluate_seeds(6000..6064, 0..2)
}
pub fn manifest(e: &Evaluation) -> Result<e7::ReplayManifest, String> {
    Ok(e7::ReplayManifest {
        version: "replay-manifest-012-v1".into(),
        series: e.rows.len(),
        incidents: e.incidents,
        evaluation_digest: canonical_digest(e)?,
        metric_digest: canonical_digest(&(&e.comparisons, &e.prediction_groups))?,
        series_sections: e.rows.iter().map(|r| r.digests.clone()).collect(),
    })
}
