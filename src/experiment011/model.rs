use super::*;
use crate::{experiment006 as e6, experiment010 as e10};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Reference {
    pub episode: u8,
    pub slot: u8,
}
impl Reference {
    pub fn valid(self) -> bool {
        self.episode < 8 && self.slot < 24
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    Favorable,
    Unfavorable,
    Uncertain,
}
pub fn rating(c: Counts) -> Rating {
    match c.success.cmp(&c.failure) {
        std::cmp::Ordering::Greater => Rating::Favorable,
        std::cmp::Ordering::Less => Rating::Unfavorable,
        _ => Rating::Uncertain,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimedBasis {
    Firsthand,
    Summary,
    Relay,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payload {
    pub family: MethodId,
    pub rating: Rating,
    pub counts: Option<Counts>,
    pub basis: ClaimedBasis,
    pub claimed_origin: Option<u8>,
    pub parent: Option<Reference>,
    pub acquisition: Option<Acquisition>,
    pub verification_claimed: bool,
}
impl Payload {
    pub fn validate(&self) -> Result<(), String> {
        self.family.validate()?;
        if self.claimed_origin.is_some_and(|x| x >= 6)
            || self.parent.is_some_and(|r| !r.valid())
            || self.counts.is_some_and(|c| {
                [c.success, c.failure, c.nonmissing, c.missing]
                    .iter()
                    .any(|n| *n > 100)
            })
            || self
                .acquisition
                .is_some_and(|a| a.episode >= 8 || a.observer >= 6 || a.local >= 2)
        {
            return Err("invalid bounded method payload".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub version: String,
    pub reference: Reference,
    pub speaker: u8,
    pub payload: Payload,
    pub revision: Option<Reference>,
    pub retract: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharingKind {
    SilentMethod,
    SincereExperienceReport,
    SelectiveExperienceReport,
    UncertainReport,
    RelayExperience,
    MisleadingReportControl,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialKind {
    DirectOnly,
    NaiveSocial,
    ProvenanceAwareSocial,
    CredibilityWeightedSocial,
}
pub const LISTENERS: [SocialKind; 4] = [
    SocialKind::DirectOnly,
    SocialKind::NaiveSocial,
    SocialKind::ProvenanceAwareSocial,
    SocialKind::CredibilityWeightedSocial,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodIntent {
    Sincere,
    Contrary,
    Unsupported,
    FaithfulRelay,
    Withheld,
    NoMethod,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivateBasis {
    Personal(MethodId),
    Received(Reference),
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodAction {
    IncidentSpeech,
    Silence,
    Report {
        payload: Payload,
        revision: Option<Reference>,
        retract: bool,
        recipients: Vec<u8>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharingChoice {
    pub action: MethodAction,
    pub intent: MethodIntent,
    pub basis: PrivateBasis,
    pub eligible: bool,
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharingView {
    pub reference: Reference,
    pub incident: CommunicationView,
    pub personal: MethodMemory,
    pub social: SocialMemory,
    pub catalog: Vec<PublicMethod>,
    pub credits: [e6::Credit; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injection {
    pub episode: u8,
    pub slot: u8,
    pub choice: SharingChoice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub base: e10::Config,
    pub sharing: [SharingKind; 6],
    pub social: [SocialKind; 6],
    pub injections: Vec<Injection>,
    pub sharing_schedule: Vec<[SharingKind; 6]>,
    pub report_round: Option<u8>,
}
impl Deref for Config {
    type Target = e10::Config;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        self.base.validate()?;
        if self.version != "series-011-v1"
            || self.injections.len() > 96
            || (!self.sharing_schedule.is_empty()
                && self.sharing_schedule.len() != self.incidents.len())
            || self.report_round.is_some_and(|r| r > 1)
        {
            return Err("invalid011 version/intervention bound".into());
        }
        let mut keys = std::collections::BTreeSet::new();
        for x in &self.injections {
            if usize::from(x.episode) >= self.incidents.len()
                || x.slot >= 12
                || !keys.insert((x.episode, x.slot))
            {
                return Err("invalid injection slot".into());
            }
        }
        Ok(())
    }
    pub fn new(base: e10::Config, sharing: SharingKind, social: SocialKind) -> Self {
        Self {
            version: "series-011-v1".into(),
            base,
            sharing: [sharing; 6],
            social: [social; 6],
            injections: vec![],
            sharing_schedule: vec![],
            report_round: None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    Selected,
    Intervention(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialDecision {
    pub input: SharingView,
    pub choice: SharingChoice,
    pub origin: Origin,
    pub rejection: Option<String>,
    pub message: Option<Message>,
    pub actual_origins: Vec<Reference>,
    pub actual_depth: u8,
    pub deliveries: Vec<SocialIngestion>,
}
