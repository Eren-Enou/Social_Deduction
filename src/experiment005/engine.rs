use super::*;
use crate::{Phase, experiment004 as e4, verdict};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Root {
    Observation(usize),
    Briefing(u8),
    Construction(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub basis: Basis,
    pub evidence_parents: Vec<usize>,
    pub roots: Vec<Root>,
    pub depth: u8,
    pub revision_of: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Selected { decision: usize },
    Injected { injection: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    SincereAssertion,
    IntentionalContradiction,
    SincereRepair,
    SincereStillWrong,
    SincereRevision,
    UncertainRetraction,
    FaithfulAssertion,
    FaithfulCorrection,
    Withheld,
    Silence,
    Uncertainty,
    Ballot,
    Unclassifiable,
    Rejected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Testimony {
    pub message: Message,
    pub recipients: Vec<u8>,
    pub origin: Origin,
    pub lineage: Lineage,
    pub intent: Intent,
    pub belief: Option<u8>,
    pub classification: Classification,
    pub correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub input: ListenerView,
    pub assessment: Assessment,
    pub before_correct: Option<bool>,
    pub after_correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub update: usize,
    pub input: AgentView,
    pub choice: Choice,
    pub rejection: Option<String>,
    pub classification: Classification,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Transition(Phase),
    Incident {
        actor: u8,
    },
    Assigned {
        agent: u8,
        original: IndividualState,
        observation: Observation,
    },
    Observed {
        intervention: usize,
        agent: u8,
        observation: Observation,
    },
    Injected {
        injection: usize,
        message: usize,
    },
    Updated {
        update: usize,
    },
    Decided {
        decision: usize,
    },
    Delivered {
        message: usize,
        recipient: u8,
    },
    Resolved {
        decision: usize,
        accepted: bool,
        message: Option<usize>,
        ballot: Option<u8>,
    },
    Outcome {
        tally: [u8; 6],
        verdict: Option<u8>,
        correct: Option<bool>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub rules_version: String,
    pub revision_version: String,
    pub listener_version: String,
    pub policy_version: String,
    pub implementation: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub updates: Vec<Update>,
    pub decisions: Vec<Decision>,
    pub testimony: Vec<Testimony>,
    pub injection_inputs: Vec<AgentView>,
    pub tally: [u8; 6],
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
    pub retained_received_copies: usize,
    pub retained_sent_copies: usize,
}
pub struct Policies {
    pub listeners: Vec<Box<dyn Listener>>,
    pub speakers: Vec<Box<dyn CommunicationPolicy>>,
    pub judges: Vec<Box<dyn JudgmentPolicy>>,
}
impl Policies {
    pub fn from_config(c: &Config) -> Self {
        Self {
            listeners: c
                .listeners
                .iter()
                .map(|&p| Box::new(p) as Box<dyn Listener>)
                .collect(),
            speakers: c
                .speakers
                .iter()
                .map(|&p| Box::new(SpeakerPolicy::new(p)) as Box<dyn CommunicationPolicy>)
                .collect(),
            judges: c
                .judgments
                .iter()
                .map(|&p| Box::new(p) as Box<dyn JudgmentPolicy>)
                .collect(),
        }
    }
}
pub fn validate_action(v: &AgentView, a: &Action) -> Result<(), String> {
    if v.opportunities_left == 0 {
        return Err("budget exhausted".into());
    }
    match (v.phase, a) {
        (Phase::Discussion, Action::Silence) => Ok(()),
        (Phase::Judgment, Action::Vote(c)) if c.is_none_or(|c| v.roster.contains(&c)) => Ok(()),
        (
            Phase::Discussion,
            Action::Speak {
                body,
                attribution,
                basis,
                recipients,
            },
        ) => {
            if body.candidate().is_some_and(|c| !v.roster.contains(&c))
                || !e4::unique_ids(recipients)
                || recipients
                    .iter()
                    .any(|r| !v.permitted_recipients.contains(r))
            {
                return Err("invalid candidate or recipients".into());
            }
            match attribution {
                Attribution::NamedSource(id) if !v.roster.contains(id) => {
                    return Err("unknown named source".into());
                }
                Attribution::Reference(id) if !v.received.iter().any(|m| m.id == *id) => {
                    return Err("attribution reference not received".into());
                }
                _ => {}
            }
            if let Body::Revise {
                target, forwarded, ..
            } = body
            {
                if !v.sent.iter().any(|m| {
                    m.id == *target
                        && m.speaker == v.id
                        && matches!(m.body, Body::Assert(_) | Body::Revise { .. })
                }) {
                    return Err("revision target must be an own earlier assertion/revision".into());
                }
                if let Some(id) = forwarded
                    && !v.received.iter().any(|m| {
                        m.id == *id
                            && matches!(m.body, Body::Revise { .. })
                            && m.body.candidate() == body.candidate()
                    })
                {
                    return Err("forwarded revision must be a received exact replacement".into());
                }
            }
            match basis {
                Basis::Private => {
                    let own = effective_original(&v.original, &v.observations);
                    if body.candidate().is_none()
                        || (body.candidate() != own.observation
                            && !(own.briefing.is_some() && body.candidate() == Some(v.id)))
                    {
                        return Err("unsupported private basis".into());
                    }
                }
                Basis::ReceivedAssertion(id) => {
                    if !matches!(body, Body::Assert(_))
                        || !v.received.iter().any(|m| m.id == *id && m.body == *body)
                    {
                        return Err("assertion parent not received or not an exact copy".into());
                    }
                }
                Basis::ReceivedRevision(id) => {
                    if !matches!(body,Body::Revise {forwarded:Some(f),..} if f==id)
                        || !v.received.iter().any(|m| {
                            m.id == *id
                                && matches!(m.body, Body::Revise { .. })
                                && m.body.candidate() == body.candidate()
                        })
                    {
                        return Err("revision parent not received or not an exact copy".into());
                    }
                }
                Basis::Inference => {}
            }
            Ok(())
        }
        _ => Err("illegal phase or action".into()),
    }
}
fn private_root(v: &AgentView, c: u8) -> Option<Root> {
    if v.original.briefing.is_some() && c == v.id {
        Some(Root::Briefing(v.id))
    } else {
        v.observations
            .last()
            .filter(|o| o.candidate == Some(c))
            .map(|o| Root::Observation(o.acquisition))
    }
}
pub fn causal_lineage(
    v: &AgentView,
    choice: &Choice,
    a: &Assessment,
    history: &[Testimony],
) -> Result<Lineage, String> {
    let Action::Speak { body, basis, .. } = &choice.action else {
        return Err("lineage requires speech".into());
    };
    let mut roots = BTreeSet::new();
    let mut parents = vec![];
    match basis {
        Basis::Private => {
            roots.insert(
                private_root(v, body.candidate().ok_or("private assertion needed")?)
                    .ok_or("missing private acquisition")?,
            );
        }
        Basis::ReceivedAssertion(id) | Basis::ReceivedRevision(id) => {
            if !v.received.iter().any(|m| m.id == *id) {
                return Err("inaccessible evidence parent".into());
            }
            let parent = history.get(*id).ok_or("unknown evidence parent")?;
            parents.push(*id);
            roots.extend(parent.lineage.roots.iter().copied());
        }
        Basis::Inference => {
            if let Some(c) = v.current.candidate {
                if let Some(r) = private_root(v, c) {
                    roots.insert(r);
                }
                for i in &a.interpretations {
                    if i.standing == Standing::Active
                        && a.groups
                            .iter()
                            .any(|g| g.contributes && Some(g.group) == i.group)
                        && v.received
                            .iter()
                            .any(|m| m.id == i.message && m.body.candidate() == Some(c))
                    {
                        let parent = history
                            .get(i.message)
                            .ok_or("missing contributing testimony")?;
                        parents.push(i.message);
                        roots.extend(parent.lineage.roots.iter().copied());
                    }
                }
            }
            if roots.is_empty() {
                roots.insert(Root::Construction(history.len()));
            }
        }
    }
    parents.sort_unstable();
    parents.dedup();
    let depth = parents
        .iter()
        .map(|&id| history[id].lineage.depth + 1)
        .max()
        .unwrap_or(0);
    if depth > 3 {
        return Err("evidence depth exceeds three".into());
    }
    Ok(Lineage {
        basis: *basis,
        evidence_parents: parents,
        roots: roots.into_iter().collect(),
        depth,
        revision_of: body.target(),
    })
}
pub fn classify(
    v: &AgentView,
    c: &Choice,
    actor: u8,
    history: &[Testimony],
    accepted: bool,
) -> Classification {
    if !accepted {
        return Classification::Rejected;
    }
    match &c.action {
        Action::Vote(_) => Classification::Ballot,
        Action::Silence => {
            if c.intent == Intent::Withhold && (c.recognized_change || c.eligible_forward.is_some())
            {
                Classification::Withheld
            } else {
                Classification::Silence
            }
        }
        Action::Speak { body, basis, .. } => {
            if c.intent == Intent::Faithful {
                return match (body, basis) {
                    (Body::Assert(_), Basis::ReceivedAssertion(_)) => {
                        Classification::FaithfulAssertion
                    }
                    (Body::Revise { .. }, Basis::ReceivedRevision(_)) => {
                        Classification::FaithfulCorrection
                    }
                    _ => Classification::Unclassifiable,
                };
            }
            if c.intent == Intent::Contrary
                && v.current.candidate.is_some()
                && body.candidate().is_some()
                && body.candidate() != v.current.candidate
            {
                return Classification::IntentionalContradiction;
            }
            if c.intent != Intent::Sincere || body.candidate() != v.current.candidate {
                return Classification::Unclassifiable;
            }
            match body {
                Body::Uncertain => Classification::Uncertainty,
                Body::Assert(_) => Classification::SincereAssertion,
                Body::Revise {
                    target,
                    replacement,
                    ..
                } => match replacement {
                    None => Classification::UncertainRetraction,
                    Some(c) if *c != actor => Classification::SincereStillWrong,
                    Some(_) => {
                        if history
                            .get(*target)
                            .is_some_and(|t| t.correct == Some(false))
                        {
                            Classification::SincereRepair
                        } else {
                            Classification::SincereRevision
                        }
                    }
                },
            }
        }
    }
}
fn deliver(
    report: &mut Report,
    received: &mut [Vec<Message>],
    sent: &mut [Vec<Message>],
    v: &AgentView,
    c: &Choice,
    lineage: Lineage,
    origin: Origin,
) {
    let Action::Speak {
        body,
        attribution,
        recipients,
        ..
    } = &c.action
    else {
        unreachable!()
    };
    let id = report.testimony.len();
    let message = Message {
        id,
        speaker: v.id,
        body: body.clone(),
        attribution: *attribution,
    };
    let classification = classify(v, c, report.config.actor(), &report.testimony, true);
    report.testimony.push(Testimony {
        message: message.clone(),
        recipients: recipients.clone(),
        origin,
        lineage,
        intent: c.intent,
        belief: v.current.candidate,
        classification,
        correct: body.candidate().map(|c| c == report.config.actor()),
    });
    sent[usize::from(v.id)].push(message.clone());
    for &recipient in recipients {
        received[usize::from(recipient)].push(message.clone());
        report.objective_history.push(Event::Delivered {
            message: id,
            recipient,
        });
    }
}
pub fn run(config: Config) -> Result<Report, String> {
    let mut p = Policies::from_config(&config);
    let mut r = run_with(config, &mut p)?;
    r.implementation = "builtin-005-v1".into();
    Ok(r)
}
pub fn run_with(config: Config, p: &mut Policies) -> Result<Report, String> {
    config.validate()?;
    if [p.listeners.len(), p.speakers.len(), p.judges.len()]
        .iter()
        .any(|&n| n != 6)
    {
        return Err("six policies per mechanism required".into());
    }
    let actor = config.actor();
    let original: Vec<_> = (0..6)
        .map(|id| IndividualState {
            observation: config.observations[id],
            briefing: if config.participation_briefing && id as u8 == actor {
                Some(Briefing::Participated)
            } else {
                None
            },
            belief: if config.participation_briefing && id as u8 == actor {
                Some(id as u8)
            } else {
                config.observations[id]
            },
            incentives: config.incentives[id],
        })
        .collect();
    let mut observations: Vec<_> = (0..6)
        .map(|id| {
            vec![Observation {
                acquisition: id,
                candidate: config.observations[id],
            }]
        })
        .collect();
    let mut current: Vec<_> = original
        .iter()
        .enumerate()
        .map(|(id, o)| initial_belief(id as u8, o))
        .collect();
    let mut received = vec![vec![]; 6];
    let mut sent = vec![vec![]; 6];
    let mut r = Report {
        rules_version: "incident-005-v1".into(),
        revision_version: "revision-005-v1".into(),
        listener_version: "revision-listener-005-v1 + frozen provenance-group-004-v1".into(),
        policy_version: "correction-policy-005-v1".into(),
        implementation: "caller-supplied-unversioned".into(),
        config: config.clone(),
        objective_history: vec![
            Event::Transition(Phase::Incident),
            Event::Incident { actor },
            Event::Transition(Phase::Observation),
        ],
        updates: vec![],
        decisions: vec![],
        testimony: vec![],
        injection_inputs: vec![],
        tally: [0; 6],
        verdict: None,
        correct: None,
        retained_received_copies: 0,
        retained_sent_copies: 0,
    };
    for id in 0..6 {
        r.objective_history.push(Event::Assigned {
            agent: id as u8,
            original: original[id].clone(),
            observation: observations[id][0].clone(),
        });
    }
    for slot in 0..18 {
        let phase = if slot < 12 {
            Phase::Discussion
        } else {
            Phase::Judgment
        };
        if slot == 0 || slot == 12 {
            r.objective_history.push(Event::Transition(phase));
        }
        for (index, o) in config
            .later
            .iter()
            .enumerate()
            .filter(|(_, o)| o.before_slot == slot)
        {
            let observation = Observation {
                acquisition: 6 + index,
                candidate: o.candidate,
            };
            observations[usize::from(o.agent)].push(observation.clone());
            r.objective_history.push(Event::Observed {
                intervention: index,
                agent: o.agent,
                observation,
            });
        }
        for (index, i) in config
            .injections
            .iter()
            .enumerate()
            .filter(|(_, i)| i.before_slot == slot)
        {
            let id = usize::from(i.speaker);
            let view = AgentView {
                id: i.speaker,
                roster: (0..6).collect(),
                phase: Phase::Discussion,
                opportunities_left: 1,
                original: original[id].clone(),
                observations: observations[id].clone(),
                current: current[id].clone(),
                received: received[id].clone(),
                sent: sent[id].clone(),
                permitted_recipients: if slot < 6 {
                    config.routes[id].clone()
                } else {
                    config.second_routes[id].clone()
                },
                revision_benefit: config.revision_benefit[id],
                revision_cost: config.revision_cost[id],
            };
            let mut view = view;
            let mut listener = config.listeners[id];
            let assessment = listener.assess(&as_listener_view(&view));
            view.current = assessment.after.clone();
            validate_action(&view, &i.choice.action)
                .map_err(|e| format!("injection {index}: {e}"))?;
            let lineage = causal_lineage(&view, &i.choice, &assessment, &r.testimony)?;
            r.injection_inputs.push(view.clone());
            r.objective_history.push(Event::Injected {
                injection: index,
                message: r.testimony.len(),
            });
            deliver(
                &mut r,
                &mut received,
                &mut sent,
                &view,
                &i.choice,
                lineage,
                Origin::Injected { injection: index },
            );
        }
        let agent = config.order[slot % 6];
        let id = usize::from(agent);
        let input = ListenerView {
            id: agent,
            original: original[id].clone(),
            observations: observations[id].clone(),
            prior: current[id].clone(),
            received: received[id].clone(),
        };
        let assessment = p.listeners[id].assess(&input);
        if Belief::from_scores(assessment.after.scores) != assessment.after
            || assessment.interpretations.len() != input.received.len()
            || assessment
                .interpretations
                .iter()
                .map(|a| a.message)
                .collect::<BTreeSet<_>>()
                != input.received.iter().map(|m| m.id).collect::<BTreeSet<_>>()
            || assessment.interpretations.iter().any(|a| {
                a.checked
                    .iter()
                    .any(|id| !input.received.iter().any(|m| m.id == *id))
            })
        {
            return Err("inconsistent or inaccessible listener assessment".into());
        }
        let update = r.updates.len();
        r.updates.push(Update {
            before_correct: input.prior.candidate.map(|c| c == actor),
            after_correct: assessment.after.candidate.map(|c| c == actor),
            input,
            assessment: assessment.clone(),
        });
        r.objective_history.push(Event::Updated { update });
        current[id] = assessment.after.clone();
        let view = AgentView {
            id: agent,
            roster: (0..6).collect(),
            phase,
            opportunities_left: if slot < 6 { 2 } else { 1 },
            original: original[id].clone(),
            observations: observations[id].clone(),
            current: current[id].clone(),
            received: received[id].clone(),
            sent: sent[id].clone(),
            permitted_recipients: if slot < 6 {
                config.routes[id].clone()
            } else {
                config.second_routes[id].clone()
            },
            revision_benefit: config.revision_benefit[id],
            revision_cost: config.revision_cost[id],
        };
        let choice = if slot < 12 {
            p.speakers[id].choose(&view)
        } else {
            p.judges[id].choose(&view)
        };
        let mut rejection = validate_action(&view, &choice.action).err();
        let mut lineage = None;
        if rejection.is_none() && matches!(choice.action, Action::Speak { .. }) {
            match causal_lineage(&view, &choice, &assessment, &r.testimony) {
                Ok(l) => lineage = Some(l),
                Err(e) => rejection = Some(e),
            }
        }
        let accepted = rejection.is_none();
        let classification = classify(&view, &choice, actor, &r.testimony, accepted);
        let decision = r.decisions.len();
        r.decisions.push(Decision {
            update,
            input: view.clone(),
            choice: choice.clone(),
            rejection,
            classification,
        });
        r.objective_history.push(Event::Decided { decision });
        let mut message = None;
        let mut ballot = None;
        if accepted {
            match choice.action {
                Action::Speak { .. } => {
                    message = Some(r.testimony.len());
                    deliver(
                        &mut r,
                        &mut received,
                        &mut sent,
                        &view,
                        &choice,
                        lineage.unwrap(),
                        Origin::Selected { decision },
                    );
                }
                Action::Vote(Some(c)) => {
                    r.tally[usize::from(c)] += 1;
                    ballot = Some(c)
                }
                _ => {}
            }
        }
        r.objective_history.push(Event::Resolved {
            decision,
            accepted,
            message,
            ballot,
        });
    }
    r.retained_received_copies = received.iter().map(Vec::len).sum();
    r.retained_sent_copies = sent.iter().map(Vec::len).sum();
    r.verdict = verdict(r.tally);
    r.correct = r.verdict.map(|c| c == actor);
    r.objective_history
        .push(Event::Transition(Phase::Terminated));
    r.objective_history.push(Event::Outcome {
        tally: r.tally,
        verdict: r.verdict,
        correct: r.correct,
    });
    Ok(r)
}
