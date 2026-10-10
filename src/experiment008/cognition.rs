use super::*;
use crate::{experiment005 as e5, experiment006 as e6};
use e6::ExperienceListener;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Perceived {
    Source(e5::Group),
    Artifact(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    pub group: Perceived,
    pub candidate: Option<u8>,
    pub weight: u16,
    pub contested: bool,
    pub observations: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub base: e5::Assessment,
    pub unweighted: e5::Belief,
    pub after: e5::Belief,
    pub contributions: Vec<Contribution>,
    pub inspections: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceView {
    pub incident: e5::ListenerView,
    pub credits: [e6::Credit; 6],
    pub findings: Vec<Finding>,
    pub reports: Vec<ReportedFinding>,
}
pub trait EvidenceListener {
    fn assess(&mut self, v: &EvidenceView) -> Assessment;
}
pub struct EvidenceAware {
    pub testimony: e6::ListenerKind,
}
fn group(source: &Descriptor) -> Perceived {
    source
        .visible_parent
        .map(|s| Perceived::Source(e5::Group::Speaker(s)))
        .unwrap_or(Perceived::Artifact(source.id))
}
impl EvidenceListener for EvidenceAware {
    fn assess(&mut self, v: &EvidenceView) -> Assessment {
        let (mut base, weights, _) = self.testimony.assess(e6::CredibilityView {
            incident: &v.incident,
            credits: &v.credits,
        });
        let own = e5::effective_original(&v.incident.original, &v.incident.observations);
        let anchor = e5::initial_belief(v.incident.id, &own);
        let mut groups: BTreeMap<Perceived, Contribution> = BTreeMap::new();
        let mut add = |key: Perceived, candidate: Option<u8>, weight: u16, contested: bool| {
            if candidate.is_none() && !contested {
                return;
            }
            if let Some(g) = groups.get_mut(&key) {
                g.observations += 1;
                if g.candidate != candidate || contested {
                    g.contested = true;
                }
                g.weight = g.weight.max(weight);
            } else {
                groups.insert(
                    key,
                    Contribution {
                        group: key,
                        candidate,
                        weight,
                        contested,
                        observations: 1,
                    },
                );
            }
        };
        if let Some(c) = anchor.candidate {
            add(
                Perceived::Source(e5::Group::Speaker(v.incident.id)),
                Some(c),
                anchor.scores[usize::from(c)],
                false,
            );
        }
        for g in &base.groups {
            // The first explicit report in a visible revision branch fixes its
            // perceived source. Later revisions retain that source identity.
            let finding = v.reports.iter().find(|r| {
                base.interpretations.iter().any(|i| {
                    i.message == r.message && i.root.is_some_and(|root| g.members.contains(&root))
                })
            });
            let key = finding
                .map(|r| group(&r.finding.source))
                .unwrap_or(Perceived::Source(g.group));
            let weight = if finding.is_some() {
                1
            } else {
                weights
                    .iter()
                    .find(|w| w.group == g.group)
                    .map(|w| w.weight)
                    .unwrap_or(0)
            };
            if g.contributes
                || g.contested
                || (finding.is_some()
                    && g.candidate.is_some()
                    && g.group != e5::Group::Speaker(v.incident.id))
            {
                add(key, g.candidate, weight, g.contested);
            }
        }
        for f in &v.findings {
            add(group(&f.source), f.candidate, 2, false);
        }
        let mut scores = [0; 6];
        let mut contributions: Vec<_> = groups.into_values().collect();
        for g in &mut contributions {
            if g.contested {
                g.weight = 0;
            }
            if let Some(c) = g.candidate {
                scores[usize::from(c)] += g.weight;
            }
        }
        let after = e5::Belief::from_scores(scores);
        base.after = after.clone();
        let unweighted = if self.testimony == e6::ListenerKind::Credibility {
            EvidenceAware {
                testimony: e6::ListenerKind::Frozen005,
            }
            .assess(v)
            .after
        } else {
            after.clone()
        };
        Assessment {
            base,
            unweighted,
            after,
            contributions,
            inspections: (v.incident.received.len() + v.findings.len() + v.reports.len())
                * if self.testimony == e6::ListenerKind::Credibility {
                    2
                } else {
                    1
                },
        }
    }
}
