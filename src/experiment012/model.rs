use super::*;
use crate::{experiment006 as e6, experiment011 as e11};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    First,
    Response,
    Final,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RequestType {
    MethodReliability,
    MethodAvailability,
    ExperienceBasis,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub version: String,
    pub reference: Reference,
    pub requester: u8,
    pub recipient: u8,
    pub family: MethodId,
    pub kind: RequestType,
    pub prior: Option<Reference>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Answer {
    Assessment {
        payload: Payload,
        revision: Option<Reference>,
        retract: bool,
    },
    InsufficientInformation,
    Refuse,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub version: String,
    pub reference: Reference,
    pub speaker: u8,
    pub recipient: u8,
    pub request: Reference,
    pub answer: Answer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestKind {
    NeverAsk,
    RandomAsk,
    UncertaintyDrivenAsk,
    ExperienceGapAsk,
    CredibilityGuidedAsk,
    CostAwareAsk,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseKind {
    NoReply,
    SincereReply,
    UncertainReply,
    SelectiveReply,
    MisleadingReplyControl,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestOption {
    pub recipient: u8,
    pub family: MethodId,
    pub kind: RequestType,
    pub prior: Option<Reference>,
    pub counts: Counts,
    pub accuracy: crate::experiment007::Probability,
    pub missingness: crate::experiment007::Probability,
    pub prior_report: bool,
    pub credit: i8,
    pub responsiveness: i32,
    pub score: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestChoice {
    pub options: Vec<RequestOption>,
    pub selected: Option<Request>,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseChoice {
    pub eligible: Vec<Reference>,
    pub selected: Option<Response>,
    pub basis: PrivateBasis,
    pub intent: MethodIntent,
    pub reason: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Responsiveness {
    pub requests: u16,
    pub replies: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InquiryMemory {
    pub version: String,
    pub observer: u8,
    pub sent: Vec<Request>,
    pub received: Vec<Request>,
    pub responses: Vec<Response>,
    pub answered: Vec<Reference>,
    pub responsiveness: [Responsiveness; 6],
}
impl InquiryMemory {
    pub fn neutral(observer: u8) -> Self {
        Self {
            version: "inquiry-memory-012-v1".into(),
            observer,
            sent: vec![],
            received: vec![],
            responses: vec![],
            answered: vec![],
            responsiveness: std::array::from_fn(|_| Responsiveness::default()),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "inquiry-memory-012-v1"
            || self.observer >= 6
            || [
                self.sent.len(),
                self.received.len(),
                self.responses.len(),
                self.answered.len(),
            ]
            .iter()
            .any(|n| *n > 12)
        {
            return Err("invalid inquiry memory bound/identity".into());
        }
        for list in [&self.sent, &self.received] {
            let mut seen = std::collections::BTreeSet::new();
            for q in list {
                q.validate()?;
                if !seen.insert(q.reference) {
                    return Err("duplicate inquiry memory identity".into());
                }
            }
        }
        if self.sent.iter().any(|q| q.requester != self.observer)
            || self.received.iter().any(|q| q.recipient != self.observer)
            || self.responses.iter().any(|a| a.recipient != self.observer)
        {
            return Err("foreign inquiry memory".into());
        }
        for a in &self.responses {
            a.validate()?;
        }
        Ok(())
    }
}
impl Request {
    pub fn validate(&self) -> Result<(), String> {
        self.family.validate()?;
        if self.version != "request-012-v1"
            || !self.reference.valid()
            || self.reference.slot >= 6
            || self.requester >= 6
            || self.recipient >= 6
            || self.requester == self.recipient
            || self
                .prior
                .is_some_and(|r| !r.valid() || r >= self.reference)
            || (self.kind == RequestType::ExperienceBasis && self.prior.is_none())
        {
            return Err("malformed typed request".into());
        }
        Ok(())
    }
}
impl Response {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "response-012-v1"
            || !self.reference.valid()
            || !(6..18).contains(&self.reference.slot)
            || !self.request.valid()
            || self.request >= self.reference
            || self.speaker >= 6
            || self.recipient >= 6
            || self.speaker == self.recipient
        {
            return Err("malformed typed response".into());
        }
        if let Answer::Assessment {
            payload, revision, ..
        } = &self.answer
        {
            payload.validate()?;
            if revision.is_some_and(|r| !r.valid() || r >= self.reference) {
                return Err("malformed response revision".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InquiryView {
    pub local: SharingView,
    pub memory: InquiryMemory,
    pub window: Window,
    pub capacity: u8,
    pub balance: u8,
    pub permitted_request_type: Option<RequestType>,
}
impl InquiryView {
    pub fn id(&self) -> u8 {
        self.local.incident.incident.id
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InquiryEffect {
    pub before: InquiryMemory,
    pub after: InquiryMemory,
    pub evicted: Vec<Reference>,
    pub kind: String,
    pub duplicate: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestDecision {
    pub input: InquiryView,
    pub choice: RequestChoice,
    pub rejection: Option<String>,
    pub delivered: Option<InquiryEffect>,
    pub sent: Option<InquiryEffect>,
    pub recipient_counts: Option<Counts>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseDecision {
    pub input: InquiryView,
    pub choice: ResponseChoice,
    pub proposal: Option<ResponseChoice>,
    pub origin: Origin,
    pub rejection: Option<String>,
    pub delivered: Vec<InquiryEffect>,
    pub social: Option<SocialDecision>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityRecord {
    pub reference: Reference,
    pub agent: u8,
    pub window: Window,
    pub before: u8,
    pub after: u8,
    pub submitted: bool,
    pub rejected: bool,
    pub exhausted: bool,
    pub foregone: Option<CommunicationChoice>,
    pub foregone_input: Option<CommunicationView>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnswerInjection {
    pub episode: u8,
    pub speaker: u8,
    pub window: Window,
    pub choice: ResponseChoice,
    pub wider: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub base: e11::Config,
    pub request: [RequestKind; 6],
    pub response: [ResponseKind; 6],
    pub request_type: Option<RequestType>,
    pub start_episode: u8,
    pub late_responses: bool,
    pub answer_injections: Vec<AnswerInjection>,
}
impl Deref for Config {
    type Target = e11::Config;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl Config {
    pub fn new(base: e11::Config) -> Self {
        Self {
            version: "series-012-v1".into(),
            base,
            request: [RequestKind::NeverAsk; 6],
            response: [ResponseKind::NoReply; 6],
            request_type: None,
            start_episode: 0,
            late_responses: false,
            answer_injections: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()?;
        if self.version != "series-012-v1"
            || self.start_episode >= 8
            || self.answer_injections.len() > 48
        {
            return Err("invalid012 configuration".into());
        }
        let mut keys = std::collections::BTreeSet::new();
        for i in &self.answer_injections {
            if usize::from(i.episode) >= self.incidents.len()
                || i.speaker >= 6
                || i.window == Window::First
                || !crate::experiment004::unique_ids(&i.wider)
                || !keys.insert((i.episode, i.speaker, format!("{:?}", i.window)))
            {
                return Err("invalid answer intervention".into());
            }
        }
        Ok(())
    }
}
pub fn credits(v: &InquiryView) -> [e6::Credit; 6] {
    v.local.credits
}
