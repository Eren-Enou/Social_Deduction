use super::*;
use crate::experiment007 as e7;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodProcess {
    pub family: MethodId,
    pub accuracy_before: u8,
    pub missing_before: u8,
    pub accuracy_after: u8,
    pub missing_after: u8,
    pub shift_episode: Option<u8>,
}
impl MethodProcess {
    pub fn validate(&self) -> Result<(), String> {
        self.family.validate()?;
        if [
            self.accuracy_before,
            self.accuracy_after,
            self.missing_before,
            self.missing_after,
        ]
        .iter()
        .any(|n| *n > 4)
            || self.shift_episode.is_some_and(|e| e >= 8)
        {
            return Err("invalid hidden method process".into());
        }
        Ok(())
    }
    pub fn accuracy(&self, episode: u8) -> e7::Probability {
        e7::Probability {
            numerator: u64::from(if self.shift_episode.is_some_and(|e| episode >= e) {
                self.accuracy_after
            } else {
                self.accuracy_before
            }),
            denominator: 4,
        }
    }
    pub fn missingness(&self, episode: u8) -> e7::Probability {
        e7::Probability {
            numerator: u64::from(if self.shift_episode.is_some_and(|e| episode >= e) {
                self.missing_after
            } else {
                self.missing_before
            }),
            denominator: 4,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackPlan {
    pub after_episode: u8,
    pub about_episode: u8,
    pub recipients: Vec<u8>,
    pub token: e7::Token,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodFeedbackAudit {
    pub observer: u8,
    pub plan: usize,
    pub actual_actor: u8,
    pub correct: Option<bool>,
    pub effect: MethodFeedback,
}
pub fn validate_method_ingestion(
    input: &MethodIngestView,
    effect: &MethodIngestion,
) -> Result<(), String> {
    effect.after.validate()?;
    if effect.input != *input || effect.after.observer != input.memory.observer {
        return Err("inconsistent custom method ingestion".into());
    }
    Ok(())
}
pub fn validate_method_feedback(
    input: &MethodFeedbackView,
    effect: &MethodFeedback,
) -> Result<(), String> {
    effect.after.validate()?;
    if effect.input != *input || effect.after.observer != input.memory.observer {
        return Err("inconsistent custom method feedback".into());
    }
    Ok(())
}
