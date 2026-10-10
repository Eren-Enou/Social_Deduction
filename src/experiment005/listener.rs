use super::*;
use crate::experiment004 as e4;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListenerView {
    pub id: u8,
    pub original: IndividualState,
    pub observations: Vec<Observation>,
    pub prior: Belief,
    pub received: Vec<Message>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Standing {
    Active,
    Superseded,
    Retracted,
    Disputed,
    MissingOriginal,
    InvalidRevision,
    IgnoredRepeat,
    Uncertainty,
    UncertainProvenance,
    IgnoredFrozenRevision,
    SelfIgnored,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interpretation {
    pub message: usize,
    pub root: Option<usize>,
    pub group: Option<Group>,
    pub standing: Standing,
    pub verification: Verification,
    pub checked: Vec<usize>,
    pub origin_verified: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupAssessment {
    pub group: Group,
    pub members: Vec<usize>,
    pub candidate: Option<u8>,
    pub contested: bool,
    pub contributes: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub after: Belief,
    pub interpretations: Vec<Interpretation>,
    pub groups: Vec<GroupAssessment>,
    pub mechanism: String,
}
pub trait Listener {
    fn assess(&mut self, view: &ListenerView) -> Assessment;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListenerKind {
    Frozen004,
    Revision,
}
pub fn as_listener_view(v: &AgentView) -> ListenerView {
    ListenerView {
        id: v.id,
        original: v.original.clone(),
        observations: v.observations.clone(),
        prior: v.current.clone(),
        received: v.received.clone(),
    }
}
impl Listener for ListenerKind {
    fn assess(&mut self, v: &ListenerView) -> Assessment {
        let own = effective_original(&v.original, &v.observations);
        let originals = original_messages(&v.received);
        if *self == Self::Frozen004 {
            let old = e4::Listener::assess(
                &mut e4::ListenerKind::Provenance,
                &e4::ListenerView {
                    id: v.id,
                    original: own,
                    prior: v.prior.clone(),
                    messages: originals,
                },
            );
            let interpretations = v
                .received
                .iter()
                .map(|m| {
                    let a = old.sources.iter().find(|a| a.message == m.id);
                    Interpretation {
                        message: m.id,
                        root: if a.is_some() { Some(m.id) } else { None },
                        group: a.and_then(|a| a.group),
                        standing: match a.map(|a| a.treatment) {
                            None => Standing::IgnoredFrozenRevision,
                            Some(e4::Treatment::Accepted) => Standing::Active,
                            Some(e4::Treatment::SelfIgnored) => Standing::SelfIgnored,
                            Some(e4::Treatment::UncertaintyIgnored) => Standing::Uncertainty,
                            Some(e4::Treatment::UncertainProvenance) => {
                                Standing::UncertainProvenance
                            }
                            _ => Standing::IgnoredRepeat,
                        },
                        verification: a.map(|a| a.verification).unwrap_or(Verification::NotUsed),
                        checked: a.map(|a| a.checked_messages.clone()).unwrap_or_default(),
                        origin_verified: false,
                    }
                })
                .collect();
            let groups = old
                .perceived_groups
                .iter()
                .map(|&g| {
                    let members: Vec<_> = old
                        .sources
                        .iter()
                        .filter(|s| s.group == Some(g) && s.treatment == e4::Treatment::Accepted)
                        .map(|s| s.message)
                        .collect();
                    let candidate = members
                        .first()
                        .and_then(|id| v.received.iter().find(|m| m.id == *id))
                        .and_then(|m| m.body.candidate());
                    GroupAssessment {
                        group: g,
                        members,
                        candidate,
                        contested: false,
                        contributes: true,
                    }
                })
                .collect();
            return Assessment {
                after: old.after,
                interpretations,
                groups,
                mechanism: "frozen provenance-group-004-v1; typed revisions ignored".into(),
            };
        }
        let mut interpretations = vec![];
        let mut roots: Vec<Option<usize>> = vec![];
        let mut seen = [false; 6];
        let mut children: Vec<Vec<usize>> = vec![vec![]; v.received.len()];
        for (index, m) in v.received.iter().enumerate() {
            let mut a = Interpretation {
                message: m.id,
                root: None,
                group: None,
                standing: Standing::Uncertainty,
                verification: Verification::NotUsed,
                checked: vec![],
                origin_verified: false,
            };
            let mut root = None;
            if m.speaker >= 6 || v.received[..index].iter().any(|p| p.id == m.id) {
                a.standing = Standing::InvalidRevision;
                interpretations.push(a);
                roots.push(root);
                continue;
            }
            match m.body {
                Body::Uncertain => {}
                Body::Assert(c) if c >= 6 => {
                    a.standing = Standing::UncertainProvenance;
                }
                Body::Assert(_) if seen[usize::from(m.speaker)] => {
                    a.standing = Standing::IgnoredRepeat
                }
                Body::Assert(_) => {
                    seen[usize::from(m.speaker)] = true;
                    root = Some(index);
                    a.root = Some(m.id);
                    let converted = originals.iter().find(|p| p.id == m.id).unwrap();
                    let (group, verification, checked) = e4::resolve_source(converted, &originals);
                    a.group = group;
                    a.verification = verification;
                    a.checked = checked;
                    a.standing = if group.is_some() {
                        Standing::Active
                    } else {
                        Standing::UncertainProvenance
                    };
                }
                Body::Revise {
                    target,
                    replacement,
                    forwarded,
                } => {
                    if target >= m.id || replacement.is_some_and(|c| c >= 6) {
                        a.standing = Standing::InvalidRevision;
                    } else if let Some(parent) =
                        v.received[..index].iter().position(|p| p.id == target)
                    {
                        a.checked.push(target);
                        let forward_ok=forwarded.is_none_or(|id|id<m.id && v.received[..index].iter().any(|p|p.id==id && matches!(p.body,Body::Revise {replacement:r,..} if r==replacement)));
                        if v.received[parent].speaker != m.speaker
                            || roots[parent].is_none()
                            || !forward_ok
                        {
                            a.standing = Standing::InvalidRevision;
                        } else {
                            let r = roots[parent].unwrap();
                            root = Some(r);
                            a.root = Some(v.received[r].id);
                            a.group = interpretations[r].group;
                            a.verification = interpretations[r].verification;
                            a.checked.extend(interpretations[r].checked.iter().copied());
                            if let Some(id) = forwarded {
                                a.checked.push(id);
                            }
                            children[parent].push(index);
                            a.standing = Standing::Active;
                        }
                    } else {
                        a.standing = Standing::MissingOriginal;
                    }
                }
            }
            interpretations.push(a);
            roots.push(root);
        }
        let mut group_members: BTreeMap<Group, Vec<(usize, Option<u8>, bool)>> = BTreeMap::new();
        for r in 0..v.received.len() {
            if roots[r] != Some(r) {
                continue;
            }
            let leaves: Vec<_> = (r..v.received.len())
                .filter(|&i| roots[i] == Some(r) && children[i].is_empty())
                .collect();
            let candidate = v.received[leaves[0]].body.candidate();
            let disputed = leaves
                .iter()
                .any(|&i| v.received[i].body.candidate() != candidate);
            for i in r..v.received.len() {
                if roots[i] != Some(r) {
                    continue;
                }
                interpretations[i].standing = if !children[i].is_empty() {
                    Standing::Superseded
                } else if disputed {
                    Standing::Disputed
                } else if interpretations[r].group.is_none() {
                    Standing::UncertainProvenance
                } else if candidate.is_none() {
                    Standing::Retracted
                } else {
                    Standing::Active
                };
                if v.received[i].speaker == v.id {
                    interpretations[i].standing = Standing::SelfIgnored;
                }
            }
            if let Some(group) = interpretations[r].group
                && v.received[r].speaker != v.id
                && (candidate.is_some() || disputed)
            {
                group_members.entry(group).or_default().push((
                    v.received[r].id,
                    candidate,
                    disputed,
                ));
            }
        }
        let mut scores = initial_belief(v.id, &own).scores;
        let groups = group_members
            .into_iter()
            .map(|(group, members)| {
                let candidate = members[0].1;
                let contested = members.iter().any(|m| m.2 || m.1 != candidate);
                let contributes =
                    !contested && candidate.is_some() && group != Group::Speaker(v.id);
                if contributes {
                    scores[usize::from(candidate.unwrap())] += 1;
                }
                GroupAssessment {
                    group,
                    members: members.iter().map(|m| m.0).collect(),
                    candidate: if contested { None } else { candidate },
                    contested,
                    contributes,
                }
            })
            .collect();
        Assessment {after:Belief::from_scores(scores),interpretations,groups,mechanism:"revision-listener-005-v1: explicit own branches; contested leaves/groups contribute zero".into()}
    }
}
