use super::*;
use crate::{experiment007 as e7, experiment009 as e9};
use e9::InvestigationPolicy as FrozenPolicy;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicMethod {
    pub source: e9::PublicSource,
    pub family: MethodId,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationView {
    pub local: e9::InvestigationView,
    pub methods: Vec<PublicMethod>,
    pub experience: BTreeMap<MethodId, Counts>,
}
impl InvestigationView {
    pub fn validate(&self) -> Result<(), String> {
        if self.methods.len() != self.local.catalog.len()
            || self.experience.len() > 4
            || self.methods.len() > 4
        {
            return Err("invalid bounded local method selection view".into());
        }
        let mut ids = BTreeSet::new();
        for (m, s) in self.methods.iter().zip(&self.local.catalog) {
            m.family.validate()?;
            m.source.validate()?;
            if m.source != *s || !ids.insert(s.descriptor.id) {
                return Err("method identity does not match accessible catalog".into());
            }
        }
        for m in self.experience.keys() {
            m.validate()?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyKind {
    NeverInvestigate,
    FirstAccessible,
    CheapestAvailable,
    SeededRandomTarget,
    Frozen009InformationValue,
    Frozen009BudgetAware,
    AdvertisedQuality,
    ExperienceAdjusted,
    ConservativeLearner,
    ExploratoryLearner,
}
pub const POLICIES: [PolicyKind; 10] = [
    PolicyKind::NeverInvestigate,
    PolicyKind::FirstAccessible,
    PolicyKind::CheapestAvailable,
    PolicyKind::SeededRandomTarget,
    PolicyKind::Frozen009InformationValue,
    PolicyKind::Frozen009BudgetAware,
    PolicyKind::AdvertisedQuality,
    PolicyKind::ExperienceAdjusted,
    PolicyKind::ConservativeLearner,
    PolicyKind::ExploratoryLearner,
];
impl PolicyKind {
    pub fn frozen(self) -> e9::PolicyKind {
        match self {
            Self::NeverInvestigate => e9::PolicyKind::NeverInvestigate,
            Self::FirstAccessible => e9::PolicyKind::FirstAccessible,
            Self::CheapestAvailable => e9::PolicyKind::CheapestAvailable,
            Self::SeededRandomTarget => e9::PolicyKind::SeededRandomTarget,
            Self::Frozen009BudgetAware => e9::PolicyKind::BudgetAwareTarget,
            _ => e9::PolicyKind::InformationValueHeuristic,
        }
    }
    pub fn adaptive(self) -> bool {
        matches!(
            self,
            Self::ExperienceAdjusted | Self::ConservativeLearner | Self::ExploratoryLearner
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodExpectation {
    pub source: u8,
    pub family: MethodId,
    pub counts: Counts,
    pub advertised_accuracy: e7::Probability,
    pub advertised_missing: e7::Probability,
    pub learned_accuracy: e7::Probability,
    pub learned_missing: e7::Probability,
    pub effective_accuracy: e7::Probability,
    pub effective_missing: e7::Probability,
    pub experienced_accuracy_used: bool,
    pub experienced_missing_used: bool,
    pub effective_accuracy_percent: u8,
    pub effective_missing_percent: u8,
}
fn percent(p: e7::Probability) -> u8 {
    (100 * p.numerator / p.denominator) as u8
}
pub fn expectation(kind: PolicyKind, m: &PublicMethod, counts: Counts) -> MethodExpectation {
    let advertised_accuracy = e7::Probability {
        numerator: u64::from(m.source.declared_accuracy),
        denominator: 100,
    };
    let advertised_missing = e7::Probability {
        numerator: u64::from(m.source.declared_missing),
        denominator: 100,
    };
    let learned_accuracy = counts.accuracy(1);
    let learned_missing = counts.missingness(1);
    let conservative = kind == PolicyKind::ConservativeLearner;
    let use_accuracy = kind.adaptive() && (!conservative || counts.graded() >= 3);
    let use_missing = kind.adaptive() && (!conservative || counts.observed() >= 3);
    let effective_accuracy = if use_accuracy {
        counts.accuracy(if conservative { 2 } else { 1 })
    } else {
        advertised_accuracy
    };
    let effective_missing = if use_missing {
        counts.missingness(if conservative { 2 } else { 1 })
    } else {
        advertised_missing
    };
    MethodExpectation {
        source: m.source.descriptor.id,
        family: m.family.clone(),
        counts,
        advertised_accuracy,
        advertised_missing,
        learned_accuracy,
        learned_missing,
        effective_accuracy,
        effective_missing,
        experienced_accuracy_used: use_accuracy,
        experienced_missing_used: use_missing,
        effective_accuracy_percent: percent(effective_accuracy),
        effective_missing_percent: percent(effective_missing),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationChoice {
    pub action: e9::InvestigationAction,
    pub considered: Vec<e9::Estimate>,
    pub mechanism: String,
    pub expectations: Vec<MethodExpectation>,
    pub exploration: bool,
    pub opportunity: u8,
    pub exploitation_proposal: e9::InvestigationAction,
}
pub trait InvestigationPolicy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice;
}
pub struct Policy {
    pub kind: PolicyKind,
    pub frozen: e9::Policy,
    pub opportunities: u8,
}
impl Policy {
    pub fn new(kind: PolicyKind, seed: u64, id: u8) -> Self {
        Self {
            kind,
            frozen: e9::Policy::new(kind.frozen(), seed, id),
            opportunities: 0,
        }
    }
}
impl InvestigationPolicy for Policy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice {
        let available =
            v.local.id < 6 && v.local.stage == e9::Stage::Investigation && !v.local.used;
        if available {
            self.opportunities = self.opportunities.saturating_add(1);
        }
        let expectations: Vec<_> = v
            .methods
            .iter()
            .map(|m| {
                expectation(
                    self.kind,
                    m,
                    v.experience.get(&m.family).copied().unwrap_or_default(),
                )
            })
            .collect();
        let mut effective = v.local.clone();
        for (source, expectation) in effective.catalog.iter_mut().zip(&expectations) {
            source.declared_accuracy = expectation.effective_accuracy_percent;
            source.declared_missing = expectation.effective_missing_percent;
        }
        let selected = self.frozen.choose(&effective);
        let exploitation_proposal = selected.action;
        let exploration_due = available
            && self.kind == PolicyKind::ExploratoryLearner
            && self.opportunities.is_multiple_of(4);
        let mut action = exploitation_proposal;
        let mut exploration = false;
        if exploration_due {
            let mut legal: Vec<_> = v
                .methods
                .iter()
                .filter(|m| m.source.cost <= v.local.balance)
                .collect();
            legal.sort_by(|a, b| {
                v.experience
                    .get(&a.family)
                    .copied()
                    .unwrap_or_default()
                    .observed()
                    .cmp(
                        &v.experience
                            .get(&b.family)
                            .copied()
                            .unwrap_or_default()
                            .observed(),
                    )
                    .then(a.source.cost.cmp(&b.source.cost))
                    .then(a.source.method_key.cmp(&b.source.method_key))
            });
            if let Some(m) = legal.first() {
                action = e9::InvestigationAction::Inspect(m.source.descriptor.id);
                exploration = true;
            }
        }
        InvestigationChoice {
            action,
            considered: selected.considered,
            mechanism: format!(
                "adaptive-target-010-v1::{:?}; unchanged009 enumeration and cost gate; personal count estimates; exploration every fourth legal opportunity",
                self.kind
            ),
            expectations,
            exploration,
            opportunity: self.opportunities,
            exploitation_proposal,
        }
    }
}
pub fn validate_investigation(
    v: &InvestigationView,
    action: e9::InvestigationAction,
) -> Result<(), String> {
    v.validate()?;
    e9::validate_investigation(&v.local, action)
}
