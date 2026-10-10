use super::{Briefing, IndividualState, Message, Speech};
use crate::N;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Belief {
    pub scores: [u16; 6],
    pub candidate: Option<u8>,
    pub margin: u16,
}
impl Belief {
    pub fn from_scores(scores: [u16; 6]) -> Self {
        let mut sorted = scores;
        sorted.sort_unstable();
        let maximum = sorted[5];
        let margin = maximum - sorted[4];
        let candidate = if maximum > 0 && margin > 0 {
            scores.iter().position(|&s| s == maximum).map(|i| i as u8)
        } else {
            None
        };
        Self {
            scores,
            candidate,
            margin,
        }
    }
}
pub fn initial_belief(id: u8, original: &IndividualState) -> Belief {
    let mut scores = [0; 6];
    if original.briefing == Some(Briefing::Participated) {
        scores[usize::from(id)] = 6;
    } else if let Some(candidate) = original.observation {
        scores[usize::from(candidate)] = 2;
    }
    Belief::from_scores(scores)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListenerView {
    pub id: u8,
    pub original: IndividualState,
    pub prior: Belief,
    pub messages: Vec<Message>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Handling {
    SupportsPrior,
    ConflictsPrior,
    AddsEvidence,
    SelfIgnored,
    UncertaintyIgnored,
    RepeatIgnored,
    RetainPolicyIgnored,
    InvalidIgnored,
}
impl Handling {
    pub fn ignored(self) -> bool {
        matches!(
            self,
            Self::SelfIgnored
                | Self::UncertaintyIgnored
                | Self::RepeatIgnored
                | Self::RetainPolicyIgnored
                | Self::InvalidIgnored
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageAssessment {
    pub message_id: usize,
    pub handling: Handling,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub after: Belief,
    pub messages: Vec<MessageAssessment>,
    pub mechanism: String,
}
pub trait Listener {
    fn assess(&mut self, view: &ListenerView) -> Assessment;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListenerKind {
    Evidence,
    Retain,
}
impl Listener for ListenerKind {
    fn assess(&mut self, view: &ListenerView) -> Assessment {
        if *self == Self::Retain {
            return Assessment {
                after: view.prior.clone(),
                messages: view
                    .messages
                    .iter()
                    .map(|m| MessageAssessment {
                        message_id: m.id,
                        handling: Handling::RetainPolicyIgnored,
                    })
                    .collect(),
                mechanism: "retain prior; ignore all testimony".into(),
            };
        }
        let mut scores = initial_belief(view.id, &view.original).scores;
        let mut sources = [false; 6];
        let mut messages = vec![];
        for message in &view.messages {
            let handling = if message.speaker >= N {
                Handling::InvalidIgnored
            } else if message.speaker == view.id {
                Handling::SelfIgnored
            } else {
                match message.speech {
                    Speech::Uncertain => Handling::UncertaintyIgnored,
                    Speech::Assert(candidate) if candidate >= N => Handling::InvalidIgnored,
                    Speech::Assert(_) if sources[usize::from(message.speaker)] => {
                        Handling::RepeatIgnored
                    }
                    Speech::Assert(candidate) => {
                        sources[usize::from(message.speaker)] = true;
                        scores[usize::from(candidate)] += 1;
                        match view.prior.candidate {
                            Some(prior) if prior == candidate => Handling::SupportsPrior,
                            Some(_) => Handling::ConflictsPrior,
                            None => Handling::AddsEvidence,
                        }
                    }
                }
            };
            messages.push(MessageAssessment {
                message_id: message.id,
                handling,
            });
        }
        Assessment {
            after: Belief::from_scores(scores),
            messages,
            mechanism:
                "recompute private anchor plus first assertion per other speaker; ties uncertain"
                    .into(),
        }
    }
}
