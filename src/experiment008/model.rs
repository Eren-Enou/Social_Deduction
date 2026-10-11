use crate::{experiment005 as e5, experiment006 as e6};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    FirstDiscussion,
    Investigation,
    SecondDiscussion,
    Judgment,
    Complete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Root {
    Artifact(u8),
    Observation(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Descriptor {
    pub id: u8,
    pub description: String,
    pub visible_parent: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub descriptor: Descriptor,
    pub inspectors: Vec<u8>,
    pub candidate: Option<u8>,
    pub actual_root: Root,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Acquisition {
    pub episode: u8,
    pub observer: u8,
    pub local: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub source: Descriptor,
    pub candidate: Option<u8>,
    pub acquisition: Acquisition,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvestigationAction {
    Decline,
    Inspect(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationView {
    pub id: u8,
    pub episode: u8,
    pub episodes_remaining: u8,
    pub belief: e5::Belief,
    pub unweighted: e5::Belief,
    pub received: Vec<e5::Message>,
    pub credits: [e6::Credit; 6],
    pub catalog: Vec<Descriptor>,
    pub balance: u8,
    pub stage: Stage,
    pub used: bool,
    pub action_cost: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationChoice {
    pub action: InvestigationAction,
    pub usefulness: u8,
    pub cost: u8,
    pub credibility_warning: bool,
    pub mechanism: String,
}
pub trait InvestigationPolicy {
    fn choose(&mut self, view: &InvestigationView) -> InvestigationChoice;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyKind {
    NeverInvestigate,
    AlwaysInvestigate,
    ConflictDriven,
    CostSensitive,
    CredibilityAware,
    SeededRandom,
}
pub const POLICIES: [PolicyKind; 6] = [
    PolicyKind::NeverInvestigate,
    PolicyKind::AlwaysInvestigate,
    PolicyKind::ConflictDriven,
    PolicyKind::CostSensitive,
    PolicyKind::CredibilityAware,
    PolicyKind::SeededRandom,
];
pub struct Policy {
    pub kind: PolicyKind,
    pub rng: crate::Rng,
}
impl Policy {
    pub fn new(kind: PolicyKind, seed: u64, id: u8) -> Self {
        Self {
            kind,
            rng: crate::Rng::new(seed ^ (0x8008 + u64::from(id))),
        }
    }
}
impl InvestigationPolicy for Policy {
    fn choose(&mut self, v: &InvestigationView) -> InvestigationChoice {
        let competing = v.belief.scores.iter().filter(|s| **s > 0).count() > 1;
        let unknown = v.belief.candidate.is_none();
        let usefulness = if unknown {
            3
        } else if competing || v.belief.margin <= 1 {
            2
        } else {
            0
        };
        let warning = v.received.iter().any(|m| {
            m.speaker < 6
                && matches!(m.body, e5::Body::Assert(_))
                && v.credits[usize::from(m.speaker)].score < 0
        }) || v.belief.candidate != v.unweighted.candidate;
        let usefulness = usefulness
            + if self.kind == PolicyKind::CredibilityAware && warning {
                2
            } else {
                0
            };
        let cost = v.action_cost + u8::from(v.balance == 1);
        let inspect = match self.kind {
            PolicyKind::NeverInvestigate => false,
            PolicyKind::AlwaysInvestigate => true,
            PolicyKind::ConflictDriven => unknown || competing,
            PolicyKind::CostSensitive | PolicyKind::CredibilityAware => usefulness > cost,
            PolicyKind::SeededRandom => self.rng.draw() % 2 == 1,
        };
        let action = if inspect
            && v.balance > 0
            && v.stage == Stage::Investigation
            && !v.used
            && !v.catalog.is_empty()
        {
            let index = if self.kind == PolicyKind::SeededRandom {
                (self.rng.draw() % v.catalog.len() as u64) as usize
            } else {
                0
            };
            InvestigationAction::Inspect(v.catalog[index].id)
        } else {
            InvestigationAction::Decline
        };
        InvestigationChoice {
            action,
            usefulness,
            cost,
            credibility_warning: warning,
            mechanism: format!(
                "investigation-policy-008-v1::{:?}; strict usefulness/cost gate where applicable",
                self.kind
            ),
        }
    }
}
pub fn validate_investigation(
    v: &InvestigationView,
    action: InvestigationAction,
) -> Result<(), String> {
    if v.id >= 6 || v.stage != Stage::Investigation || v.used {
        return Err("investigation opportunity unavailable or already consumed".into());
    }
    if let InvestigationAction::Inspect(id) = action {
        if v.balance == 0 {
            return Err("effort exhausted".into());
        }
        if !v.catalog.iter().any(|s| s.id == id) {
            return Err("source not available".into());
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationKind {
    Sincere,
    ShareFinding,
    Silent,
    RelayFinding,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportBasis {
    Acquired(Acquisition),
    Received(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportedFinding {
    pub message: usize,
    pub finding: Finding,
    pub basis: ReportBasis,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationView {
    pub incident: e5::AgentView,
    pub findings: Vec<Finding>,
    pub reports: Vec<ReportedFinding>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationChoice {
    pub choice: e5::Choice,
    pub report: Option<(Finding, ReportBasis)>,
}
pub trait CommunicationPolicy {
    fn choose(&mut self, v: &CommunicationView) -> CommunicationChoice;
}
pub struct Speaker {
    pub kind: CommunicationKind,
    pub baseline: e5::SpeakerPolicy,
}
impl Speaker {
    pub fn new(kind: CommunicationKind) -> Self {
        Self {
            kind,
            baseline: e5::SpeakerPolicy::new(e5::SpeakerKind::Sincere),
        }
    }
}
impl CommunicationPolicy for Speaker {
    fn choose(&mut self, v: &CommunicationView) -> CommunicationChoice {
        let mut result = CommunicationChoice {
            choice: e5::Choice::silence("no finding disclosure chosen"),
            report: None,
        };
        if self.kind == CommunicationKind::Silent {
            return result;
        }
        if self.kind == CommunicationKind::Sincere || v.incident.opportunities_left == 2 {
            if self.kind != CommunicationKind::RelayFinding {
                result.choice = e5::CommunicationPolicy::choose(&mut self.baseline, &v.incident);
            }
            return result;
        }
        let selected = if self.kind == CommunicationKind::ShareFinding {
            v.findings.first().cloned().map(|f| {
                let basis = ReportBasis::Acquired(f.acquisition);
                (f, basis)
            })
        } else {
            v.reports
                .iter()
                .find(|r| {
                    r.finding.candidate.is_some()
                        && r.finding.candidate == v.incident.current.candidate
                })
                .map(|r| (r.finding.clone(), ReportBasis::Received(r.message)))
        };
        if let Some((finding, basis)) = selected {
            let target = v
                .incident
                .sent
                .iter()
                .rev()
                .find(|m| matches!(m.body, e5::Body::Assert(_) | e5::Body::Revise { .. }))
                .map(|m| m.id);
            let body = match (finding.candidate, target) {
                (None, _) => e5::Body::Uncertain,
                (Some(candidate), Some(target)) => e5::Body::Revise {
                    target,
                    replacement: Some(candidate),
                    forwarded: None,
                },
                (Some(candidate), None) => e5::Body::Assert(candidate),
            };
            result.choice.action = e5::Action::Speak {
                body,
                attribution: match basis {
                    ReportBasis::Received(id) => e5::Attribution::Reference(id),
                    _ => e5::Attribution::Firsthand,
                },
                basis: e5::Basis::Inference,
                recipients: v.incident.permitted_recipients.clone(),
            };
            result.choice.intent = e5::Intent::Faithful;
            result.choice.mechanism =
                "voluntary exact acquired/received finding report; accuracy unverified".into();
            result.report = Some((finding, basis));
        }
        result
    }
}
pub fn validate_communication(
    v: &CommunicationView,
    c: &CommunicationChoice,
) -> Result<(), String> {
    e5::validate_action(&v.incident, &c.choice.action)?;
    if let Some((f, b)) = &c.report {
        let e5::Action::Speak { body, .. } = &c.choice.action else {
            return Err("finding payload requires speech".into());
        };
        if body.candidate() != f.candidate {
            return Err("finding content mismatch".into());
        }
        let accessible = match b {
            ReportBasis::Acquired(a) => v
                .findings
                .iter()
                .any(|own| own == f && own.acquisition == *a && a.observer == v.incident.id),
            ReportBasis::Received(id) => {
                v.reports
                    .iter()
                    .any(|r| r.message == *id && r.finding == *f)
                    && v.incident.received.iter().any(|m| m.id == *id)
            }
        };
        if !accessible {
            return Err("finding not legitimately acquired or received".into());
        }
    }
    Ok(())
}
