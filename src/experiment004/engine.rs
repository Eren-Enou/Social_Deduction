use super::*;
use crate::{N, Phase, experiment002 as e2, verdict};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Root {
    Observation(u8),
    Briefing(u8),
    Construction(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub basis: Basis,
    pub parents: Vec<usize>,
    pub roots: Vec<Root>,
    pub depth: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Selected { decision: usize },
    Injected { injection: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationClass {
    SincereTruth,
    SincereMistake,
    IntentionalContradiction,
    FaithfulRelay,
    MisattributedRelay,
    Silence,
    Uncertainty,
    Ballot,
    Unsubstantiated,
    Rejected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Testimony {
    pub message: Message,
    pub recipients: Vec<u8>,
    pub origin: Origin,
    pub lineage: Lineage,
    pub strategy: Strategy,
    pub speaker_belief: Option<u8>,
    pub classification: CommunicationClass,
    pub matches_truth: Option<bool>,
    pub attribution_compatible: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub input: ListenerView,
    pub assessment: Assessment,
    pub observer_received_roots: Vec<Root>,
    pub before_correct: Option<bool>,
    pub after_correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub update: usize,
    pub input: AgentView,
    pub choice: Choice,
    pub rejection: Option<String>,
    pub classification: CommunicationClass,
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
        initial: Belief,
    },
    Injected {
        injection: usize,
        message: usize,
    },
    Delivered {
        message: usize,
        recipient: u8,
    },
    Updated {
        update: usize,
    },
    DecisionMade {
        decision: usize,
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
    pub provenance_version: String,
    pub listener_version: String,
    pub relay_version: String,
    pub implementation: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub updates: Vec<Update>,
    pub decisions: Vec<Decision>,
    pub testimony: Vec<Testimony>,
    pub tally: [u8; 6],
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
    pub retained_message_copies: usize,
}
pub struct Policies {
    pub listeners: Vec<Box<dyn Listener>>,
    pub speakers: Vec<Box<dyn CommunicationPolicy>>,
    pub judges: Vec<Box<dyn JudgmentPolicy>>,
}
impl Policies {
    pub fn from_config(config: &Config) -> Self {
        Self {
            listeners: config
                .listeners
                .iter()
                .map(|&p| Box::new(p) as Box<dyn Listener>)
                .collect(),
            speakers: config
                .speakers
                .iter()
                .map(|&p| Box::new(SpeakerPolicy::new(p)) as Box<dyn CommunicationPolicy>)
                .collect(),
            judges: config
                .judgments
                .iter()
                .map(|&p| Box::new(p) as Box<dyn JudgmentPolicy>)
                .collect(),
        }
    }
}
pub fn validate_action(view: &AgentView, action: &Action) -> Result<(), String> {
    if view.opportunities_left == 0 {
        return Err("budget exhausted".into());
    }
    match (view.phase, action) {
        (Phase::Discussion, Action::Silence) => Ok(()),
        (Phase::Judgment, Action::Vote(candidate))
            if candidate.is_none_or(|id| view.roster.contains(&id)) =>
        {
            Ok(())
        }
        (
            Phase::Discussion,
            Action::Speak {
                speech,
                attribution,
                basis,
                recipients,
            },
        ) => {
            if matches!(speech,Speech::Assert(id) if !view.roster.contains(id))
                || !unique_ids(recipients)
                || recipients
                    .iter()
                    .any(|id| !view.permitted_recipients.contains(id))
            {
                return Err("illegal proposition or recipients".into());
            }
            if matches!(attribution,Attribution::NamedSource(id) if !view.roster.contains(id)) {
                return Err("unknown claimed source".into());
            }
            if let Attribution::Reference(reference) = attribution
                && !view.messages.iter().any(|m| m.id == *reference)
            {
                return Err("claimed message reference was not received".into());
            }
            match basis {
                Basis::Received(parent) => {
                    let Some(message) = view.messages.iter().find(|m| m.id == *parent) else {
                        return Err("relay parent was not received".into());
                    };
                    if !matches!(speech, Speech::Assert(_)) || message.speech != *speech {
                        return Err(
                            "relay must copy an asserted received proposition exactly".into()
                        );
                    }
                }
                Basis::PrivateObservation => {
                    if !matches!(speech,Speech::Assert(id) if view.original.observation==Some(*id) || (view.original.briefing.is_some() && *id==view.id))
                    {
                        return Err("private basis does not support proposition".into());
                    }
                }
                Basis::Inference => {}
            }
            Ok(())
        }
        _ => Err("illegal phase or target".into()),
    }
}
fn root_for_private(view: &AgentView, candidate: u8) -> Option<Root> {
    if view.original.briefing.is_some() && candidate == view.id {
        Some(Root::Briefing(view.id))
    } else if view.original.observation == Some(candidate) {
        Some(Root::Observation(view.id))
    } else {
        None
    }
}
pub fn causal_lineage(
    view: &AgentView,
    choice: &Choice,
    assessment: Option<&Assessment>,
    history: &[Testimony],
) -> Result<Lineage, String> {
    let Action::Speak { basis, speech, .. } = &choice.action else {
        return Err("lineage requires speech".into());
    };
    let mut roots = BTreeSet::new();
    let mut parents = vec![];
    match basis {
        Basis::PrivateObservation => {
            let Speech::Assert(candidate) = speech else {
                return Err("private assertion required".into());
            };
            roots.insert(
                root_for_private(view, *candidate)
                    .ok_or("no private acquisition supports this assertion")?,
            );
        }
        Basis::Received(parent) => {
            if !view.messages.iter().any(|m| m.id == *parent) {
                return Err("inaccessible relay parent".into());
            }
            let Some(actual) = history.get(*parent) else {
                return Err("unknown or forward relay parent".into());
            };
            parents.push(*parent);
            roots.extend(actual.lineage.roots.iter().copied());
        }
        Basis::Inference => {
            if let Some(candidate) = view.current.candidate {
                if let Some(root) = root_for_private(view, candidate) {
                    roots.insert(root);
                }
                if let Some(assessment) = assessment {
                    for source in &assessment.sources {
                        if source.treatment == Treatment::Accepted
                            && view.messages.iter().any(|m| {
                                m.id == source.message && m.speech == Speech::Assert(candidate)
                            })
                        {
                            let actual = history
                                .get(source.message)
                                .ok_or("missing actual cause for accepted received claim")?;
                            parents.push(source.message);
                            roots.extend(actual.lineage.roots.iter().copied());
                        }
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
    if depth > MAX_DEPTH {
        return Err("causal chain exceeds depth three".into());
    }
    Ok(Lineage {
        basis: *basis,
        parents,
        roots: roots.into_iter().collect(),
        depth,
    })
}
pub fn classify(
    view: &AgentView,
    choice: &Choice,
    actor: u8,
    accepted: bool,
) -> CommunicationClass {
    if !accepted {
        return CommunicationClass::Rejected;
    }
    if matches!(
        choice.action,
        Action::Speak {
            basis: Basis::Received(_),
            ..
        }
    ) {
        match choice.strategy {
            Strategy::RelayReceived | Strategy::NamedUnverified => {
                return CommunicationClass::FaithfulRelay;
            }
            Strategy::MisattributeSource => return CommunicationClass::MisattributedRelay,
            _ => {}
        }
    }
    let Strategy::Baseline(intent) = choice.strategy else {
        return CommunicationClass::Unsubstantiated;
    };
    let action = match &choice.action {
        Action::Silence => e2::Action::Silence,
        Action::Vote(candidate) => e2::Action::Vote(*candidate),
        Action::Speak { speech, .. } => e2::Action::Speak {
            speech: speech.clone(),
            response_to: None,
        },
    };
    let classified = e2::classify(
        &view.baseline_view(),
        &e2::Choice {
            action,
            intent,
            scores: vec![],
            mechanism: choice.mechanism.clone(),
        },
        actor,
        true,
    );
    match classified {
        e2::Classification::SincereTruth => CommunicationClass::SincereTruth,
        e2::Classification::SincereMistake => CommunicationClass::SincereMistake,
        e2::Classification::IntentionalContradiction => {
            CommunicationClass::IntentionalContradiction
        }
        e2::Classification::Silence => CommunicationClass::Silence,
        e2::Classification::Uncertainty => CommunicationClass::Uncertainty,
        e2::Classification::Ballot => CommunicationClass::Ballot,
        _ => CommunicationClass::Unsubstantiated,
    }
}
fn build_testimony(
    view: &AgentView,
    choice: &Choice,
    actor: u8,
    history: &[Testimony],
    lineage: Lineage,
    origin: Origin,
) -> Testimony {
    let Action::Speak {
        speech,
        attribution,
        recipients,
        ..
    } = &choice.action
    else {
        unreachable!()
    };
    let attribution_compatible = match attribution {
        Attribution::Unspecified => None,
        Attribution::Firsthand => Some(lineage.basis == Basis::PrivateObservation),
        Attribution::NamedSource(id) => Some(
            lineage.roots.len() == 1
                && matches!(lineage.roots[0],Root::Observation(source)|Root::Briefing(source) if source==*id),
        ),
        Attribution::Reference(parent) => history
            .get(*parent)
            .map(|source| source.lineage.roots == lineage.roots),
    };
    Testimony {
        message: Message {
            id: history.len(),
            speaker: view.id,
            speech: speech.clone(),
            attribution: *attribution,
        },
        recipients: recipients.clone(),
        origin,
        lineage,
        strategy: choice.strategy,
        speaker_belief: view.current.candidate,
        classification: classify(view, choice, actor, true),
        matches_truth: match speech {
            Speech::Assert(id) => Some(*id == actor),
            Speech::Uncertain => None,
        },
        attribution_compatible,
    }
}
fn deliver(report: &mut Report, received: &mut [Vec<Message>], testimony: Testimony) {
    let id = testimony.message.id;
    report.testimony.push(testimony);
    for &recipient in &report.testimony[id].recipients {
        received[usize::from(recipient)].push(report.testimony[id].message.clone());
        report.objective_history.push(Event::Delivered {
            message: id,
            recipient,
        });
    }
}
pub fn run(config: Config) -> Result<Report, String> {
    let mut policies = Policies::from_config(&config);
    let mut report = run_with(config, &mut policies)?;
    report.implementation = "builtin-004-v1".into();
    Ok(report)
}
pub fn run_with(config: Config, policies: &mut Policies) -> Result<Report, String> {
    config.validate()?;
    if [
        policies.listeners.len(),
        policies.speakers.len(),
        policies.judges.len(),
    ]
    .iter()
    .any(|&n| n != 6)
    {
        return Err("six policies of each kind required".into());
    }
    let actor = config.actor();
    let mut original = vec![];
    for id in 0..N {
        let briefing = if config.participation_briefing && id == actor {
            Some(Briefing::Participated)
        } else {
            None
        };
        original.push(IndividualState {
            observation: config.observations[usize::from(id)],
            briefing,
            belief: if briefing.is_some() {
                Some(id)
            } else {
                config.observations[usize::from(id)]
            },
            incentives: config.incentives[usize::from(id)],
        });
    }
    let mut current: Vec<_> = original
        .iter()
        .enumerate()
        .map(|(id, o)| initial_belief(id as u8, o))
        .collect();
    let mut received = vec![vec![]; 6];
    let mut report = Report {
        rules_version: "incident-004-v1".into(),
        provenance_version: "provenance-004-v1".into(),
        listener_version: "provenance-group-004-v1 + source-count-003-v1".into(),
        relay_version: "relay-004-v1".into(),
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
        tally: [0; 6],
        verdict: None,
        correct: None,
        retained_message_copies: 0,
    };
    for id in 0..N {
        report.objective_history.push(Event::Assigned {
            agent: id,
            original: original[usize::from(id)].clone(),
            initial: current[usize::from(id)].clone(),
        });
    }
    for (index, injection) in config.injections.iter().enumerate() {
        let id = usize::from(injection.speaker);
        let view = AgentView {
            id: injection.speaker,
            roster: (0..N).collect(),
            phase: Phase::Discussion,
            opportunities_left: 1,
            original: original[id].clone(),
            current: current[id].clone(),
            messages: received[id].clone(),
            permitted_recipients: config.routes[id].clone(),
        };
        validate_action(&view, &injection.choice.action)
            .map_err(|e| format!("injection {index}: {e}"))?;
        let mut listener = config.listeners[id];
        let assessment = listener.assess(&as_listener_view(&view));
        let lineage = causal_lineage(
            &view,
            &injection.choice,
            Some(&assessment),
            &report.testimony,
        )?;
        let mid = report.testimony.len();
        report.objective_history.push(Event::Injected {
            injection: index,
            message: mid,
        });
        let testimony = build_testimony(
            &view,
            &injection.choice,
            actor,
            &report.testimony,
            lineage,
            Origin::Injected { injection: index },
        );
        deliver(&mut report, &mut received, testimony);
    }
    for phase in [Phase::Discussion, Phase::Judgment] {
        report.objective_history.push(Event::Transition(phase));
        let rounds = if phase == Phase::Discussion { 2 } else { 1 };
        for round in 0..rounds {
            for agent in config.order {
                let id = usize::from(agent);
                let input = ListenerView {
                    id: agent,
                    original: original[id].clone(),
                    prior: current[id].clone(),
                    messages: received[id].clone(),
                };
                let assessment = policies.listeners[id].assess(&input);
                if Belief::from_scores(assessment.after.scores) != assessment.after
                    || assessment.sources.len() != input.messages.len()
                    || assessment
                        .sources
                        .iter()
                        .any(|s| !input.messages.iter().any(|m| m.id == s.message))
                    || assessment
                        .sources
                        .iter()
                        .map(|s| s.message)
                        .collect::<BTreeSet<_>>()
                        .len()
                        != assessment.sources.len()
                {
                    return Err(
                        "inconsistent listener output or inaccessible assessment reference".into(),
                    );
                }
                let observer_received_roots = input
                    .messages
                    .iter()
                    .filter(|m| m.speaker != agent && matches!(m.speech, Speech::Assert(_)))
                    .flat_map(|m| report.testimony[m.id].lineage.roots.iter().copied())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                let update = report.updates.len();
                report.updates.push(Update {
                    before_correct: input.prior.candidate.map(|c| c == actor),
                    after_correct: assessment.after.candidate.map(|c| c == actor),
                    input,
                    assessment: assessment.clone(),
                    observer_received_roots,
                });
                report.objective_history.push(Event::Updated { update });
                current[id] = assessment.after.clone();
                let input = AgentView {
                    id: agent,
                    roster: (0..N).collect(),
                    phase,
                    opportunities_left: rounds - round,
                    original: original[id].clone(),
                    current: current[id].clone(),
                    messages: received[id].clone(),
                    permitted_recipients: config.routes[id].clone(),
                };
                let choice = if phase == Phase::Discussion {
                    policies.speakers[id].choose(&input)
                } else {
                    policies.judges[id].choose(&input)
                };
                let mut rejection = validate_action(&input, &choice.action).err();
                let mut lineage = None;
                if rejection.is_none() && matches!(choice.action, Action::Speak { .. }) {
                    match causal_lineage(&input, &choice, Some(&assessment), &report.testimony) {
                        Ok(value) => lineage = Some(value),
                        Err(error) => rejection = Some(error),
                    }
                }
                let accepted = rejection.is_none();
                let decision = report.decisions.len();
                let classification = classify(&input, &choice, actor, accepted);
                report.decisions.push(Decision {
                    update,
                    input: input.clone(),
                    choice: choice.clone(),
                    rejection,
                    classification,
                });
                report
                    .objective_history
                    .push(Event::DecisionMade { decision });
                let mut message = None;
                let mut ballot = None;
                if accepted {
                    match &choice.action {
                        Action::Speak { .. } => {
                            let mid = report.testimony.len();
                            message = Some(mid);
                            let testimony = build_testimony(
                                &input,
                                &choice,
                                actor,
                                &report.testimony,
                                lineage.unwrap(),
                                Origin::Selected { decision },
                            );
                            deliver(&mut report, &mut received, testimony);
                        }
                        Action::Vote(Some(candidate)) => {
                            report.tally[usize::from(*candidate)] += 1;
                            ballot = Some(*candidate);
                        }
                        _ => {}
                    }
                }
                report.objective_history.push(Event::Resolved {
                    decision,
                    accepted,
                    message,
                    ballot,
                });
            }
        }
    }
    report.retained_message_copies = received.iter().map(Vec::len).sum();
    report.verdict = verdict(report.tally);
    report.correct = report.verdict.map(|id| id == actor);
    report
        .objective_history
        .push(Event::Transition(Phase::Terminated));
    report.objective_history.push(Event::Outcome {
        tally: report.tally,
        verdict: report.verdict,
        correct: report.correct,
    });
    Ok(report)
}
