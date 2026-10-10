use super::*;
use crate::{N, Phase, experiment002 as e2, experiment003 as e3};
use serde::{Deserialize, Serialize};

pub const MAX_DEPTH: u8 = 3;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attribution {
    Firsthand,
    Unspecified,
    NamedSource(u8),
    Reference(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: usize,
    pub speaker: u8,
    pub speech: Speech,
    pub attribution: Attribution,
}
impl Message {
    pub fn without_provenance(&self) -> e2::Message {
        e2::Message {
            id: self.id,
            speaker: self.speaker,
            speech: self.speech.clone(),
            response_to: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Basis {
    PrivateObservation,
    Inference,
    Received(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    Baseline(Intent),
    RelayReceived,
    MisattributeSource,
    NamedUnverified,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Silence,
    Speak {
        speech: Speech,
        attribution: Attribution,
        basis: Basis,
        recipients: Vec<u8>,
    },
    Vote(Option<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub action: Action,
    pub strategy: Strategy,
    pub scores: Vec<e2::OptionScore>,
    pub mechanism: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Presentation {
    Disclose,
    Unspecified,
    Firsthand,
    Named(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeakerKind {
    Baseline {
        kind: PolicyKind,
        attribution: Option<Attribution>,
    },
    Relay {
        source: u8,
        presentation: Presentation,
    },
}
impl SpeakerKind {
    pub fn baseline(kind: PolicyKind) -> Self {
        Self::Baseline {
            kind,
            attribution: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentView {
    pub id: u8,
    pub roster: Vec<u8>,
    pub phase: Phase,
    pub opportunities_left: u8,
    pub original: IndividualState,
    pub current: Belief,
    pub messages: Vec<Message>,
    pub permitted_recipients: Vec<u8>,
}
impl AgentView {
    pub fn baseline_view(&self) -> e2::AgentView {
        let mut own = self.original.clone();
        own.belief = self.current.candidate;
        e2::AgentView {
            id: self.id,
            roster: self.roster.clone(),
            phase: self.phase,
            opportunities_left: self.opportunities_left,
            own,
            messages: self
                .messages
                .iter()
                .map(Message::without_provenance)
                .collect(),
        }
    }
}
pub trait CommunicationPolicy {
    fn choose(&mut self, view: &AgentView) -> Choice;
}
pub trait JudgmentPolicy {
    fn choose(&mut self, view: &AgentView) -> Choice;
}
impl JudgmentPolicy for JudgmentKind {
    fn choose(&mut self, view: &AgentView) -> Choice {
        let candidate = match self {
            Self::Current => view.current.candidate,
            Self::Original => view.original.belief,
            Self::Abstain => None,
        };
        Choice {
            action: Action::Vote(candidate),
            strategy: Strategy::Baseline(Intent::NoCommunication),
            scores: vec![],
            mechanism: format!("belief-vote-003-v1: {self:?}"),
        }
    }
}
pub struct SpeakerPolicy {
    kind: SpeakerKind,
    baseline: Option<e2::BaselinePolicy>,
    spoken: bool,
}
impl SpeakerPolicy {
    pub fn new(kind: SpeakerKind) -> Self {
        Self {
            kind,
            baseline: match kind {
                SpeakerKind::Baseline { kind, .. } => Some(e2::BaselinePolicy::new(kind)),
                _ => None,
            },
            spoken: false,
        }
    }
}
impl CommunicationPolicy for SpeakerPolicy {
    fn choose(&mut self, view: &AgentView) -> Choice {
        let silent = |reason: &str| Choice {
            action: Action::Silence,
            strategy: Strategy::Baseline(Intent::NoCommunication),
            scores: vec![],
            mechanism: reason.into(),
        };
        if view.phase != Phase::Discussion || view.opportunities_left == 0 {
            return silent("no discussion opportunity");
        }
        match self.kind {
            SpeakerKind::Baseline { attribution, .. } => {
                let chosen =
                    e2::Policy::choose(self.baseline.as_mut().unwrap(), &view.baseline_view());
                let action = match chosen.action {
                    e2::Action::Silence => Action::Silence,
                    e2::Action::Vote(vote) => Action::Vote(vote),
                    e2::Action::Speak { speech, .. } => {
                        let private = matches!(speech,Speech::Assert(candidate) if chosen.intent==Intent::ReportBelief && (view.original.observation==Some(candidate) || (view.original.briefing.is_some() && candidate==view.id)));
                        let basis = if private {
                            Basis::PrivateObservation
                        } else {
                            Basis::Inference
                        };
                        let reported = attribution.unwrap_or(if private {
                            Attribution::Firsthand
                        } else {
                            Attribution::Unspecified
                        });
                        Action::Speak {
                            speech,
                            attribution: reported,
                            basis,
                            recipients: view.permitted_recipients.clone(),
                        }
                    }
                };
                Choice {
                    action,
                    strategy: Strategy::Baseline(chosen.intent),
                    scores: chosen.scores,
                    mechanism: chosen.mechanism,
                }
            }
            SpeakerKind::Relay {
                source,
                presentation,
            } => {
                if self.spoken {
                    return silent("one-message relay policy budget already used");
                }
                let Some(parent) = view
                    .messages
                    .iter()
                    .find(|m| m.speaker == source && matches!(m.speech, Speech::Assert(_)))
                else {
                    return silent("no legitimately received assertion from configured speaker");
                };
                let Speech::Assert(candidate) = parent.speech else {
                    unreachable!()
                };
                if view.current.candidate != Some(candidate) {
                    return silent("received assertion does not agree with current candidate");
                }
                let (attribution, strategy) = match presentation {
                    Presentation::Disclose => {
                        (Attribution::Reference(parent.id), Strategy::RelayReceived)
                    }
                    Presentation::Unspecified => {
                        (Attribution::Unspecified, Strategy::RelayReceived)
                    }
                    Presentation::Firsthand => {
                        (Attribution::Firsthand, Strategy::MisattributeSource)
                    }
                    Presentation::Named(id) => {
                        (Attribution::NamedSource(id), Strategy::NamedUnverified)
                    }
                };
                self.spoken = true;
                Choice {
                    action: Action::Speak {
                        speech: parent.speech.clone(),
                        attribution,
                        basis: Basis::Received(parent.id),
                        recipients: view.permitted_recipients.clone(),
                    },
                    strategy,
                    scores: vec![],
                    mechanism: "relay-004-v1: forward first received agreeing assertion once"
                        .into(),
                }
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injection {
    pub speaker: u8,
    pub choice: Choice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub seed: u64,
    pub observations: [Option<u8>; 6],
    pub participation_briefing: bool,
    pub order: [u8; 6],
    pub speakers: [SpeakerKind; 6],
    pub listeners: [ListenerKind; 6],
    pub judgments: [JudgmentKind; 6],
    pub incentives: [Incentives; 6],
    pub routes: [Vec<u8>; 6],
    pub injections: Vec<Injection>,
}
pub fn unique_ids(ids: &[u8]) -> bool {
    let mut copy = ids.to_vec();
    copy.sort_unstable();
    copy.dedup();
    copy.len() == ids.len() && copy.iter().all(|&id| id < N)
}
impl Config {
    pub fn new(seed: u64) -> Self {
        Self {
            version: "config-004-v1".into(),
            seed,
            observations: [None; 6],
            participation_briefing: false,
            order: [0, 1, 2, 3, 4, 5],
            speakers: [SpeakerKind::baseline(PolicyKind::Silent); 6],
            listeners: [ListenerKind::Provenance; 6],
            judgments: [JudgmentKind::Current; 6],
            incentives: [Incentives::ACCURACY; 6],
            routes: std::array::from_fn(|_| (0..N).collect()),
            injections: vec![],
        }
    }
    pub fn actor(&self) -> u8 {
        e2::actor_for_seed(self.seed)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "config-004-v1" {
            return Err("unsupported configuration version".into());
        }
        if !unique_ids(&self.order) || self.order.len() != usize::from(N) {
            return Err("speaking order must be a permutation".into());
        }
        if self
            .observations
            .iter()
            .any(|id| id.is_some_and(|id| id >= N))
            || self.routes.iter().any(|r| !unique_ids(r))
        {
            return Err("invalid observations or permitted recipients".into());
        }
        if self.injections.len() > 12
            || self
                .injections
                .iter()
                .any(|i| i.speaker >= N || !matches!(i.choice.action, Action::Speak { .. }))
        {
            return Err("invalid controller injection budget or action".into());
        }
        for speaker in self.speakers {
            match speaker {
                SpeakerKind::Relay { source, .. } if source >= N => {
                    return Err("unknown relay source".into());
                }
                SpeakerKind::Relay {
                    presentation: Presentation::Named(id),
                    ..
                }
                | SpeakerKind::Baseline {
                    attribution: Some(Attribution::NamedSource(id)),
                    ..
                } if id >= N => return Err("unknown attributed source".into()),
                _ => {}
            }
        }
        Ok(())
    }
}
pub fn as_listener_view(view: &AgentView) -> ListenerView {
    ListenerView {
        id: view.id,
        original: view.original.clone(),
        prior: view.current.clone(),
        messages: view.messages.clone(),
    }
}
pub fn source_count_view(view: &ListenerView) -> e3::ListenerView {
    e3::ListenerView {
        id: view.id,
        original: view.original.clone(),
        prior: view.prior.clone(),
        messages: view
            .messages
            .iter()
            .map(Message::without_provenance)
            .collect(),
    }
}
