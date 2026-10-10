use super::*;
use crate::{Phase, experiment004 as e4};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Body {
    Assert(u8),
    Uncertain,
    Revise {
        target: usize,
        replacement: Option<u8>,
        forwarded: Option<usize>,
    },
}
impl Body {
    pub fn candidate(&self) -> Option<u8> {
        match self {
            Self::Assert(c) => Some(*c),
            Self::Revise { replacement, .. } => *replacement,
            Self::Uncertain => None,
        }
    }
    pub fn target(&self) -> Option<usize> {
        if let Self::Revise { target, .. } = self {
            Some(*target)
        } else {
            None
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: usize,
    pub speaker: u8,
    pub body: Body,
    pub attribution: Attribution,
}
pub fn original_messages(messages: &[Message]) -> Vec<e4::Message> {
    messages
        .iter()
        .filter_map(|m| {
            Some(e4::Message {
                id: m.id,
                speaker: m.speaker,
                attribution: m.attribution,
                speech: match m.body {
                    Body::Assert(c) => Speech::Assert(c),
                    Body::Uncertain => Speech::Uncertain,
                    Body::Revise { .. } => return None,
                },
            })
        })
        .collect()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Basis {
    Private,
    Inference,
    ReceivedAssertion(usize),
    ReceivedRevision(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    Sincere,
    Contrary,
    Faithful,
    Withhold,
    NoCommunication,
    Unspecified,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Silence,
    Speak {
        body: Body,
        attribution: Attribution,
        basis: Basis,
        recipients: Vec<u8>,
    },
    Vote(Option<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub action: Action,
    pub intent: Intent,
    pub recognized_change: bool,
    pub eligible_forward: Option<usize>,
    pub revise_score: Option<i64>,
    pub silence_score: i64,
    pub mechanism: String,
}
impl Choice {
    pub fn silence(reason: &str) -> Self {
        Self {
            action: Action::Silence,
            intent: Intent::NoCommunication,
            recognized_change: false,
            eligible_forward: None,
            revise_score: None,
            silence_score: 0,
            mechanism: reason.into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub acquisition: usize,
    pub candidate: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentView {
    pub id: u8,
    pub roster: Vec<u8>,
    pub phase: Phase,
    pub opportunities_left: u8,
    pub original: IndividualState,
    pub observations: Vec<Observation>,
    pub current: Belief,
    pub received: Vec<Message>,
    pub sent: Vec<Message>,
    pub permitted_recipients: Vec<u8>,
    pub revision_benefit: u32,
    pub revision_cost: u32,
}
pub fn effective_original(
    original: &IndividualState,
    observations: &[Observation],
) -> IndividualState {
    let mut own = original.clone();
    if let Some(o) = observations.last() {
        own.observation = o.candidate;
    }
    own
}
pub trait CommunicationPolicy {
    fn choose(&mut self, view: &AgentView) -> Choice;
}
pub trait JudgmentPolicy {
    fn choose(&mut self, view: &AgentView) -> Choice;
}
impl JudgmentPolicy for JudgmentKind {
    fn choose(&mut self, v: &AgentView) -> Choice {
        let mut c = Choice::silence("belief-vote-003-v1");
        c.action = Action::Vote(match self {
            Self::Current => v.current.candidate,
            Self::Original => v.original.belief,
            Self::Abstain => None,
        });
        c
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpeakerKind {
    Retain(e4::SpeakerKind),
    Sincere,
    Selective,
    Misleading,
    RelayRevision { source: u8, benefit: u32, cost: u32 },
    LateTruth,
}
pub struct SpeakerPolicy {
    kind: SpeakerKind,
    baseline: Option<e4::SpeakerPolicy>,
}
impl SpeakerPolicy {
    pub fn new(kind: SpeakerKind) -> Self {
        Self {
            kind,
            baseline: if let SpeakerKind::Retain(k) = kind {
                Some(e4::SpeakerPolicy::new(k))
            } else {
                None
            },
        }
    }
}
fn local_basis(v: &AgentView, c: Option<u8>) -> Basis {
    let own = effective_original(&v.original, &v.observations);
    if c.is_some() && (own.observation == c || (own.briefing.is_some() && c == Some(v.id))) {
        Basis::Private
    } else {
        Basis::Inference
    }
}
impl CommunicationPolicy for SpeakerPolicy {
    fn choose(&mut self, v: &AgentView) -> Choice {
        if v.phase != Phase::Discussion || v.opportunities_left == 0 {
            return Choice::silence("no discussion opportunity");
        }
        if let Some(baseline) = &mut self.baseline {
            let view = e4::AgentView {
                id: v.id,
                roster: v.roster.clone(),
                phase: v.phase,
                opportunities_left: v.opportunities_left,
                original: effective_original(&v.original, &v.observations),
                current: v.current.clone(),
                messages: original_messages(&v.received),
                permitted_recipients: v.permitted_recipients.clone(),
            };
            let c = e4::CommunicationPolicy::choose(baseline, &view);
            let mut result = Choice::silence(&c.mechanism);
            result.recognized_change = v
                .sent
                .iter()
                .rev()
                .find(|m| m.body.candidate().is_some())
                .is_some_and(|m| m.body.candidate() != v.current.candidate);
            result.action = match c.action {
                e4::Action::Silence => Action::Silence,
                e4::Action::Vote(c) => Action::Vote(c),
                e4::Action::Speak {
                    speech,
                    attribution,
                    basis,
                    recipients,
                } => Action::Speak {
                    body: match speech {
                        Speech::Assert(c) => Body::Assert(c),
                        Speech::Uncertain => Body::Uncertain,
                    },
                    attribution,
                    basis: match basis {
                        e4::Basis::PrivateObservation => Basis::Private,
                        e4::Basis::Inference => Basis::Inference,
                        e4::Basis::Received(id) => Basis::ReceivedAssertion(id),
                    },
                    recipients,
                },
            };
            result.intent = match c.strategy {
                e4::Strategy::RelayReceived => Intent::Faithful,
                e4::Strategy::Baseline(crate::experiment002::Intent::ReportBelief) => {
                    Intent::Sincere
                }
                e4::Strategy::Baseline(crate::experiment002::Intent::ContradictBelief) => {
                    Intent::Contrary
                }
                _ => Intent::Unspecified,
            };
            if result.recognized_change && result.action == Action::Silence {
                result.intent = Intent::Withhold;
            }
            return result;
        }
        let prior = v
            .sent
            .iter()
            .rev()
            .find(|m| matches!(m.body, Body::Assert(_) | Body::Revise { .. }));
        let mut result = Choice::silence("correction-policy-005-v1: no supported revision");
        if let SpeakerKind::RelayRevision {
            source,
            benefit,
            cost,
        } = self.kind
        {
            if let Some(prior) = prior {
                let initial_parent = v.sent.iter().find_map(|m| {
                    if let Attribution::Reference(id) = m.attribution {
                        Some(id)
                    } else {
                        None
                    }
                });
                let revision = v.received.iter().find(|m| {
                    m.speaker == source
                        && matches!(m.body,Body::Revise {target,..} if Some(target)==initial_parent)
                });
                if let Some(revision) = revision {
                    result.eligible_forward = Some(revision.id);
                    result.recognized_change = revision.body.candidate() != prior.body.candidate();
                    result.revise_score = Some(i64::from(benefit) - i64::from(cost));
                    if result.recognized_change
                        && revision.body.candidate() == v.current.candidate
                        && benefit > cost
                    {
                        result.action = Action::Speak {
                            body: Body::Revise {
                                target: prior.id,
                                replacement: revision.body.candidate(),
                                forwarded: Some(revision.id),
                            },
                            attribution: revision.attribution,
                            basis: Basis::ReceivedRevision(revision.id),
                            recipients: v.permitted_recipients.clone(),
                        };
                        result.intent = Intent::Faithful;
                    } else {
                        result.intent = Intent::Withhold;
                    }
                }
            } else if let Some(parent) = v
                .received
                .iter()
                .find(|m| m.speaker == source && matches!(m.body, Body::Assert(_)))
                && parent.body.candidate() == v.current.candidate
            {
                result.action = Action::Speak {
                    body: parent.body.clone(),
                    attribution: Attribution::Reference(parent.id),
                    basis: Basis::ReceivedAssertion(parent.id),
                    recipients: v.permitted_recipients.clone(),
                };
                result.intent = Intent::Faithful;
            }
            return result;
        }
        if prior.is_none() {
            if self.kind == SpeakerKind::LateTruth && v.opportunities_left > 1 {
                return Choice::silence("declared late-testimony opportunity");
            }
            let candidate = v.current.candidate;
            result.action = Action::Speak {
                body: candidate.map(Body::Assert).unwrap_or(Body::Uncertain),
                attribution: if local_basis(v, candidate) == Basis::Private {
                    Attribution::Firsthand
                } else {
                    Attribution::Unspecified
                },
                basis: local_basis(v, candidate),
                recipients: v.permitted_recipients.clone(),
            };
            result.intent = Intent::Sincere;
            return result;
        }
        let prior = prior.unwrap();
        result.recognized_change = prior.body.candidate() != v.current.candidate;
        if self.kind == SpeakerKind::LateTruth {
            return result;
        }
        let replacement = if self.kind == SpeakerKind::Misleading {
            let Some(c) = v.current.candidate else {
                return result;
            };
            Some((c + 2) % 6)
        } else {
            v.current.candidate
        };
        if !result.recognized_change && self.kind != SpeakerKind::Misleading {
            return result;
        }
        result.revise_score = Some(i64::from(v.revision_benefit) - i64::from(v.revision_cost));
        if self.kind == SpeakerKind::Selective && v.revision_benefit <= v.revision_cost {
            result.intent = Intent::Withhold;
            return result;
        }
        result.action = Action::Speak {
            body: Body::Revise {
                target: prior.id,
                replacement,
                forwarded: None,
            },
            attribution: Attribution::Unspecified,
            basis: local_basis(v, replacement),
            recipients: v.permitted_recipients.clone(),
        };
        result.intent = if self.kind == SpeakerKind::Misleading {
            Intent::Contrary
        } else {
            Intent::Sincere
        };
        result
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaterObservation {
    pub before_slot: usize,
    pub agent: u8,
    pub candidate: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injection {
    pub before_slot: usize,
    pub speaker: u8,
    pub choice: Choice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub seed: u64,
    pub observations: [Option<u8>; 6],
    pub participation_briefing: bool,
    pub order: [u8; 6],
    pub speakers: [SpeakerKind; 6],
    pub listeners: [ListenerKind; 6],
    pub judgments: [JudgmentKind; 6],
    pub incentives: [Incentives; 6],
    pub revision_benefit: [u32; 6],
    pub revision_cost: [u32; 6],
    pub routes: [Vec<u8>; 6],
    pub second_routes: [Vec<u8>; 6],
    pub later: Vec<LaterObservation>,
    pub injections: Vec<Injection>,
}
impl Config {
    pub fn new(seed: u64) -> Self {
        Self {
            version: "config-005-v1".into(),
            seed,
            observations: [None; 6],
            participation_briefing: false,
            order: [0, 1, 2, 3, 4, 5],
            speakers: [SpeakerKind::Retain(e4::SpeakerKind::baseline(PolicyKind::Silent)); 6],
            listeners: [ListenerKind::Revision; 6],
            judgments: [JudgmentKind::Current; 6],
            incentives: [Incentives::ACCURACY; 6],
            revision_benefit: [2; 6],
            revision_cost: [1; 6],
            routes: std::array::from_fn(|_| (0..6).collect()),
            second_routes: std::array::from_fn(|_| (0..6).collect()),
            later: vec![],
            injections: vec![],
        }
    }
    pub fn actor(&self) -> u8 {
        crate::experiment002::actor_for_seed(self.seed)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "config-005-v1" || !e4::unique_ids(&self.order) {
            return Err("invalid version or order".into());
        }
        if self.observations.iter().any(|c| c.is_some_and(|c| c >= 6))
            || self
                .routes
                .iter()
                .chain(&self.second_routes)
                .any(|r| !e4::unique_ids(r))
        {
            return Err("invalid observation or routes".into());
        }
        if self.later.len() > 12
            || self
                .later
                .iter()
                .any(|o| o.agent >= 6 || o.before_slot >= 18 || o.candidate.is_some_and(|c| c >= 6))
        {
            return Err("invalid later observation".into());
        }
        if self.injections.len() > 12
            || self.injections.iter().any(|i| {
                i.speaker >= 6
                    || i.before_slot >= 12
                    || !matches!(i.choice.action, Action::Speak { .. })
            })
        {
            return Err("invalid injection".into());
        }
        for s in self.speakers {
            match s {
                SpeakerKind::Retain(e4::SpeakerKind::Baseline {
                    attribution: Some(Attribution::NamedSource(id)),
                    ..
                })
                | SpeakerKind::Retain(e4::SpeakerKind::Relay {
                    presentation: e4::Presentation::Named(id),
                    ..
                }) if id >= 6 => return Err("unknown claimed source".into()),
                SpeakerKind::RelayRevision { source, .. } if source >= 6 => {
                    return Err("unknown relay source".into());
                }
                SpeakerKind::Retain(e4::SpeakerKind::Relay { source, .. }) if source >= 6 => {
                    return Err("unknown relay source".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selective_ties_choose_silence_and_large_costs_do_not_overflow() {
        let r = run(fixture_config(
            Fixture::SelectiveLow,
            0,
            ListenerKind::Revision,
        ))
        .unwrap();
        let mut v = r.decisions[6].input.clone();
        v.revision_benefit = u32::MAX;
        v.revision_cost = u32::MAX;
        let c = SpeakerPolicy::new(SpeakerKind::Selective).choose(&v);
        assert!(c.recognized_change);
        assert_eq!(c.action, Action::Silence);
        assert_eq!(c.revise_score, Some(0));
        v.revision_benefit = 0;
        let c = SpeakerPolicy::new(SpeakerKind::Selective).choose(&v);
        assert_eq!(c.revise_score, Some(-i64::from(u32::MAX)));
    }
    #[test]
    fn withdrawing_an_observation_preserves_history_and_explicit_briefing() {
        let own = IndividualState {
            observation: Some(2),
            briefing: Some(Briefing::Participated),
            belief: Some(0),
            incentives: Incentives::ACCURACY,
        };
        let observations = vec![
            Observation {
                acquisition: 0,
                candidate: Some(2),
            },
            Observation {
                acquisition: 6,
                candidate: None,
            },
        ];
        let effective = effective_original(&own, &observations);
        assert_eq!(effective.observation, None);
        assert_eq!(initial_belief(0, &effective).candidate, Some(0));
        assert_eq!(own.observation, Some(2));
        assert_eq!(observations[0].candidate, Some(2));
    }
}
