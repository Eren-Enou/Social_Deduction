use super::*;
use crate::{experiment006 as e6, experiment007 as e7, experiment009 as e9, experiment010 as e10};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Group {
    Speaker(u8),
    Opaque(Reference),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialView {
    pub observer: u8,
    pub family: MethodId,
    pub personal: Counts,
    pub memory: SocialMemory,
    pub credits: [e6::Credit; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interpretation {
    pub message: Reference,
    pub standing: Standing,
    pub group: Option<Group>,
    pub weight: u8,
    pub discounted: bool,
    pub origin_verified: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialEstimate {
    pub input: SocialView,
    pub kind: SocialKind,
    pub interpretations: Vec<Interpretation>,
    pub positive: u8,
    pub negative: u8,
    pub missing: u8,
    pub nonmissing: u8,
    pub personal_accuracy: e7::Probability,
    pub personal_missing: e7::Probability,
    pub effective_accuracy: e7::Probability,
    pub effective_missing: e7::Probability,
}
pub trait SocialInterpreter {
    fn assess(&mut self, v: &SocialView) -> SocialEstimate;
}
fn visible_group(memory: &SocialMemory, m: &Message) -> Option<Group> {
    let root = branch_root(memory, m)?;
    let mut x = memory.received.iter().find(|m| m.reference == root)?;
    for _ in 0..=3 {
        if let Some(parent) = x.payload.parent {
            let Some(next) = memory.received.iter().find(|y| y.reference == parent) else {
                return Some(Group::Opaque(parent));
            };
            if next.payload.family != m.payload.family || next.reference >= x.reference {
                return None;
            }
            x = next;
        } else {
            return Some(Group::Speaker(
                x.payload.claimed_origin.unwrap_or(x.speaker),
            ));
        }
    }
    None
}
fn cap(a: u32, b: u32) -> (u8, u8) {
    let n = a + b;
    if n <= 4 {
        (a as u8, b as u8)
    } else {
        ((4 * a / n) as u8, (4 * b / n) as u8)
    }
}
impl SocialInterpreter for SocialKind {
    fn assess(&mut self, v: &SocialView) -> SocialEstimate {
        let mut interpreted = vec![];
        let mut per_speaker = BTreeMap::new();
        for m in v
            .memory
            .received
            .iter()
            .filter(|m| m.payload.family == v.family && m.speaker != v.observer)
        {
            let state = standing(&v.memory, m);
            let group = if *self == Self::NaiveSocial {
                Some(Group::Speaker(m.speaker))
            } else {
                visible_group(&v.memory, m)
            };
            let index = interpreted.len();
            interpreted.push(Interpretation {
                message: m.reference,
                standing: state,
                group,
                weight: 0,
                discounted: false,
                origin_verified: false,
            });
            if matches!(
                state,
                Standing::Active | Standing::Retracted | Standing::Contested
            ) && *self != Self::DirectOnly
                && let Some(old) = per_speaker.insert(m.speaker, index)
            {
                interpreted[old].discounted = true;
            }
        }
        let mut groups: BTreeMap<Group, Vec<usize>> = BTreeMap::new();
        for i in per_speaker.values() {
            if interpreted[*i].standing == Standing::Active
                && let Some(g) = interpreted[*i].group.clone()
                && g != Group::Speaker(v.observer)
            {
                groups.entry(g).or_default().push(*i);
            }
        }
        let mut p = 0;
        let mut n = 0;
        let mut miss = 0;
        let mut non = 0;
        for (group, indices) in groups {
            let messages: Vec<_> = indices
                .iter()
                .map(|i| {
                    v.memory
                        .received
                        .iter()
                        .find(|m| m.reference == interpreted[*i].message)
                        .unwrap()
                })
                .collect();
            let first = messages[0];
            if messages.iter().any(|m| {
                m.payload.rating != first.payload.rating || m.payload.counts != first.payload.counts
            }) {
                for i in indices {
                    interpreted[i].standing = Standing::Contested;
                }
                continue;
            }
            let weight = if *self == Self::CredibilityWeightedSocial {
                match group {
                    Group::Speaker(id) => {
                        (1 + i16::from(v.credits[usize::from(id)].score)).clamp(0, 3) as u8
                    }
                    Group::Opaque(_) => 1,
                }
            } else {
                1
            };
            for (j, i) in indices.into_iter().enumerate() {
                interpreted[i].weight = if j == 0 { weight } else { 0 };
                interpreted[i].discounted = j > 0;
            }
            match first.payload.rating {
                Rating::Favorable => p += u32::from(weight),
                Rating::Unfavorable => n += u32::from(weight),
                _ => {}
            }
            if let Some(c) = first.payload.counts {
                if c.missing > c.nonmissing {
                    miss += u32::from(weight);
                }
                if c.nonmissing > c.missing {
                    non += u32::from(weight);
                }
            }
        }
        let (positive, negative) = cap(p, n);
        let (missing, nonmissing) = cap(miss, non);
        let personal_accuracy = v.personal.accuracy(1);
        let personal_missing = v.personal.missingness(1);
        let effective_accuracy = e7::Probability {
            numerator: personal_accuracy.numerator + u64::from(positive),
            denominator: personal_accuracy.denominator + u64::from(positive + negative),
        };
        let effective_missing = e7::Probability {
            numerator: personal_missing.numerator + u64::from(missing),
            denominator: personal_missing.denominator + u64::from(missing + nonmissing),
        };
        SocialEstimate {
            input: v.clone(),
            kind: *self,
            interpretations: interpreted,
            positive,
            negative,
            missing,
            nonmissing,
            personal_accuracy,
            personal_missing,
            effective_accuracy,
            effective_missing,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationView {
    pub personal: e10::InvestigationView,
    pub social: SocialMemory,
    pub listener: SocialKind,
}
impl Deref for InvestigationView {
    type Target = e10::InvestigationView;
    fn deref(&self) -> &Self::Target {
        &self.personal
    }
}
impl InvestigationView {
    pub fn validate(&self) -> Result<(), String> {
        self.personal.validate()?;
        self.social.validate()?;
        if self.social.observer != self.local.id {
            return Err("foreign investigation social memory".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationChoice {
    pub selected: e10::InvestigationChoice,
    pub personal_proposal: e10::InvestigationChoice,
    pub social: Vec<SocialEstimate>,
}
impl Deref for InvestigationChoice {
    type Target = e10::InvestigationChoice;
    fn deref(&self) -> &Self::Target {
        &self.selected
    }
}
impl DerefMut for InvestigationChoice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.selected
    }
}
pub trait InvestigationPolicy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice;
}
pub struct Policy {
    personal: e10::Policy,
    social: e9::Policy,
}
impl Policy {
    pub fn new(kind: e10::PolicyKind, seed: u64, id: u8) -> Self {
        Self {
            personal: e10::Policy::new(kind, seed, id),
            social: e9::Policy::new(e9::PolicyKind::InformationValueHeuristic, seed, id),
        }
    }
}
impl InvestigationPolicy for Policy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice {
        let personal_proposal = e10::InvestigationPolicy::choose(&mut self.personal, &v.personal);
        let mut selected = personal_proposal.clone();
        let social: Vec<_> = v
            .methods
            .iter()
            .map(|m| {
                v.listener.clone().assess(&SocialView {
                    observer: v.local.id,
                    family: m.family.clone(),
                    personal: v.experience.get(&m.family).copied().unwrap_or_default(),
                    memory: v.social.clone(),
                    credits: v.local.evidence.credits,
                })
            })
            .collect();
        if v.listener != SocialKind::DirectOnly {
            let mut effective = v.local.clone();
            for ((s, e), out) in effective
                .catalog
                .iter_mut()
                .zip(&social)
                .zip(&mut selected.expectations)
            {
                s.declared_accuracy =
                    (100 * e.effective_accuracy.numerator / e.effective_accuracy.denominator) as u8;
                s.declared_missing =
                    (100 * e.effective_missing.numerator / e.effective_missing.denominator) as u8;
                out.effective_accuracy = e.effective_accuracy;
                out.effective_missing = e.effective_missing;
                out.effective_accuracy_percent = s.declared_accuracy;
                out.effective_missing_percent = s.declared_missing;
            }
            let result = e9::InvestigationPolicy::choose(&mut self.social, &effective);
            selected.action = result.action;
            selected.considered = result.considered;
            selected.mechanism =
                "social-target-011-v1; separate bounded pseudo-support, unchanged009 gate".into();
        }
        InvestigationChoice {
            selected,
            personal_proposal,
            social,
        }
    }
}
pub fn validate_investigation(v: &InvestigationView, a: InvestigationAction) -> Result<(), String> {
    crate::experiment010::validate_investigation(&v.personal, a)
}
