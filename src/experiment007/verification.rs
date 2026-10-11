use crate::experiment006 as e6;
use e6::CredibilityUpdater;
use serde::{Deserialize, Serialize};

pub const ISSUER: &str = "lab-verification-006-v1";
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quarantine_recomputes_clamped_history_instead_of_subtracting_a_score() {
        let mut memory = Memory::neutral(5);
        for episode in 0..4 {
            memory.local.records.push(e6::Experience {
                key: e6::Reference { episode, local: 0 },
                group: None,
                source: Some(0),
                initial_candidate: Some(0),
                final_candidate: Some(0),
                received: vec![],
                relays: vec![],
                corrections: 0,
                retractions: 0,
                verified: None,
                announcement: None,
            });
            memory.receipts.push(Receipt {
                episode,
                state: VerificationState::Accepted {
                    actor: if episode == 3 { 1 } else { 0 },
                    delivery: usize::from(episode),
                },
            });
        }
        memory.recompute();
        assert_eq!(memory.local.credits[0].score, 1);
        memory.receipts[0].state = VerificationState::Quarantined {
            first: 0,
            conflicting: 1,
        };
        memory.recompute();
        assert_eq!(memory.local.credits[0].score, 1);
        assert_eq!(memory.local.credits[0].accurate, 2);
        assert_eq!(memory.local.credits[0].inaccurate, 1);
        memory.receipts[3].state = VerificationState::Quarantined {
            first: 1,
            conflicting: 0,
        };
        memory.recompute();
        assert_eq!(memory.local.credits[0].score, 2);
        assert_eq!(memory.local.credits[0].inaccurate, 0);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    Outcome(u8),
    Verdict(Option<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub version: String,
    pub issuer: String,
    pub authenticated_channel: bool,
    pub session: u64,
    pub episode: u8,
    pub serial: u16,
    pub kind: TokenKind,
}
impl Token {
    pub fn outcome(session: u64, episode: u8, edition: u16, actor: u8) -> Self {
        Self {
            version: "token-007-v1".into(),
            issuer: ISSUER.into(),
            authenticated_channel: true,
            session,
            episode,
            serial: u16::from(episode) * 4 + edition,
            kind: TokenKind::Outcome(actor),
        }
    }
    pub fn authentic(&self) -> bool {
        self.authenticated_channel && self.issuer == ISSUER
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationState {
    Accepted { actor: u8, delivery: usize },
    Quarantined { first: u8, conflicting: u8 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receipt {
    pub episode: u8,
    pub state: VerificationState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Acceptance {
    Accepted,
    Repeated,
    RejectedAuthentication,
    RejectedAssociation,
    Pending,
    Quarantined,
    ConflictRetained,
    Unverified,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenEntry {
    pub token: Token,
    pub acceptance: Acceptance,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory {
    pub local: e6::LocalMemory,
    pub checkpoint: [e6::Credit; 6],
    pub receipts: Vec<Receipt>,
    pub tokens: Vec<TokenEntry>,
}
impl Memory {
    pub fn neutral(observer: u8) -> Self {
        Self {
            local: e6::LocalMemory::neutral(observer),
            checkpoint: [e6::Credit::default(); 6],
            receipts: vec![],
            tokens: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.local.validate()?;
        if self.receipts.len() > 8
            || self.tokens.len() > 32
            || self.checkpoint.iter().any(|c| !(-2..=2).contains(&c.score))
        {
            return Err("unbounded007 local memory".into());
        }
        Ok(())
    }
    pub fn ingest(
        &mut self,
        view: e6::IngestView,
        policy: VerificationKind,
    ) -> Result<e6::Ingestion, String> {
        let result = e6::ingest(&self.local, view)?;
        if policy == VerificationKind::ConflictAware {
            // Forgotten accepted contributions become declared compact consequences.
            for key in &result.evicted {
                if let Some(record) = self.local.records.iter().find(|r| r.key == *key)
                    && let Some((actor, delivery)) = self.accepted(key.episode)
                {
                    apply_record(&mut self.checkpoint, record, actor, delivery);
                }
            }
            for (base, new) in self.checkpoint.iter_mut().zip(&result.after.credits) {
                base.corrections = new.corrections;
                base.retractions = new.retractions;
            }
        }
        self.local = result.after.clone();
        self.receipts.retain(|r| {
            self.local
                .records
                .iter()
                .any(|e| e.key.episode == r.episode)
        });
        if policy == VerificationKind::ConflictAware {
            self.recompute();
        }
        self.validate()?;
        Ok(result)
    }
    fn accepted(&self, episode: u8) -> Option<(u8, usize)> {
        self.receipts.iter().find_map(|r| {
            if r.episode == episode {
                match r.state {
                    VerificationState::Accepted { actor, delivery } => Some((actor, delivery)),
                    _ => None,
                }
            } else {
                None
            }
        })
    }
    fn recompute(&mut self) {
        let mut credits = self.checkpoint;
        let statuses = self.receipts.clone();
        for record in &mut self.local.records {
            let accepted = statuses.iter().find_map(|r| {
                if r.episode == record.key.episode {
                    match r.state {
                        VerificationState::Accepted { actor, delivery } => Some((actor, delivery)),
                        _ => None,
                    }
                } else {
                    None
                }
            });
            record.verified = None;
            if let Some((actor, delivery)) = accepted {
                apply_record(&mut credits, record, actor, delivery);
                record.verified = Some(e6::Verified {
                    feedback: delivery,
                    actor,
                    issuer: ISSUER.into(),
                });
            }
        }
        self.local.credits = credits;
    }
}
fn apply_record(
    credits: &mut [e6::Credit; 6],
    record: &e6::Experience,
    actor: u8,
    delivery: usize,
) {
    let mut r = record.clone();
    r.verified = None;
    let effect = e6::UpdateKind::Evidence.update(&e6::FeedbackView {
        delivery,
        memory: e6::LocalMemory {
            observer: 0,
            credits: *credits,
            records: vec![r],
        },
        received: e6::Feedback::VerifiedOutcome {
            episode: record.key.episode,
            actor,
            issuer: ISSUER.into(),
        },
    });
    *credits = effect.after.credits;
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationView {
    pub delivery: usize,
    pub completed_episode: u8,
    pub session: u64,
    pub memory: Memory,
    pub received: Token,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationEffect {
    pub after: Memory,
    pub acceptance: Acceptance,
    pub record_inspections: usize,
    pub unverifiable_eviction: bool,
    pub evicted_tokens: Vec<TokenEntry>,
}
pub trait VerificationPolicy {
    fn accept(&mut self, view: &VerificationView) -> VerificationEffect;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationKind {
    Frozen006,
    ConflictAware,
}
impl VerificationPolicy for VerificationKind {
    fn accept(&mut self, v: &VerificationView) -> VerificationEffect {
        let t = &v.received;
        let mut after = v.memory.clone();
        let eligible = t.episode <= v.completed_episode
            && after
                .local
                .records
                .iter()
                .any(|r| r.key.episode == t.episode);
        let eviction = !eligible
            && after.tokens.iter().any(|e| {
                e.token.episode == t.episode
                    && matches!(
                        e.acceptance,
                        Acceptance::Accepted
                            | Acceptance::Quarantined
                            | Acceptance::ConflictRetained
                    )
            });
        let status = if matches!(t.kind, TokenKind::Verdict(_)) {
            if let TokenKind::Verdict(verdict) = t.kind {
                let effect = e6::UpdateKind::Evidence.update(&e6::FeedbackView {
                    delivery: v.delivery,
                    memory: after.local.clone(),
                    received: e6::Feedback::Unverified {
                        episode: t.episode,
                        announcement: e6::Announcement::CollectiveVerdict(verdict),
                    },
                });
                after.local = effect.after;
            }
            Acceptance::Unverified
        } else if !t.authentic() || matches!(t.kind,TokenKind::Outcome(c) if c>=6) {
            Acceptance::RejectedAuthentication
        } else if *self == Self::ConflictAware
            && (t.version != "token-007-v1"
                || t.session != v.session
                || t.serial / 4 != u16::from(t.episode))
        {
            Acceptance::RejectedAssociation
        } else if !eligible {
            Acceptance::Pending
        } else {
            let TokenKind::Outcome(actor) = t.kind else {
                unreachable!()
            };
            if *self == Self::Frozen006 {
                // Deliberately call the unchanged historical acceptance/update implementation.
                let effect = e6::UpdateKind::Evidence.update(&e6::FeedbackView {
                    delivery: v.delivery,
                    memory: after.local.clone(),
                    received: e6::Feedback::VerifiedOutcome {
                        episode: t.episode,
                        actor,
                        issuer: t.issuer.clone(),
                    },
                });
                let status = if effect.integrity_conflicts > 0 {
                    Acceptance::ConflictRetained
                } else if effect.duplicate_verifications > 0 && effect.graded_sources == 0 {
                    Acceptance::Repeated
                } else {
                    Acceptance::Accepted
                };
                after.local = effect.after;
                status
            } else {
                let status =
                    if let Some(r) = after.receipts.iter_mut().find(|r| r.episode == t.episode) {
                        match r.state {
                            VerificationState::Accepted { actor: first, .. } if first != actor => {
                                r.state = VerificationState::Quarantined {
                                    first,
                                    conflicting: actor,
                                };
                                Acceptance::Quarantined
                            }
                            VerificationState::Accepted { .. } => Acceptance::Repeated,
                            VerificationState::Quarantined { .. } => Acceptance::Quarantined,
                        }
                    } else {
                        after.receipts.push(Receipt {
                            episode: t.episode,
                            state: VerificationState::Accepted {
                                actor,
                                delivery: v.delivery,
                            },
                        });
                        Acceptance::Accepted
                    };
                after.recompute();
                status
            }
        };
        after.tokens.push(TokenEntry {
            token: t.clone(),
            acceptance: status,
        });
        let evicted_tokens = if after.tokens.len() > 32 {
            vec![after.tokens.remove(0)]
        } else {
            vec![]
        };
        VerificationEffect {
            after,
            acceptance: status,
            record_inspections: v.memory.local.records.len(),
            unverifiable_eviction: eviction,
            evicted_tokens,
        }
    }
}
