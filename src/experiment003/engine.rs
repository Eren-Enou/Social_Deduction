use super::*;
use crate::{N, Phase, experiment002 as previous, verdict};
use serde::{Deserialize, Serialize};

pub const RULES: &str = "incident-003-v1";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injection {
    pub speaker: u8,
    pub speech: Speech,
    pub intent: Intent,
    pub recipients: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JudgmentKind {
    Current,
    Original,
    Abstain,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub seed: u64,
    pub layout: [u8; 6],
    pub speaking_order: [u8; 6],
    pub communication: [PolicyKind; 6],
    pub listeners: [ListenerKind; 6],
    pub judgments: [JudgmentKind; 6],
    pub incentives: [Incentives; 6],
    pub participation_briefing: bool,
    pub observation_override: Option<[Option<u8>; 6]>,
    pub injections: Vec<Injection>,
}
impl Config {
    pub fn new(seed: u64) -> Self {
        Self {
            version: "config-003-v1".into(),
            seed,
            layout: [0, 1, 2, 3, 4, 5],
            speaking_order: [0, 1, 2, 3, 4, 5],
            communication: [PolicyKind::Truthful; 6],
            listeners: [ListenerKind::Evidence; 6],
            judgments: [JudgmentKind::Current; 6],
            incentives: [Incentives::ACCURACY; 6],
            participation_briefing: true,
            observation_override: None,
            injections: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "config-003-v1" {
            return Err("unsupported config version".into());
        }
        for mut permutation in [self.layout, self.speaking_order] {
            permutation.sort_unstable();
            if permutation != [0, 1, 2, 3, 4, 5] {
                return Err("layout and speaking order must be permutations".into());
            }
        }
        if self
            .observation_override
            .is_some_and(|o| o.iter().any(|id| id.is_some_and(|id| id >= N)))
        {
            return Err("unknown observation candidate".into());
        }
        if self.injections.len() > 12 {
            return Err("controller injection budget exceeds 12".into());
        }
        for injection in &self.injections {
            let mut recipients = injection.recipients.clone();
            recipients.sort_unstable();
            recipients.dedup();
            if injection.speaker >= N
                || matches!(injection.speech,Speech::Assert(id) if id>=N)
                || recipients.len() != injection.recipients.len()
                || recipients.iter().any(|&id| id >= N)
            {
                return Err("illegal injection source, candidate, or recipients".into());
            }
        }
        Ok(())
    }
    pub fn actor(&self) -> u8 {
        self.layout[usize::from(previous::actor_for_seed(self.seed))]
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionView {
    pub id: u8,
    pub roster: Vec<u8>,
    pub phase: Phase,
    pub opportunities_left: u8,
    pub original: IndividualState,
    pub current: Belief,
    pub messages: Vec<Message>,
}
impl DecisionView {
    pub fn communication_view(&self) -> previous::AgentView {
        let mut own = self.original.clone();
        own.belief = self.current.candidate;
        previous::AgentView {
            id: self.id,
            roster: self.roster.clone(),
            phase: self.phase,
            opportunities_left: self.opportunities_left,
            own,
            messages: self.messages.clone(),
        }
    }
}
pub trait CommunicationPolicy {
    fn choose(&mut self, view: &DecisionView) -> Choice;
}
impl CommunicationPolicy for previous::BaselinePolicy {
    fn choose(&mut self, view: &DecisionView) -> Choice {
        previous::Policy::choose(self, &view.communication_view())
    }
}
pub trait JudgmentPolicy {
    fn choose(&mut self, view: &DecisionView) -> Choice;
}
impl JudgmentPolicy for JudgmentKind {
    fn choose(&mut self, view: &DecisionView) -> Choice {
        let (candidate, mechanism) = match self {
            Self::Current => (
                view.current.candidate,
                "vote unique supported current candidate; otherwise abstain",
            ),
            Self::Original => (
                view.original.belief,
                "vote original private candidate; otherwise abstain",
            ),
            Self::Abstain => (None, "explicit abstention policy"),
        };
        Choice {
            action: Action::Vote(candidate),
            intent: Intent::NoCommunication,
            scores: vec![],
            mechanism: mechanism.into(),
        }
    }
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
                .communication
                .iter()
                .map(|&p| {
                    Box::new(previous::BaselinePolicy::new(p)) as Box<dyn CommunicationPolicy>
                })
                .collect(),
            judges: config
                .judgments
                .iter()
                .map(|&p| Box::new(p) as Box<dyn JudgmentPolicy>)
                .collect(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub input: ListenerView,
    pub assessment: Assessment,
    pub candidate_changed: bool,
    pub before_correct: Option<bool>,
    pub after_correct: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub update: usize,
    pub input: DecisionView,
    pub choice: Choice,
    pub rejection: Option<String>,
    pub classification: Classification,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Selected { decision: usize },
    Intervention { injection: usize },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Testimony {
    pub message: Message,
    pub origin: Origin,
    pub recipients: Vec<u8>,
    pub speaker_belief: Option<u8>,
    pub intent: Intent,
    pub classification: Classification,
    pub assertion_matches_truth: Option<bool>,
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
        mechanism: String,
    },
    Delivered {
        message: usize,
        recipient: u8,
    },
    Updated {
        update: usize,
    },
    Resolved {
        decision: usize,
        accepted: bool,
        message: Option<usize>,
        ballot: Option<u8>,
    },
    Injected {
        injection: usize,
        message: usize,
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
    pub listener_version: String,
    pub judgment_version: String,
    pub communication_version: String,
    pub implementation: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub updates: Vec<Update>,
    pub decisions: Vec<Decision>,
    pub testimony: Vec<Testimony>,
    pub tally: [u8; 6],
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
}
fn deliver(report: &mut Report, received: &mut [Vec<Message>], testimony: Testimony) {
    for &recipient in &testimony.recipients {
        received[usize::from(recipient)].push(testimony.message.clone());
        report.objective_history.push(Event::Delivered {
            message: testimony.message.id,
            recipient,
        });
    }
    report.testimony.push(testimony);
}
fn make_testimony(
    view: &DecisionView,
    choice: &Choice,
    actor: u8,
    message_id: usize,
    origin: Origin,
    recipients: Vec<u8>,
) -> Testimony {
    let Action::Speak {
        speech,
        response_to,
    } = &choice.action
    else {
        unreachable!("only accepted speech creates testimony")
    };
    Testimony {
        message: Message {
            id: message_id,
            speaker: view.id,
            speech: speech.clone(),
            response_to: *response_to,
        },
        origin,
        recipients,
        speaker_belief: view.current.candidate,
        intent: choice.intent,
        classification: previous::classify(&view.communication_view(), choice, actor, true),
        assertion_matches_truth: match speech {
            Speech::Assert(id) => Some(*id == actor),
            Speech::Uncertain => None,
        },
    }
}
pub fn run(config: Config) -> Result<Report, String> {
    let mut policies = Policies::from_config(&config);
    let mut report = run_with(config, &mut policies)?;
    report.implementation = "builtin-003-v1".into();
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
        return Err("six listener, communication, and judgment policies required".into());
    }
    let actor = config.actor();
    let logical_actor = previous::actor_for_seed(config.seed);
    let mut original = vec![];
    for id in 0..N {
        let slot = config
            .layout
            .iter()
            .position(|&mapped| mapped == id)
            .unwrap();
        let observation = if let Some(overrides) = config.observation_override {
            overrides[usize::from(id)]
        } else {
            match slot {
                0 | 1 => Some(actor),
                2 => Some(config.layout[usize::from((logical_actor + 1) % N)]),
                _ => None,
            }
        };
        let briefing = if config.participation_briefing && id == actor {
            Some(Briefing::Participated)
        } else {
            None
        };
        original.push(IndividualState {
            observation,
            briefing,
            belief: if briefing.is_some() {
                Some(id)
            } else {
                observation
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
        rules_version: RULES.into(),
        listener_version: "source-count-003-v1".into(),
        judgment_version: "belief-vote-003-v1".into(),
        communication_version: previous::POLICIES.into(),
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
    };
    for id in 0..N {
        report.objective_history.push(Event::Assigned {
            agent: id,
            original: original[usize::from(id)].clone(),
            initial: current[usize::from(id)].clone(),
            mechanism: if config.observation_override.is_some() {
                "explicit observation override"
            } else {
                "versioned layout witness assignment"
            }
            .into(),
        });
    }
    for (index, injection) in config.injections.iter().enumerate() {
        let id = usize::from(injection.speaker);
        let view = DecisionView {
            id: injection.speaker,
            roster: (0..N).collect(),
            phase: Phase::Discussion,
            opportunities_left: 1,
            original: original[id].clone(),
            current: current[id].clone(),
            messages: received[id].clone(),
        };
        let choice = Choice {
            action: Action::Speak {
                speech: injection.speech.clone(),
                response_to: None,
            },
            intent: injection.intent,
            scores: vec![],
            mechanism: "controller supplied testimony; not a voluntary decision".into(),
        };
        let mid = report.testimony.len();
        report.objective_history.push(Event::Injected {
            injection: index,
            message: mid,
        });
        let testimony = make_testimony(
            &view,
            &choice,
            actor,
            mid,
            Origin::Intervention { injection: index },
            injection.recipients.clone(),
        );
        deliver(&mut report, &mut received, testimony);
    }
    for phase in [Phase::Discussion, Phase::Judgment] {
        report.objective_history.push(Event::Transition(phase));
        let rounds = if phase == Phase::Discussion { 2 } else { 1 };
        for round in 0..rounds {
            for agent in config.speaking_order {
                let id = usize::from(agent);
                let listener_input = ListenerView {
                    id: agent,
                    original: original[id].clone(),
                    prior: current[id].clone(),
                    messages: received[id].clone(),
                };
                let assessment = policies.listeners[id].assess(&listener_input);
                if Belief::from_scores(assessment.after.scores) != assessment.after {
                    return Err("inconsistent listener belief output".into());
                }
                let update = report.updates.len();
                report.updates.push(Update {
                    candidate_changed: listener_input.prior.candidate != assessment.after.candidate,
                    before_correct: listener_input.prior.candidate.map(|id| id == actor),
                    after_correct: assessment.after.candidate.map(|id| id == actor),
                    input: listener_input,
                    assessment: assessment.clone(),
                });
                report.objective_history.push(Event::Updated { update });
                current[id] = assessment.after;
                let input = DecisionView {
                    id: agent,
                    roster: (0..N).collect(),
                    phase,
                    opportunities_left: rounds - round,
                    original: original[id].clone(),
                    current: current[id].clone(),
                    messages: received[id].clone(),
                };
                let choice = if phase == Phase::Discussion {
                    policies.speakers[id].choose(&input)
                } else {
                    policies.judges[id].choose(&input)
                };
                let rejection =
                    previous::validate(&input.communication_view(), &choice.action).err();
                let accepted = rejection.is_none();
                let classification =
                    previous::classify(&input.communication_view(), &choice, actor, accepted);
                let decision = report.decisions.len();
                let mut message = None;
                let mut ballot = None;
                if accepted {
                    match &choice.action {
                        Action::Speak { .. } => {
                            let mid = report.testimony.len();
                            message = Some(mid);
                            let testimony = make_testimony(
                                &input,
                                &choice,
                                actor,
                                mid,
                                Origin::Selected { decision },
                                (0..N).collect(),
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
                report.decisions.push(Decision {
                    update,
                    input,
                    choice,
                    rejection,
                    classification,
                });
                report.objective_history.push(Event::Resolved {
                    decision,
                    accepted,
                    message,
                    ballot,
                });
            }
        }
    }
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
