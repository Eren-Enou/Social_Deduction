use super::*;
use crate::{experiment005 as e5, experiment006 as e6, experiment008 as e8};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Partial,
    Physical,
    Specialized,
    Derived,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicProvenance {
    Unknown,
    SeparateAcquisition,
    Observation(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityOrigin {
    InstrumentSpecification,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicSource {
    pub descriptor: e8::Descriptor,
    pub method_key: String,
    pub cost: u8,
    pub category: Category,
    pub declared_accuracy: u8,
    pub declared_missing: u8,
    pub quality_origin: QualityOrigin,
    pub provenance: PublicProvenance,
    pub coverage: Vec<u8>,
}
impl PublicSource {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=3).contains(&self.cost)
            || self.declared_accuracy > 100
            || self.declared_missing > 100
            || self.descriptor.description.trim().is_empty()
            || self.descriptor.description.len() > 128
            || self.method_key.trim().is_empty()
            || self.method_key.len() > 32
            || self.descriptor.visible_parent.is_some_and(|id| id >= 6)
            || self.coverage.is_empty()
            || !crate::experiment004::unique_ids(&self.coverage)
        {
            return Err("invalid bounded public source description/cost/coverage".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub descriptor: e8::Descriptor,
    pub method_key: String,
    pub cost: u8,
    pub category: Category,
    pub declared_accuracy: u8,
    pub declared_missing: u8,
    pub coverage: Vec<u8>,
    pub inspectors: Vec<u8>,
    pub separate_acquisition_disclosed: bool,
    pub candidate: Option<u8>,
    pub actual_root: Root,
}
impl Source {
    pub fn public(&self) -> PublicSource {
        PublicSource {
            descriptor: self.descriptor.clone(),
            method_key: self.method_key.clone(),
            cost: self.cost,
            category: self.category,
            declared_accuracy: self.declared_accuracy,
            declared_missing: self.declared_missing,
            coverage: self.coverage.clone(),
            quality_origin: QualityOrigin::InstrumentSpecification,
            provenance: self
                .descriptor
                .visible_parent
                .map(PublicProvenance::Observation)
                .unwrap_or(if self.separate_acquisition_disclosed {
                    PublicProvenance::SeparateAcquisition
                } else {
                    PublicProvenance::Unknown
                }),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationView {
    pub id: u8,
    pub episode: u8,
    pub episodes_remaining: u8,
    pub evidence: EvidenceView,
    pub belief: e5::Belief,
    pub testimony: e6::ListenerKind,
    pub catalog: Vec<PublicSource>,
    pub balance: u8,
    pub stage: Stage,
    pub used: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PossibleResult {
    pub candidate: u8,
    pub belief: e5::Belief,
    pub decision_changed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estimate {
    pub source: u8,
    pub method_key: String,
    pub cost: u8,
    pub affordable: bool,
    pub known_overlap: bool,
    pub outcomes: Vec<PossibleResult>,
    pub usefulness: u16,
    pub relevance: u16,
    pub reservation: bool,
    pub score: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationChoice {
    pub action: InvestigationAction,
    pub considered: Vec<Estimate>,
    pub mechanism: String,
}
pub trait InvestigationPolicy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyKind {
    NeverInvestigate,
    FirstAccessible,
    SeededRandomTarget,
    CheapestAvailable,
    ConflictDrivenTarget,
    InformationValueHeuristic,
    BudgetAwareTarget,
}
pub const POLICIES: [PolicyKind; 7] = [
    PolicyKind::NeverInvestigate,
    PolicyKind::FirstAccessible,
    PolicyKind::SeededRandomTarget,
    PolicyKind::CheapestAvailable,
    PolicyKind::ConflictDrivenTarget,
    PolicyKind::InformationValueHeuristic,
    PolicyKind::BudgetAwareTarget,
];
pub struct Policy {
    pub kind: PolicyKind,
    pub rng: crate::Rng,
}
impl Policy {
    pub fn new(kind: PolicyKind, seed: u64, id: u8) -> Self {
        Self {
            kind,
            rng: crate::Rng::new(seed ^ (0x9009 + u64::from(id))),
        }
    }
}
pub fn estimate(v: &InvestigationView, s: &PublicSource) -> Estimate {
    // No observer Source is accepted here: enumerate hypothetical local observations only.
    let known = s.descriptor.visible_parent.and_then(|parent| {
        let a = EvidenceAware {
            testimony: v.testimony,
        }
        .assess(&v.evidence);
        a.contributions
            .iter()
            .find(|g| {
                g.group == Perceived::Source(e5::Group::Speaker(parent))
                    && !g.contested
                    && g.weight > 0
            })
            .and_then(|g| g.candidate)
    });
    let candidates = known
        .map(|id| vec![id])
        .unwrap_or_else(|| s.coverage.clone());
    let outcomes: Vec<_> = candidates
        .iter()
        .map(|candidate| {
            let mut ev = v.evidence.clone();
            ev.findings.push(Finding {
                source: s.descriptor.clone(),
                candidate: Some(*candidate),
                acquisition: Acquisition {
                    episode: v.episode,
                    observer: v.id,
                    local: ev.findings.len(),
                },
            });
            let belief = EvidenceAware {
                testimony: v.testimony,
            }
            .assess(&ev)
            .after;
            PossibleResult {
                candidate: *candidate,
                decision_changed: belief.candidate != v.belief.candidate,
                belief,
            }
        })
        .collect();
    let quality = u32::from(s.declared_accuracy) * u32::from(100 - s.declared_missing);
    let changed = outcomes.iter().filter(|o| o.decision_changed).count() as u32;
    let usefulness = (1000 * changed * quality / (outcomes.len() as u32 * 10000)) as u16;
    let mut competing: Vec<_> = v
        .belief
        .scores
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(id, _)| id as u8)
        .collect();
    if competing.is_empty() {
        competing = (0..6).collect();
    }
    let covered = competing
        .iter()
        .filter(|id| s.coverage.contains(id))
        .count() as u32;
    let relevance = (1000 * covered * quality / (competing.len() as u32 * 10000)) as u16;
    let affordable = s.cost <= v.balance;
    let reservation =
        affordable && v.episodes_remaining > 0 && v.balance - s.cost < v.episodes_remaining.min(2);
    Estimate {
        source: s.descriptor.id,
        method_key: s.method_key.clone(),
        cost: s.cost,
        affordable,
        known_overlap: known.is_some(),
        outcomes,
        usefulness,
        relevance,
        reservation,
        score: 0,
    }
}
impl InvestigationPolicy for Policy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice {
        let mut considered: Vec<_> = v.catalog.iter().map(|s| estimate(v, s)).collect();
        for e in &mut considered {
            e.score = match self.kind {
                PolicyKind::InformationValueHeuristic => {
                    i32::from(e.usefulness) - 100 * i32::from(e.cost)
                }
                PolicyKind::BudgetAwareTarget => {
                    i32::from(e.usefulness)
                        - 150 * i32::from(e.cost)
                        - 150 * i32::from(e.reservation)
                }
                PolicyKind::ConflictDrivenTarget => i32::from(e.relevance),
                _ => 0,
            };
        }
        let mut legal: Vec<_> = considered.iter().filter(|e| e.affordable).collect();
        let available = v.id < 6 && v.stage == Stage::Investigation && !v.used;
        let conflict =
            v.belief.candidate.is_none() || v.belief.scores.iter().filter(|s| **s > 0).count() > 1;
        let selected = if !available
            || self.kind == PolicyKind::NeverInvestigate
            || (self.kind == PolicyKind::ConflictDrivenTarget && !conflict)
        {
            None
        } else if self.kind == PolicyKind::SeededRandomTarget {
            if legal.is_empty() {
                None
            } else {
                Some(legal[(self.rng.draw() % legal.len() as u64) as usize])
            }
        } else if self.kind == PolicyKind::FirstAccessible {
            legal.first().copied()
        } else {
            legal.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then(a.cost.cmp(&b.cost))
                    .then(a.method_key.cmp(&b.method_key))
            });
            legal.first().copied().filter(|e| {
                !matches!(
                    self.kind,
                    PolicyKind::InformationValueHeuristic | PolicyKind::BudgetAwareTarget
                ) || e.score > 0
            })
        };
        let action = selected
            .map(|s| InvestigationAction::Inspect(s.source))
            .unwrap_or(InvestigationAction::Decline);
        InvestigationChoice {
            action,
            considered,
            mechanism: format!(
                "target-policy-009-v1::{:?}; strict positive value gate; ties lower actual cost then stable method key; First preserves order; Random uses own stream",
                self.kind
            ),
        }
    }
}
pub fn validate_investigation(v: &InvestigationView, a: InvestigationAction) -> Result<(), String> {
    if v.id >= 6 || v.stage != Stage::Investigation || v.used {
        return Err("opportunity unavailable or consumed".into());
    }
    if let InvestigationAction::Inspect(id) = a {
        let s = v
            .catalog
            .iter()
            .find(|s| s.descriptor.id == id)
            .ok_or("source inaccessible")?;
        s.validate()?;
        if v.balance < s.cost {
            return Err("insufficient effort for actual cost".into());
        }
    }
    Ok(())
}
