use crate::experiment005 as e5;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MEMORY_CAPACITY: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Reference {
    pub episode: u8,
    pub local: usize,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credit {
    pub score: i8,
    pub accurate: u16,
    pub inaccurate: u16,
    pub original_accurate: u16,
    pub original_inaccurate: u16,
    pub corrections: u16,
    pub retractions: u16,
    pub relay_accurate: u16,
    pub relay_inaccurate: u16,
}
impl Credit {
    pub fn verified_count(&self) -> u16 {
        self.accurate.saturating_add(self.inaccurate)
    }
    pub fn weight(&self) -> u16 {
        i16::from(self.score).saturating_add(1).max(0) as u16
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observed {
    pub reference: Reference,
    pub speaker: u8,
    pub body: e5::Body,
    pub attribution: e5::Attribution,
    pub standing: e5::Standing,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayReport {
    pub speaker: u8,
    pub candidate: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verified {
    pub feedback: usize,
    pub actor: u8,
    pub issuer: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Announcement {
    ParticipantClaim(u8),
    CollectiveVerdict(Option<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Experience {
    pub key: Reference,
    pub group: Option<e5::Group>,
    pub source: Option<u8>,
    pub initial_candidate: Option<u8>,
    pub final_candidate: Option<u8>,
    pub received: Vec<Observed>,
    pub relays: Vec<RelayReport>,
    pub corrections: u16,
    pub retractions: u16,
    pub verified: Option<Verified>,
    pub announcement: Option<Announcement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMemory {
    pub observer: u8,
    pub credits: [Credit; 6],
    pub records: Vec<Experience>,
}
impl LocalMemory {
    pub fn neutral(observer: u8) -> Self {
        Self {
            observer,
            credits: [Credit::default(); 6],
            records: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.observer >= 6
            || self.records.len() > MEMORY_CAPACITY
            || self.credits.iter().any(|c| !(-2..=2).contains(&c.score))
            || self.records.iter().any(|r| {
                r.received.len() > 24
                    || r.source.is_some_and(|s| s >= 6)
                    || r.relays.iter().any(|p| p.speaker >= 6)
                    || r.received
                        .iter()
                        .any(|p| p.reference.episode != r.key.episode)
            })
        {
            return Err("invalid bounded local memory".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngestView {
    pub episode: u8,
    pub observer: u8,
    pub received: Vec<e5::Message>,
    pub interpretation: e5::Assessment,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ingestion {
    pub input: IngestView,
    pub before: LocalMemory,
    pub after: LocalMemory,
    pub evicted: Vec<Reference>,
    pub duplicate_reports_prevented: usize,
    pub record_inspections: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Bucket {
    Known(e5::Group),
    UnknownSpeaker(u8),
}
pub fn ingest(before: &LocalMemory, input: IngestView) -> Result<Ingestion, String> {
    if input.observer != before.observer || input.received.len() > 24 {
        return Err("invalid local ingestion".into());
    }
    let mut after = before.clone();
    let mut buckets: BTreeMap<Bucket, Vec<&e5::Message>> = BTreeMap::new();
    for m in &input.received {
        if m.speaker == input.observer || m.speaker >= 6 || matches!(m.body, e5::Body::Uncertain) {
            continue;
        }
        let a = input
            .interpretation
            .interpretations
            .iter()
            .find(|a| a.message == m.id)
            .ok_or("missing local interpretation")?;
        let group = a.group.or_else(|| {
            input
                .interpretation
                .interpretations
                .iter()
                .find(|p| {
                    p.group.is_some()
                        && input
                            .received
                            .iter()
                            .any(|m2| m2.id == p.message && m2.speaker == m.speaker)
                })
                .and_then(|a| a.group)
        });
        buckets
            .entry(
                group
                    .map(Bucket::Known)
                    .unwrap_or(Bucket::UnknownSpeaker(m.speaker)),
            )
            .or_default()
            .push(m);
    }
    let mut evicted = vec![];
    let mut duplicate_reports_prevented = 0;
    let mut record_inspections = 0;
    for (bucket, messages) in buckets {
        let group = if let Bucket::Known(g) = bucket {
            Some(g)
        } else {
            None
        };
        let source = if let Some(e5::Group::Speaker(s)) = group {
            if s != input.observer
                && messages.iter().any(|m| {
                    m.speaker == s
                        && matches!(m.body, e5::Body::Assert(_))
                        && input
                            .interpretation
                            .interpretations
                            .iter()
                            .any(|a| a.message == m.id && a.root == Some(m.id))
                })
            {
                Some(s)
            } else {
                None
            }
        } else {
            None
        };
        let original = source.and_then(|s| {
            messages
                .iter()
                .find(|m| m.speaker == s && matches!(m.body, e5::Body::Assert(_)))
                .copied()
        });
        let branch_candidate = |speaker: u8, root: usize| {
            let leaves: Vec<_> = input
                .interpretation
                .interpretations
                .iter()
                .filter(|a| {
                    a.root == Some(root)
                        && matches!(a.standing, e5::Standing::Active | e5::Standing::Retracted)
                })
                .filter_map(|a| {
                    messages
                        .iter()
                        .find(|m| m.id == a.message && m.speaker == speaker)
                })
                .collect();
            leaves.first().and_then(|m| m.body.candidate())
        };
        let final_candidate = original.and_then(|m| branch_candidate(m.speaker, m.id));
        let corrections = messages
            .iter()
            .filter(|m| {
                Some(m.speaker) == source
                    && matches!(
                        m.body,
                        e5::Body::Revise {
                            replacement: Some(_),
                            ..
                        }
                    )
            })
            .count() as u16;
        let retractions = messages
            .iter()
            .filter(|m| {
                Some(m.speaker) == source
                    && matches!(
                        m.body,
                        e5::Body::Revise {
                            replacement: None,
                            ..
                        }
                    )
            })
            .count() as u16;
        let mut relays = vec![];
        for m in &messages {
            if Some(m.speaker) != source
                && matches!(m.body, e5::Body::Assert(_))
                && matches!(
                    m.attribution,
                    e5::Attribution::Reference(_) | e5::Attribution::NamedSource(_)
                )
                && !relays.iter().any(|r: &RelayReport| r.speaker == m.speaker)
            {
                relays.push(RelayReport {
                    speaker: m.speaker,
                    candidate: branch_candidate(m.speaker, m.id),
                });
            }
        }
        duplicate_reports_prevented += messages
            .iter()
            .filter(|m| matches!(m.body, e5::Body::Assert(_)))
            .count()
            .saturating_sub(1);
        let record = Experience {
            key: Reference {
                episode: input.episode,
                local: messages[0].id,
            },
            group,
            source,
            initial_candidate: original.and_then(|m| m.body.candidate()),
            final_candidate,
            received: messages
                .iter()
                .map(|m| Observed {
                    reference: Reference {
                        episode: input.episode,
                        local: m.id,
                    },
                    speaker: m.speaker,
                    body: m.body.clone(),
                    attribution: m.attribution,
                    standing: input
                        .interpretation
                        .interpretations
                        .iter()
                        .find(|a| a.message == m.id)
                        .unwrap()
                        .standing,
                })
                .collect(),
            relays,
            corrections,
            retractions,
            verified: None,
            announcement: None,
        };
        record_inspections += after.records.len();
        if after.records.iter().any(|r| {
            r.key.episode == input.episode && r.group == record.group && r.key == record.key
        }) {
            return Err("episode ingested twice".into());
        }
        if let Some(s) = source {
            let c = &mut after.credits[usize::from(s)];
            c.corrections = c.corrections.saturating_add(corrections);
            c.retractions = c.retractions.saturating_add(retractions);
        }
        after.records.push(record);
        if after.records.len() > MEMORY_CAPACITY {
            evicted.push(after.records.remove(0).key);
        }
    }
    after.validate()?;
    Ok(Ingestion {
        input,
        before: before.clone(),
        after,
        evicted,
        duplicate_reports_prevented,
        record_inspections,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Feedback {
    VerifiedOutcome {
        episode: u8,
        actor: u8,
        issuer: String,
    },
    Unverified {
        episode: u8,
        announcement: Announcement,
    },
}
impl Feedback {
    pub fn episode(&self) -> u8 {
        match self {
            Self::VerifiedOutcome { episode, .. } | Self::Unverified { episode, .. } => *episode,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackView {
    pub delivery: usize,
    pub memory: LocalMemory,
    pub received: Feedback,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackEffect {
    pub after: LocalMemory,
    pub record_inspections: usize,
    pub graded_sources: usize,
    pub unknown_sources: usize,
    pub ignored_unverified: usize,
    pub duplicate_verifications: usize,
    pub integrity_conflicts: usize,
    pub no_retained_experience: bool,
}
pub trait CredibilityUpdater {
    fn update(&mut self, view: &FeedbackView) -> FeedbackEffect;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateKind {
    Evidence,
    Retain,
}
impl CredibilityUpdater for UpdateKind {
    fn update(&mut self, v: &FeedbackView) -> FeedbackEffect {
        let mut effect = FeedbackEffect {
            after: v.memory.clone(),
            record_inspections: v.memory.records.len(),
            graded_sources: 0,
            unknown_sources: 0,
            ignored_unverified: 0,
            duplicate_verifications: 0,
            integrity_conflicts: 0,
            no_retained_experience: !v
                .memory
                .records
                .iter()
                .any(|r| r.key.episode == v.received.episode()),
        };
        for r in effect
            .after
            .records
            .iter_mut()
            .filter(|r| r.key.episode == v.received.episode())
        {
            match &v.received {
                Feedback::Unverified { announcement, .. } => {
                    r.announcement = Some(announcement.clone());
                    effect.ignored_unverified += 1;
                }
                Feedback::VerifiedOutcome { actor, issuer, .. } => {
                    if *actor >= 6 || issuer != "lab-verification-006-v1" {
                        effect.integrity_conflicts += 1;
                        continue;
                    }
                    if let Some(previous) = &r.verified {
                        if previous.actor == *actor {
                            effect.duplicate_verifications += 1;
                        } else {
                            effect.integrity_conflicts += 1;
                        }
                        continue;
                    }
                    r.verified = Some(Verified {
                        feedback: v.delivery,
                        actor: *actor,
                        issuer: issuer.clone(),
                    });
                    if *self == Self::Retain {
                        continue;
                    }
                    if let Some(source) = r.source {
                        let c = &mut effect.after.credits[usize::from(source)];
                        if let Some(initial) = r.initial_candidate {
                            if initial == *actor {
                                c.original_accurate = c.original_accurate.saturating_add(1);
                            } else {
                                c.original_inaccurate = c.original_inaccurate.saturating_add(1);
                            }
                        }
                        if let Some(final_c) = r.final_candidate {
                            if final_c == *actor {
                                c.score = (c.score + 1).min(2);
                                c.accurate = c.accurate.saturating_add(1);
                            } else {
                                c.score = (c.score - 1).max(-2);
                                c.inaccurate = c.inaccurate.saturating_add(1);
                            }
                            effect.graded_sources += 1;
                        }
                    } else {
                        effect.unknown_sources += 1;
                    }
                    for relay in &r.relays {
                        if let Some(candidate) = relay.candidate {
                            let c = &mut effect.after.credits[usize::from(relay.speaker)];
                            if candidate == *actor {
                                c.relay_accurate = c.relay_accurate.saturating_add(1);
                            } else {
                                c.relay_inaccurate = c.relay_inaccurate.saturating_add(1);
                            }
                        }
                    }
                }
            }
        }
        effect
    }
}
