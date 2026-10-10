use serde::{Deserialize, Serialize};

pub mod experiment002;
pub mod experiment003;
pub mod experiment004;
pub mod experiment005;

pub const N: u8 = 6;
pub const RULES: &str = "incident-001-v1";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Incident,
    Observation,
    Discussion,
    Judgment,
    Terminated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Baseline {
    Silent,
    Truthful,
    Random,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub seed: u64,
    pub policies: [Baseline; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub id: usize,
    pub speaker: u8,
    pub candidate: u8,
    pub response_to: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentView {
    pub id: u8,
    pub roster: Vec<u8>,
    pub phase: Phase,
    pub opportunities_left: u8,
    pub observation: Option<u8>,
    pub belief: Option<u8>,
    pub claims: Vec<Claim>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Silence,
    Claim {
        candidate: u8,
        response_to: Option<usize>,
    },
    Vote(Option<u8>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    NoCommunication,
    ReportBelief,
    Unspecified,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub action: Action,
    pub intent: Intent,
}
pub trait Policy {
    fn choose(&mut self, view: &AgentView) -> Choice;
}

// Fixed integer arithmetic algorithm, independent of library RNG versions.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn draw(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
pub struct BaselinePolicy {
    kind: Baseline,
    rng: Rng,
    spoken: bool,
}
impl BaselinePolicy {
    pub fn new(kind: Baseline, seed: u64) -> Self {
        Self {
            kind,
            rng: Rng::new(seed),
            spoken: false,
        }
    }
}
impl Policy for BaselinePolicy {
    fn choose(&mut self, view: &AgentView) -> Choice {
        let mut intent = Intent::NoCommunication;
        let action = match view.phase {
            Phase::Discussion => match self.kind {
                Baseline::Truthful if !self.spoken && view.belief.is_some() => {
                    self.spoken = true;
                    intent = Intent::ReportBelief;
                    Action::Claim {
                        candidate: view.belief.unwrap(),
                        response_to: None,
                    }
                }
                Baseline::Random if self.rng.draw().is_multiple_of(2) => {
                    intent = Intent::Unspecified;
                    Action::Claim {
                        candidate: (self.rng.draw() % u64::from(N)) as u8,
                        response_to: view.claims.last().map(|c| c.id),
                    }
                }
                _ => Action::Silence,
            },
            Phase::Judgment => Action::Vote(match self.kind {
                Baseline::Truthful => view.belief,
                Baseline::Random => {
                    let n = self.rng.draw() % 7;
                    if n < 6 { Some(n as u8) } else { None }
                }
                Baseline::Silent => None,
            }),
            _ => Action::Silence,
        };
        Choice { action, intent }
    }
}
pub fn validate(view: &AgentView, action: &Action) -> Result<(), String> {
    if view.opportunities_left == 0 {
        return Err("budget exhausted".into());
    }
    match (view.phase, action) {
        (Phase::Discussion, Action::Silence) => Ok(()),
        (
            Phase::Discussion,
            Action::Claim {
                candidate,
                response_to,
            },
        ) => {
            if !view.roster.contains(candidate) {
                return Err("unknown candidate".into());
            }
            if response_to.is_some_and(|id| !view.claims.iter().any(|c| c.id == id)) {
                return Err("unknown public claim".into());
            }
            Ok(())
        }
        (Phase::Judgment, Action::Vote(candidate))
            if candidate.is_none_or(|id| view.roster.contains(&id)) =>
        {
            Ok(())
        }
        _ => Err("illegal phase or target".into()),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub input: AgentView,
    pub choice: Choice,
    pub rejection: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Transition(Phase),
    Incident {
        actor: u8,
    },
    ObservationAssigned {
        agent: u8,
        perceived: Option<u8>,
        mechanism: String,
    },
    Resolved {
        decision: usize,
        accepted: bool,
        claim_id: Option<usize>,
    },
    Outcome {
        tally: [u8; 6],
        verdict: Option<u8>,
        correct: Option<bool>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub rules_version: String,
    pub policy_version: String,
    pub rng_version: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub decisions: Vec<Decision>,
    pub public_claims: Vec<Claim>,
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
}
pub fn verdict(tally: [u8; 6]) -> Option<u8> {
    // A unique strict majority; ties cannot meet the four-vote threshold.
    if tally.iter().map(|&v| u16::from(v)).sum::<u16>() > 6 {
        return None;
    }
    tally.iter().position(|&v| v >= 4).map(|i| i as u8)
}
pub fn run(config: Config) -> Report {
    let mut policies: Vec<Box<dyn Policy>> = config
        .policies
        .iter()
        .enumerate()
        .map(|(id, &kind)| {
            Box::new(BaselinePolicy::new(
                kind,
                config.seed.wrapping_add((id as u64 + 1) * 1009),
            )) as Box<dyn Policy>
        })
        .collect();
    let mut report = run_with(config, &mut policies);
    report.policy_version = "baseline-v1".into();
    report
}
pub fn run_with(config: Config, policies: &mut [Box<dyn Policy>]) -> Report {
    assert_eq!(policies.len(), usize::from(N));
    let actor = (Rng::new(config.seed).draw() % u64::from(N)) as u8;
    let observations = [
        Some(actor),
        Some(actor),
        Some((actor + 1) % N),
        None,
        None,
        None,
    ];
    let mut report = Report {
        rules_version: RULES.into(),
        policy_version: "caller-supplied-unversioned".into(),
        rng_version: "splitmix64-v1".into(),
        config,
        objective_history: vec![
            Event::Transition(Phase::Incident),
            Event::Incident { actor },
            Event::Transition(Phase::Observation),
        ],
        decisions: vec![],
        public_claims: vec![],
        verdict: None,
        correct: None,
    };
    for (id, &perceived) in observations.iter().enumerate() {
        report.objective_history.push(Event::ObservationAssigned {
            agent: id as u8,
            perceived,
            mechanism: match id {
                0 | 1 => "accurate",
                2 => "next-id error",
                _ => "no identifying evidence",
            }
            .into(),
        });
    }
    let mut tally = [0; 6];
    for phase in [Phase::Discussion, Phase::Judgment] {
        report.objective_history.push(Event::Transition(phase));
        let rounds = if phase == Phase::Discussion { 2 } else { 1 };
        for round in 0..rounds {
            for (id, policy) in policies.iter_mut().enumerate() {
                let input = AgentView {
                    id: id as u8,
                    roster: (0..N).collect(),
                    phase,
                    opportunities_left: rounds - round,
                    observation: observations[id],
                    belief: observations[id],
                    claims: report.public_claims.clone(),
                };
                let choice = policy.choose(&input);
                let rejection = validate(&input, &choice.action).err();
                let mut claim_id = None;
                if rejection.is_none() {
                    match &choice.action {
                        Action::Claim {
                            candidate,
                            response_to,
                        } => {
                            let cid = report.public_claims.len();
                            claim_id = Some(cid);
                            report.public_claims.push(Claim {
                                id: cid,
                                speaker: id as u8,
                                candidate: *candidate,
                                response_to: *response_to,
                            });
                        }
                        Action::Vote(Some(candidate)) => tally[usize::from(*candidate)] += 1,
                        _ => {}
                    }
                }
                let accepted = rejection.is_none();
                let decision = report.decisions.len();
                report.decisions.push(Decision {
                    input,
                    choice,
                    rejection,
                });
                report.objective_history.push(Event::Resolved {
                    decision,
                    accepted,
                    claim_id,
                });
            }
        }
    }
    report.verdict = verdict(tally);
    report.correct = report.verdict.map(|id| id == actor);
    report
        .objective_history
        .push(Event::Transition(Phase::Terminated));
    report.objective_history.push(Event::Outcome {
        tally,
        verdict: report.verdict,
        correct: report.correct,
    });
    report
}
