use super::*;
use crate::experiment005 as e5;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListenerKind {
    Frozen005,
    Credibility,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weight {
    pub group: e5::Group,
    pub score: Option<i8>,
    pub verified_count: u16,
    pub weight: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cognition {
    pub episode: u8,
    pub agent: u8,
    pub update: usize,
    pub credit_input: [Credit; 6],
    pub unweighted: e5::Belief,
    pub weighted: e5::Belief,
    pub weights: Vec<Weight>,
}
pub struct CredibilityView<'a> {
    pub incident: &'a e5::ListenerView,
    pub credits: &'a [Credit; 6],
}
pub trait ExperienceListener {
    fn assess(&mut self, view: CredibilityView<'_>) -> (e5::Assessment, Vec<Weight>, e5::Belief);
}
impl ExperienceListener for ListenerKind {
    fn assess(&mut self, v: CredibilityView<'_>) -> (e5::Assessment, Vec<Weight>, e5::Belief) {
        let mut base = e5::Listener::assess(&mut e5::ListenerKind::Revision, v.incident);
        let unweighted = base.after.clone();
        let own = e5::effective_original(&v.incident.original, &v.incident.observations);
        let mut scores = e5::initial_belief(v.incident.id, &own).scores;
        let mut weights = vec![];
        for group in &mut base.groups {
            if !group.contributes {
                continue;
            }
            let (score, verified_count) = match group.group {
                e5::Group::Speaker(s) => (
                    Some(v.credits[usize::from(s)].score),
                    v.credits[usize::from(s)].verified_count(),
                ),
                e5::Group::OpaqueMessage(_) => (None, 0),
            };
            let weight = if *self == Self::Frozen005 {
                1
            } else {
                score.map(|s| (i16::from(s) + 1).max(0) as u16).unwrap_or(1)
            };
            weights.push(Weight {
                group: group.group,
                score,
                verified_count,
                weight,
            });
            if weight == 0 {
                group.contributes = false;
            }
            if let Some(c) = group.candidate {
                scores[usize::from(c)] += weight;
            }
        }
        if *self == Self::Credibility {
            base.after = e5::Belief::from_scores(scores);
            base.mechanism =
                "credibility-listener-006-v1 on unchanged revision-listener-005-v1 standing".into();
        }
        (base, weights, unweighted)
    }
}
pub(crate) struct Adapter {
    pub episode: u8,
    pub kind: ListenerKind,
    pub credit: [Credit; 6],
    pub log: Rc<RefCell<Vec<Cognition>>>,
}
impl e5::Listener for Adapter {
    fn assess(&mut self, v: &e5::ListenerView) -> e5::Assessment {
        let (a, weights, unweighted) = self.kind.assess(CredibilityView {
            incident: v,
            credits: &self.credit,
        });
        let mut log = self.log.borrow_mut();
        let update = log.len();
        log.push(Cognition {
            episode: self.episode,
            agent: v.id,
            update,
            credit_input: self.credit,
            unweighted,
            weighted: a.after.clone(),
            weights,
        });
        a
    }
}
