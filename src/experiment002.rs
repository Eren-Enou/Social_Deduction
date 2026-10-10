//! Experiment 002 is isolated from the historical Experiment 001 resolver.
use crate::{N, Phase, Rng, verdict};
use serde::{Deserialize, Serialize};

pub const RULES: &str = "incident-002-v1";
pub const POLICIES: &str = "incentive-baselines-002-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyKind {
    Truthful,
    Silent,
    DeceptiveControl,
    IncentiveSensitive,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incentives {
    pub accuracy: u32,
    pub avoid_blame: u32,
    pub contradiction_cost: u32,
    pub speech_cost: u32,
}
impl Incentives {
    pub const ACCURACY: Self = Self {
        accuracy: 6,
        avoid_blame: 0,
        contradiction_cost: 1,
        speech_cost: 1,
    };
    pub const BLAME: Self = Self {
        accuracy: 1,
        avoid_blame: 12,
        contradiction_cost: 1,
        speech_cost: 1,
    };
    pub const QUIET: Self = Self {
        accuracy: 0,
        avoid_blame: 0,
        contradiction_cost: 2,
        speech_cost: 1,
    };
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub seed: u64,
    pub policies: [PolicyKind; 6],
    pub incentives: [Incentives; 6],
    pub participation_briefing: bool,
}
impl Config {
    pub fn new(seed: u64, kind: PolicyKind, incentives: Incentives) -> Self {
        Self {
            version: "config-002-v1".into(),
            seed,
            policies: [kind; 6],
            incentives: [incentives; 6],
            participation_briefing: true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Briefing {
    Participated,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndividualState {
    pub observation: Option<u8>,
    pub briefing: Option<Briefing>,
    pub belief: Option<u8>,
    pub incentives: Incentives,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Speech {
    Assert(u8),
    Uncertain,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: usize,
    pub speaker: u8,
    pub speech: Speech,
    pub response_to: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentView {
    pub id: u8,
    pub roster: Vec<u8>,
    pub phase: Phase,
    pub opportunities_left: u8,
    pub own: IndividualState,
    pub messages: Vec<Message>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Silence,
    Speak {
        speech: Speech,
        response_to: Option<usize>,
    },
    Vote(Option<u8>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    NoCommunication,
    ReportBelief,
    ContradictBelief,
    ExpressUncertainty,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionScore {
    pub action: Action,
    pub intent: Intent,
    pub score: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub action: Action,
    pub intent: Intent,
    pub scores: Vec<OptionScore>,
    pub mechanism: String,
}
pub trait Policy {
    fn choose(&mut self, view: &AgentView) -> Choice;
}
pub struct BaselinePolicy {
    kind: PolicyKind,
    spoken: bool,
}
impl BaselinePolicy {
    pub fn new(kind: PolicyKind) -> Self {
        Self {
            kind,
            spoken: false,
        }
    }
}
fn speak(speech: Speech) -> Action {
    Action::Speak {
        speech,
        response_to: None,
    }
}
pub fn options(view: &AgentView) -> Vec<OptionScore> {
    let i = view.own.incentives;
    let a = i64::from(i.accuracy);
    let b = i64::from(i.avoid_blame);
    let d = i64::from(i.contradiction_cost);
    let c = i64::from(i.speech_cost);
    let mut scores = vec![OptionScore {
        action: Action::Silence,
        intent: Intent::NoCommunication,
        score: 0,
    }];
    if let Some(belief) = view.own.belief {
        let self_attribution = if belief == view.id { b } else { 0 };
        scores.push(OptionScore {
            action: speak(Speech::Assert(belief)),
            intent: Intent::ReportBelief,
            score: a - c - self_attribution,
        });
        scores.push(OptionScore {
            action: speak(Speech::Assert((belief + 1) % N)),
            intent: Intent::ContradictBelief,
            score: -a - d - c + self_attribution,
        });
    } else {
        scores.push(OptionScore {
            action: speak(Speech::Uncertain),
            intent: Intent::ExpressUncertainty,
            score: a - c,
        });
    }
    scores
}
impl Policy for BaselinePolicy {
    fn choose(&mut self, view: &AgentView) -> Choice {
        let mut choice = Choice {
            action: Action::Silence,
            intent: Intent::NoCommunication,
            scores: vec![],
            mechanism: "no discussion opportunity".into(),
        };
        if view.phase == Phase::Judgment {
            choice.action = Action::Vote(if self.kind == PolicyKind::Silent {
                None
            } else {
                view.own.belief
            });
            choice.mechanism = "fixed sincere ballot; silent control abstains".into();
            return choice;
        }
        if view.phase != Phase::Discussion || view.opportunities_left == 0 {
            return choice;
        }
        if self.spoken {
            choice.mechanism = "one-message policy budget already used".into();
            return choice;
        }
        match self.kind {
            PolicyKind::Silent => {
                choice.mechanism = "silent control".into();
            }
            PolicyKind::Truthful => {
                choice.action = speak(
                    view.own
                        .belief
                        .map(Speech::Assert)
                        .unwrap_or(Speech::Uncertain),
                );
                choice.intent = if view.own.belief.is_some() {
                    Intent::ReportBelief
                } else {
                    Intent::ExpressUncertainty
                };
                choice.mechanism = "truthful control reports local belief or uncertainty".into();
            }
            PolicyKind::DeceptiveControl => {
                if let Some(belief) = view.own.belief {
                    choice.action = speak(Speech::Assert((belief + 1) % N));
                    choice.intent = Intent::ContradictBelief;
                }
                choice.mechanism = "explicit deceptive control; requires a belief".into();
            }
            PolicyKind::IncentiveSensitive => {
                choice.scores = options(view);
                let mut selected = &choice.scores[0];
                for option in &choice.scores[1..] {
                    if option.score > selected.score {
                        selected = option;
                    }
                }
                choice.action = selected.action.clone();
                choice.intent = selected.intent;
                choice.mechanism = "highest local signal utility; ties keep earlier option".into();
            }
        }
        if matches!(choice.action, Action::Speak { .. }) {
            self.spoken = true;
        }
        choice
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
            Action::Speak {
                speech,
                response_to,
            },
        ) => {
            if matches!(speech, Speech::Assert(id) if !view.roster.contains(id)) {
                return Err("unknown candidate".into());
            }
            if response_to.is_some_and(|id| !view.messages.iter().any(|m| m.id == id)) {
                return Err("unknown public message".into());
            }
            Ok(())
        }
        (Phase::Judgment, Action::Vote(id)) if id.is_none_or(|id| view.roster.contains(&id)) => {
            Ok(())
        }
        _ => Err("illegal phase or target".into()),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    SincereTruth,
    SincereMistake,
    IntentionalContradiction,
    Silence,
    Uncertainty,
    Ballot,
    Unsubstantiated,
    Rejected,
}
pub fn classify(view: &AgentView, choice: &Choice, actor: u8, accepted: bool) -> Classification {
    if !accepted {
        return Classification::Rejected;
    }
    match (&choice.action, choice.intent) {
        (Action::Silence, Intent::NoCommunication) => Classification::Silence,
        (Action::Vote(_), Intent::NoCommunication) => Classification::Ballot,
        (
            Action::Speak {
                speech: Speech::Uncertain,
                ..
            },
            Intent::ExpressUncertainty,
        ) if view.own.belief.is_none() => Classification::Uncertainty,
        (
            Action::Speak {
                speech: Speech::Assert(id),
                ..
            },
            Intent::ReportBelief,
        ) if view.own.belief == Some(*id) => {
            if *id == actor {
                Classification::SincereTruth
            } else {
                Classification::SincereMistake
            }
        }
        (
            Action::Speak {
                speech: Speech::Assert(id),
                ..
            },
            Intent::ContradictBelief,
        ) if view.own.belief.is_some_and(|b| b != *id) => Classification::IntentionalContradiction,
        _ => Classification::Unsubstantiated,
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub input: AgentView,
    pub choice: Choice,
    pub rejection: Option<String>,
    pub classification: Classification,
    pub assertion_matches_truth: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Transition(Phase),
    Incident {
        actor: u8,
    },
    Assigned {
        agent: u8,
        state: IndividualState,
        observation_mechanism: String,
    },
    Resolved {
        decision: usize,
        accepted: bool,
        message_id: Option<usize>,
        ballot: Option<u8>,
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
    pub public_messages: Vec<Message>,
    pub tally: [u8; 6],
    pub verdict: Option<u8>,
    pub correct: Option<bool>,
}
pub fn actor_for_seed(seed: u64) -> u8 {
    (Rng::new(seed).draw() % u64::from(N)) as u8
}
pub fn run(config: Config) -> Result<Report, String> {
    let mut policies: Vec<Box<dyn Policy>> = config
        .policies
        .iter()
        .map(|&kind| Box::new(BaselinePolicy::new(kind)) as Box<dyn Policy>)
        .collect();
    let mut report = run_with(config, &mut policies)?;
    report.policy_version = POLICIES.into();
    Ok(report)
}
pub fn run_with(config: Config, policies: &mut [Box<dyn Policy>]) -> Result<Report, String> {
    if config.version != "config-002-v1" {
        return Err("unsupported config version".into());
    }
    if policies.len() != usize::from(N) {
        return Err("exactly six policies required".into());
    }
    let actor = actor_for_seed(config.seed);
    let states: Vec<_> = (0..N)
        .map(|id| {
            let observation = match id {
                0 | 1 => Some(actor),
                2 => Some((actor + 1) % N),
                _ => None,
            };
            let briefing = if config.participation_briefing && id == actor {
                Some(Briefing::Participated)
            } else {
                None
            };
            IndividualState {
                observation,
                briefing,
                belief: if briefing.is_some() {
                    Some(id)
                } else {
                    observation
                },
                incentives: config.incentives[usize::from(id)],
            }
        })
        .collect();
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
        public_messages: vec![],
        tally: [0; 6],
        verdict: None,
        correct: None,
    };
    for (id, state) in states.iter().enumerate() {
        report.objective_history.push(Event::Assigned {
            agent: id as u8,
            state: state.clone(),
            observation_mechanism: match id {
                0 | 1 => "accurate",
                2 => "next-id error",
                _ => "no identifying evidence",
            }
            .into(),
        });
    }
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
                    own: states[id].clone(),
                    messages: report.public_messages.clone(),
                };
                let choice = policy.choose(&input);
                let rejection = validate(&input, &choice.action).err();
                let accepted = rejection.is_none();
                let classification = classify(&input, &choice, actor, accepted);
                let assertion_matches_truth = match &choice.action {
                    Action::Speak {
                        speech: Speech::Assert(candidate),
                        ..
                    } if accepted => Some(*candidate == actor),
                    _ => None,
                };
                let mut message_id = None;
                let mut ballot = None;
                if accepted {
                    match &choice.action {
                        Action::Speak {
                            speech,
                            response_to,
                        } => {
                            let mid = report.public_messages.len();
                            message_id = Some(mid);
                            report.public_messages.push(Message {
                                id: mid,
                                speaker: id as u8,
                                speech: speech.clone(),
                                response_to: *response_to,
                            });
                        }
                        Action::Vote(Some(candidate)) => {
                            report.tally[usize::from(*candidate)] += 1;
                            ballot = Some(*candidate);
                        }
                        _ => {}
                    }
                }
                let decision = report.decisions.len();
                report.decisions.push(Decision {
                    input,
                    choice,
                    rejection,
                    classification,
                    assertion_matches_truth,
                });
                report.objective_history.push(Event::Resolved {
                    decision,
                    accepted,
                    message_id,
                    ballot,
                });
            }
        }
    }
    report.verdict = verdict(report.tally);
    report.correct = report.verdict.map(|id| id == actor);
    report
        .objective_history
        .push(Event::Transition(Phase::Terminated));
    report.objective_history.push(Event::Outcome {
        tally: report.tally,
        verdict: report.verdict,
        correct: report.correct,
    });
    Ok(report)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fixture {
    Truthful,
    Silent,
    DeceptiveControl,
    Accuracy,
    Blame,
    Quiet,
    ActorOnlyBlame,
    NoBriefing,
}
pub const FIXTURES: [Fixture; 8] = [
    Fixture::Truthful,
    Fixture::Silent,
    Fixture::DeceptiveControl,
    Fixture::Accuracy,
    Fixture::Blame,
    Fixture::Quiet,
    Fixture::ActorOnlyBlame,
    Fixture::NoBriefing,
];
pub fn fixture_config(fixture: Fixture, seed: u64) -> Config {
    let kind = match fixture {
        Fixture::Truthful => PolicyKind::Truthful,
        Fixture::Silent => PolicyKind::Silent,
        Fixture::DeceptiveControl => PolicyKind::DeceptiveControl,
        _ => PolicyKind::IncentiveSensitive,
    };
    let incentives = match fixture {
        Fixture::Blame => Incentives::BLAME,
        Fixture::Quiet => Incentives::QUIET,
        _ => Incentives::ACCURACY,
    };
    let mut config = Config::new(seed, kind, incentives);
    if fixture == Fixture::ActorOnlyBlame {
        config.incentives[usize::from(actor_for_seed(seed))] = Incentives::BLAME;
    }
    if fixture == Fixture::NoBriefing {
        config.participation_briefing = false;
    }
    config
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub sincere_truth: usize,
    pub sincere_mistake: usize,
    pub intentional_contradiction: usize,
    pub silence: usize,
    pub uncertainty: usize,
    pub rejected: usize,
    pub unsubstantiated: usize,
    pub false_assertions: usize,
    pub verdicts: usize,
    pub correct_verdicts: usize,
}
impl Counts {
    fn add(&mut self, report: &Report) {
        for decision in &report.decisions {
            match decision.classification {
                Classification::SincereTruth => self.sincere_truth += 1,
                Classification::SincereMistake => self.sincere_mistake += 1,
                Classification::IntentionalContradiction => self.intentional_contradiction += 1,
                Classification::Silence => self.silence += 1,
                Classification::Uncertainty => self.uncertainty += 1,
                Classification::Rejected => self.rejected += 1,
                Classification::Unsubstantiated => self.unsubstantiated += 1,
                Classification::Ballot => {}
            }
            self.false_assertions += usize::from(decision.assertion_matches_truth == Some(false));
        }
        self.verdicts += usize::from(report.verdict.is_some());
        self.correct_verdicts += usize::from(report.correct == Some(true));
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureEvidence {
    pub fixture: Fixture,
    pub counts: Counts,
    pub reports: Vec<Report>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pair {
    pub seed: u64,
    pub comparison: Fixture,
    pub actor: u8,
    pub only_incentives_differ: bool,
    pub actor_before: Classification,
    pub actor_after: Classification,
    pub actor_choice_changed: bool,
    pub ballots_unchanged: bool,
    pub outcome_unchanged: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    pub version: String,
    pub runs: usize,
    pub replay_matches: usize,
    pub fixtures: Vec<FixtureEvidence>,
    pub pairs: Vec<Pair>,
}
pub fn evaluate() -> Result<Evaluation, String> {
    let mut evaluation = Evaluation {
        version: "evaluation-002-v1".into(),
        runs: 0,
        replay_matches: 0,
        fixtures: vec![],
        pairs: vec![],
    };
    for fixture in FIXTURES {
        let mut evidence = FixtureEvidence {
            fixture,
            counts: Counts::default(),
            reports: vec![],
        };
        for seed in 0..32 {
            let report = run(fixture_config(fixture, seed))?;
            let replay = run(fixture_config(fixture, seed))?;
            if serde_json::to_vec(&report).map_err(|e| e.to_string())?
                != serde_json::to_vec(&replay).map_err(|e| e.to_string())?
            {
                return Err("002 serialized replay mismatch".into());
            }
            evaluation.runs += 1;
            evaluation.replay_matches += 1;
            evidence.counts.add(&report);
            evidence.reports.push(report);
        }
        evaluation.fixtures.push(evidence);
    }
    let accuracy = &evaluation.fixtures[3];
    for comparison_index in [4, 5, 6] {
        let comparison = &evaluation.fixtures[comparison_index];
        for (before, after) in accuracy.reports.iter().zip(&comparison.reports) {
            let actor = actor_for_seed(before.config.seed);
            let mut matched_config = after.config.clone();
            matched_config.incentives = before.config.incentives;
            let matched_states =
                before.decisions[..6]
                    .iter()
                    .zip(&after.decisions[..6])
                    .all(|(b, a)| {
                        let mut state = a.input.own.clone();
                        state.incentives = b.input.own.incentives;
                        state == b.input.own
                    });
            evaluation.pairs.push(Pair {
                seed: before.config.seed,
                comparison: comparison.fixture,
                actor,
                only_incentives_differ: matched_config == before.config
                    && matched_states
                    && before.rules_version == after.rules_version
                    && before.policy_version == after.policy_version,
                actor_before: before.decisions[usize::from(actor)].classification,
                actor_after: after.decisions[usize::from(actor)].classification,
                actor_choice_changed: before.decisions[usize::from(actor)].choice.action
                    != after.decisions[usize::from(actor)].choice.action,
                ballots_unchanged: before.decisions[12..]
                    .iter()
                    .zip(&after.decisions[12..])
                    .all(|(b, a)| b.choice.action == a.choice.action),
                outcome_unchanged: before.tally == after.tally
                    && before.verdict == after.verdict
                    && before.correct == after.correct,
            });
        }
    }
    Ok(evaluation)
}
