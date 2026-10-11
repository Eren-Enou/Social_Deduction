use crate::{experiment006 as e6, experiment007 as e7, experiment008 as e8, experiment009 as e9};
use e7::VerificationPolicy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A public recurring family identifier, never a quality rank.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MethodId {
    pub version: String,
    pub key: String,
}
impl From<MethodId> for String {
    fn from(m: MethodId) -> Self {
        format!("{}/{}", m.version, m.key)
    }
}
impl TryFrom<String> for MethodId {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (version, key) = value
            .split_once('/')
            .ok_or("missing method identity version")?;
        let m = Self {
            version: version.into(),
            key: key.into(),
        };
        m.validate()?;
        Ok(m)
    }
}
impl MethodId {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            version: "method-family-010-v1".into(),
            key: key.into(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "method-family-010-v1"
            || self.key.is_empty()
            || self.key.len() > 32
            || !self
                .key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err("invalid public method-family identity".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub success: u16,
    pub failure: u16,
    pub nonmissing: u16,
    pub missing: u16,
}
impl Counts {
    pub fn graded(self) -> u32 {
        u32::from(self.success) + u32::from(self.failure)
    }
    pub fn observed(self) -> u32 {
        u32::from(self.nonmissing) + u32::from(self.missing)
    }
    pub fn accuracy(self, prior: u16) -> e7::Probability {
        e7::Probability {
            numerator: u64::from(self.success) + u64::from(prior),
            denominator: u64::from(self.graded()) + 2 * u64::from(prior),
        }
    }
    pub fn missingness(self, prior: u16) -> e7::Probability {
        e7::Probability {
            numerator: u64::from(self.missing) + u64::from(prior),
            denominator: u64::from(self.observed()) + 2 * u64::from(prior),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKey {
    Artifact(u8),
    Observation(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperienceStatus {
    Pending,
    Missing,
    Duplicate,
    Accepted {
        actor: u8,
        correct: bool,
        delivery: usize,
    },
    Quarantined,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectExperience {
    pub method: MethodId,
    pub finding: e8::Finding,
    pub origin: e9::DeliveryOrigin,
    pub dependency: DependencyKey,
    pub eligible: bool,
    pub status: ExperienceStatus,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndirectExperience {
    pub episode: u8,
    pub message: usize,
    pub method: Option<MethodId>,
    pub finding: e8::Finding,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodMemory {
    pub version: String,
    pub observer: u8,
    pub latest_episode: Option<u8>,
    pub records: Vec<DirectExperience>,
    pub indirect: Vec<IndirectExperience>,
    pub checkpoint: BTreeMap<MethodId, Counts>,
    pub summaries: BTreeMap<MethodId, Counts>,
    pub receipts: Vec<e7::Receipt>,
    pub tokens: Vec<e7::TokenEntry>,
}
impl MethodMemory {
    pub fn neutral(observer: u8) -> Self {
        Self {
            version: "method-memory-010-v1".into(),
            observer,
            latest_episode: None,
            records: vec![],
            indirect: vec![],
            checkpoint: BTreeMap::new(),
            summaries: BTreeMap::new(),
            receipts: vec![],
            tokens: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "method-memory-010-v1"
            || self.observer >= 6
            || self.records.len() > 8
            || self.indirect.len() > 4
            || self.summaries.len() > 4
            || self.checkpoint.len() > 4
            || self.receipts.len() > 8
            || self.tokens.len() > 32
            || self.latest_episode.is_some_and(|e| e >= 8)
        {
            return Err("invalid bounded method memory".into());
        }
        let mut keys = BTreeSet::new();
        for r in &self.records {
            r.method.validate()?;
            validate_finding(&r.finding, self.observer)?;
            if !keys.insert((r.finding.acquisition.episode, r.finding.acquisition.local))
                || self
                    .latest_episode
                    .is_none_or(|e| e < r.finding.acquisition.episode)
                || r.dependency != dependency(&r.finding)
                || !self.summaries.contains_key(&r.method)
            {
                return Err("inconsistent local method acquisition reference".into());
            }
        }
        for r in &self.indirect {
            if r.episode >= 8
                || r.finding.acquisition.episode != r.episode
                || r.finding.acquisition.observer >= 6
                || r.finding.candidate.is_some_and(|c| c >= 6)
            {
                return Err("invalid indirect method encounter".into());
            }
            if let Some(m) = &r.method {
                m.validate()?;
            }
        }
        for m in self.checkpoint.keys().chain(self.summaries.keys()) {
            m.validate()?;
        }
        let mut episodes = BTreeSet::new();
        for r in &self.receipts {
            if !episodes.insert(r.episode)
                || !self
                    .records
                    .iter()
                    .any(|x| x.finding.acquisition.episode == r.episode)
            {
                return Err("receipt has no retained method experience".into());
            }
            match r.state {
                e7::VerificationState::Accepted { actor, .. } if actor >= 6 => {
                    return Err("invalid accepted method outcome".into());
                }
                e7::VerificationState::Quarantined { first, conflicting }
                    if first >= 6 || conflicting >= 6 =>
                {
                    return Err("invalid method quarantine".into());
                }
                _ => {}
            }
        }
        let mut expected = self.clone();
        expected.recompute();
        if expected.summaries != self.summaries || expected.records != self.records {
            return Err("method summaries/status inconsistent with retained evidence".into());
        }
        Ok(())
    }
    fn accepted(&self, episode: u8) -> Option<(u8, usize)> {
        self.receipts.iter().find_map(|r| {
            if r.episode == episode {
                match r.state {
                    e7::VerificationState::Accepted { actor, delivery } => Some((actor, delivery)),
                    _ => None,
                }
            } else {
                None
            }
        })
    }
    fn recompute(&mut self) {
        self.summaries = self.checkpoint.clone();
        for r in &mut self.records {
            let receipt = self
                .receipts
                .iter()
                .find(|x| x.episode == r.finding.acquisition.episode);
            r.status = if !r.eligible {
                ExperienceStatus::Duplicate
            } else if r.finding.candidate.is_none() {
                ExperienceStatus::Missing
            } else {
                match receipt.map(|r| &r.state) {
                    Some(e7::VerificationState::Accepted { actor, delivery }) => {
                        ExperienceStatus::Accepted {
                            actor: *actor,
                            correct: r.finding.candidate == Some(*actor),
                            delivery: *delivery,
                        }
                    }
                    Some(e7::VerificationState::Quarantined { .. }) => {
                        ExperienceStatus::Quarantined
                    }
                    None => ExperienceStatus::Pending,
                }
            };
            fold(
                self.summaries.entry(r.method.clone()).or_default(),
                r,
                receipt.and_then(|r| match r.state {
                    e7::VerificationState::Accepted { actor, .. } => Some(actor),
                    _ => None,
                }),
            );
        }
    }
    /// Only eligibility and legitimately retained receipts/journal enter 007 acceptance.
    pub fn verification_projection(&self) -> e7::Memory {
        let mut memory = e7::Memory::neutral(self.observer);
        memory.receipts = self.receipts.clone();
        memory.tokens = self.tokens.clone();
        for r in &self.records {
            memory.local.records.push(e6::Experience {
                key: e6::Reference {
                    episode: r.finding.acquisition.episode,
                    local: r.finding.acquisition.local,
                },
                group: None,
                source: None,
                initial_candidate: None,
                final_candidate: None,
                received: vec![],
                relays: vec![],
                corrections: 0,
                retractions: 0,
                verified: None,
                announcement: None,
            });
        }
        memory
    }
}
fn dependency(f: &e8::Finding) -> DependencyKey {
    f.source
        .visible_parent
        .map(DependencyKey::Observation)
        .unwrap_or(DependencyKey::Artifact(f.source.id))
}
fn validate_finding(f: &e8::Finding, observer: u8) -> Result<(), String> {
    if f.acquisition.observer != observer
        || f.acquisition.episode >= 8
        || f.acquisition.local > 1
        || f.candidate.is_some_and(|c| c >= 6)
        || f.source.visible_parent.is_some_and(|c| c >= 6)
        || f.source.description.trim().is_empty()
        || f.source.description.len() > 128
    {
        return Err("invalid received method finding".into());
    }
    Ok(())
}
fn fold(c: &mut Counts, r: &DirectExperience, accepted: Option<u8>) {
    if !r.eligible {
        return;
    }
    if let Some(candidate) = r.finding.candidate {
        c.nonmissing = c.nonmissing.saturating_add(1);
        if let Some(actor) = accepted {
            if candidate == actor {
                c.success = c.success.saturating_add(1);
            } else {
                c.failure = c.failure.saturating_add(1);
            }
        }
    } else {
        c.missing = c.missing.saturating_add(1);
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodIngestView {
    pub memory: MethodMemory,
    pub method: MethodId,
    pub finding: e8::Finding,
    pub origin: e9::DeliveryOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodIngestion {
    pub input: MethodIngestView,
    pub after: MethodMemory,
    pub evicted: Vec<DirectExperience>,
    pub duplicate: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodFeedbackView {
    pub memory: MethodMemory,
    pub received: e7::Token,
    pub delivery: usize,
    pub completed_episode: u8,
    pub session: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodFeedback {
    pub input: MethodFeedbackView,
    pub transport_input: e7::VerificationView,
    pub transport_effect: e7::VerificationEffect,
    pub after: MethodMemory,
}
pub trait ReliabilityUpdater {
    fn ingest(&mut self, v: &MethodIngestView) -> Result<MethodIngestion, String>;
    fn verify(&mut self, v: &MethodFeedbackView) -> Result<MethodFeedback, String>;
}
pub struct CountUpdater;
impl ReliabilityUpdater for CountUpdater {
    fn ingest(&mut self, v: &MethodIngestView) -> Result<MethodIngestion, String> {
        v.memory.validate()?;
        v.method.validate()?;
        validate_finding(&v.finding, v.memory.observer)?;
        let episode = v.finding.acquisition.episode;
        if v.memory.latest_episode.is_some_and(|e| e > episode) {
            return Err("cannot ingest inaccessible historical finding".into());
        }
        let mut after = v.memory.clone();
        if let Some(r) = after
            .records
            .iter()
            .find(|r| r.finding.acquisition == v.finding.acquisition)
        {
            if r.finding != v.finding || r.method != v.method {
                return Err("acquisition reference changed its payload or method".into());
            }
            return Ok(MethodIngestion {
                input: v.clone(),
                after,
                evicted: vec![],
                duplicate: true,
            });
        }
        let key = dependency(&v.finding);
        let duplicate = after
            .records
            .iter()
            .any(|r| r.finding.acquisition.episode == episode && r.dependency == key);
        if !after.summaries.contains_key(&v.method) && after.summaries.len() == 4 {
            return Err("method family capacity exceeded".into());
        }
        after.latest_episode = Some(episode);
        let mut evicted = vec![];
        if after.records.len() == 8 {
            let r = after.records.remove(0);
            let accepted = after.accepted(r.finding.acquisition.episode).map(|x| x.0);
            fold(
                after.checkpoint.entry(r.method.clone()).or_default(),
                &r,
                accepted,
            );
            evicted.push(r);
        }
        after.records.push(DirectExperience {
            method: v.method.clone(),
            finding: v.finding.clone(),
            origin: v.origin,
            dependency: key,
            eligible: !duplicate,
            status: ExperienceStatus::Pending,
        });
        after.receipts.retain(|r| {
            after
                .records
                .iter()
                .any(|x| x.finding.acquisition.episode == r.episode)
        });
        after.recompute();
        after.validate()?;
        Ok(MethodIngestion {
            input: v.clone(),
            after,
            evicted,
            duplicate,
        })
    }
    fn verify(&mut self, v: &MethodFeedbackView) -> Result<MethodFeedback, String> {
        v.memory.validate()?;
        if v.completed_episode >= 8 {
            return Err("invalid method feedback episode bound".into());
        }
        let transport_input = e7::VerificationView {
            delivery: v.delivery,
            completed_episode: v.completed_episode,
            session: v.session,
            memory: v.memory.verification_projection(),
            received: v.received.clone(),
        };
        let transport_effect = e7::VerificationKind::ConflictAware.accept(&transport_input);
        transport_effect.after.validate()?;
        let mut after = v.memory.clone();
        after.receipts = transport_effect.after.receipts.clone();
        after.tokens = transport_effect.after.tokens.clone();
        after.recompute();
        after.validate()?;
        Ok(MethodFeedback {
            input: v.clone(),
            transport_input,
            transport_effect,
            after,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndirectIngestion {
    pub before: MethodMemory,
    pub received: IndirectExperience,
    pub after: MethodMemory,
    pub duplicate: bool,
    pub evicted: Vec<IndirectExperience>,
}
pub fn ingest_indirect(
    memory: &MethodMemory,
    received: IndirectExperience,
) -> Result<IndirectIngestion, String> {
    memory.validate()?;
    let duplicate = memory
        .indirect
        .iter()
        .any(|r| r.episode == received.episode && r.message == received.message);
    let mut after = memory.clone();
    let mut evicted = vec![];
    if !duplicate {
        if after.indirect.len() == 4 {
            evicted.push(after.indirect.remove(0));
        }
        after.indirect.push(received.clone());
    }
    after.validate()?;
    Ok(IndirectIngestion {
        before: memory.clone(),
        received,
        after,
        duplicate,
        evicted,
    })
}
