use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialMemory {
    pub version: String,
    pub observer: u8,
    pub received: Vec<Message>,
    pub sent: Vec<Message>,
}
impl SocialMemory {
    pub fn neutral(observer: u8) -> Self {
        Self {
            version: "social-memory-011-v1".into(),
            observer,
            received: vec![],
            sent: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "social-memory-011-v1"
            || self.observer >= 6
            || self.received.len() > 12
            || self.sent.len() > 8
        {
            return Err("invalid bounded social memory".into());
        }
        for collection in [&self.received, &self.sent] {
            let mut keys = BTreeSet::new();
            for m in collection {
                m.payload.validate()?;
                if m.version != "method-report-011-v1"
                    || m.speaker >= 6
                    || !m.reference.valid()
                    || !keys.insert(m.reference)
                    || m.revision.is_some_and(|r| !r.valid() || r >= m.reference)
                    || m.payload.parent.is_some_and(|r| r >= m.reference)
                {
                    return Err("invalid social message identity/reference".into());
                }
            }
        }
        if self.sent.iter().any(|m| m.speaker != self.observer) {
            return Err("foreign sent message".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialIngestion {
    pub before: SocialMemory,
    pub message: Message,
    pub own: bool,
    pub after: SocialMemory,
    pub evicted: Vec<Reference>,
    pub duplicate: bool,
}
pub fn ingest(
    memory: &SocialMemory,
    message: Message,
    own: bool,
) -> Result<SocialIngestion, String> {
    memory.validate()?;
    let mut after = memory.clone();
    let records = if own {
        &mut after.sent
    } else {
        &mut after.received
    };
    let duplicate = records.iter().any(|m| m.reference == message.reference);
    let mut evicted = vec![];
    if !duplicate {
        records.push(message.clone());
        if records.len() > if own { 8 } else { 12 } {
            evicted.push(records.remove(0).reference);
        }
    }
    after.validate()?;
    Ok(SocialIngestion {
        before: memory.clone(),
        message,
        own,
        after,
        evicted,
        duplicate,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standing {
    Active,
    Superseded,
    Retracted,
    Contested,
    Unresolved,
}
/// Only retained received links are walked. No observer archive enters standing.
pub fn branch_root(memory: &SocialMemory, m: &Message) -> Option<Reference> {
    let mut current = m;
    let mut checked = BTreeSet::new();
    while let Some(target) = current.revision {
        if !checked.insert(current.reference) {
            return None;
        }
        current = memory.received.iter().find(|x| {
            x.reference == target && x.speaker == m.speaker && x.payload.family == m.payload.family
        })?;
    }
    Some(current.reference)
}
pub fn standing(memory: &SocialMemory, m: &Message) -> Standing {
    let Some(root) = branch_root(memory, m) else {
        return Standing::Unresolved;
    };
    if memory.received.iter().any(|x| {
        x.revision == Some(m.reference)
            && x.speaker == m.speaker
            && x.payload.family == m.payload.family
    }) {
        return Standing::Superseded;
    }
    let leaves: Vec<_> = memory
        .received
        .iter()
        .filter(|x| {
            branch_root(memory, x) == Some(root)
                && !memory.received.iter().any(|y| {
                    y.revision == Some(x.reference)
                        && y.speaker == x.speaker
                        && y.payload.family == x.payload.family
                })
        })
        .collect();
    if leaves.iter().any(|x| {
        x.retract != m.retract
            || x.payload.rating != m.payload.rating
            || x.payload.counts != m.payload.counts
    }) {
        return Standing::Contested;
    }
    if m.retract {
        Standing::Retracted
    } else {
        Standing::Active
    }
}
