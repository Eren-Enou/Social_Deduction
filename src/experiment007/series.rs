use super::*;
use crate::{Phase, experiment004 as e4, experiment005 as e5, experiment006 as e6};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub after_episode: u8,
    pub actual_about: u8,
    pub source: String,
    pub fault: String,
    pub recipients: Vec<u8>,
    pub token: Token,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub session: u64,
    pub incidents: Vec<e5::Config>,
    pub acceptance: VerificationKind,
    pub listener: e6::ListenerKind,
    pub forecast_sources: Vec<u8>,
    pub forecast_observer: u8,
    pub feedback: Vec<Plan>,
}
impl Config {
    pub fn new(
        session: u64,
        incidents: Vec<e5::Config>,
        forecast_sources: Vec<u8>,
        forecast_observer: u8,
    ) -> Self {
        Self {
            version: "series-007-v1".into(),
            session,
            incidents,
            acceptance: VerificationKind::ConflictAware,
            listener: e6::ListenerKind::Credibility,
            forecast_sources,
            forecast_observer,
            feedback: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "series-007-v1"
            || self.incidents.is_empty()
            || self.incidents.len() > 8
            || self.forecast_observer >= 6
            || !e4::unique_ids(&self.forecast_sources)
            || self.feedback.len() > 32
        {
            return Err("invalid bounded007 config".into());
        }
        for incident in &self.incidents {
            incident.validate()?;
            if incident.listeners != [e5::ListenerKind::Revision; 6] {
                return Err("007 retains005 revision standing".into());
            }
        }
        if self.feedback.iter().any(|f| {
            usize::from(f.after_episode) >= self.incidents.len()
                || f.actual_about > f.after_episode
                || !e4::unique_ids(&f.recipients)
        }) {
            return Err("invalid007 controller delivery".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub id: u8,
    pub memory_before: [Memory; 6],
    pub incident: e5::Report,
    pub cognition: Vec<e6::Cognition>,
    pub ingestions: Vec<e6::Ingestion>,
    pub memory_after: [Memory; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackAudit {
    pub plan: usize,
    pub source: String,
    pub fault: String,
    pub after_episode: u8,
    pub actual_about: u8,
    pub actual_actor: u8,
    pub claimed_episode_actor: Option<u8>,
    pub factual_accuracy: Option<bool>,
    pub authentic: bool,
    pub recipient: u8,
    pub input: VerificationView,
    pub effect: VerificationEffect,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Begin(u8),
    IncidentFinished(u8),
    Ingested {
        episode: u8,
        observer: u8,
        evicted: Vec<e6::Reference>,
    },
    Delivered(usize),
    VerificationResolved(usize),
    End(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub version: String,
    pub implementation: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub episodes: Vec<Episode>,
    pub feedback: Vec<FeedbackAudit>,
    pub forecasts: Vec<Forecast>,
    pub final_memory: [Memory; 6],
}
struct Adapter {
    episode: u8,
    kind: Box<dyn e6::ExperienceListener>,
    credits: [e6::Credit; 6],
    log: Rc<RefCell<Vec<e6::Cognition>>>,
}
pub trait ListenerFactory {
    fn create(&mut self, observer: u8) -> Box<dyn e6::ExperienceListener>;
}
impl ListenerFactory for e6::ListenerKind {
    fn create(&mut self, _observer: u8) -> Box<dyn e6::ExperienceListener> {
        Box::new(*self)
    }
}
impl e5::Listener for Adapter {
    fn assess(&mut self, v: &e5::ListenerView) -> e5::Assessment {
        let (a, weights, unweighted) = self.kind.assess(e6::CredibilityView {
            incident: v,
            credits: &self.credits,
        });
        let mut log = self.log.borrow_mut();
        let update = log.len();
        log.push(e6::Cognition {
            episode: self.episode,
            agent: v.id,
            update,
            credit_input: self.credits,
            unweighted,
            weighted: a.after.clone(),
            weights,
        });
        a
    }
}
pub fn run(config: Config) -> Result<Report, String> {
    let mut policies: Vec<Box<dyn VerificationPolicy>> = (0..6)
        .map(|_| Box::new(config.acceptance) as Box<dyn VerificationPolicy>)
        .collect();
    let mut predictor = FrequencyPredictor;
    let mut r = run_with(config, &mut policies, &mut predictor)?;
    r.implementation = "builtin-007-v1".into();
    Ok(r)
}
pub fn run_with(
    config: Config,
    verification: &mut [Box<dyn VerificationPolicy>],
    predictor: &mut dyn Predictor,
) -> Result<Report, String> {
    let mut listeners = config.listener;
    run_with_components(config, verification, predictor, &mut listeners)
}
pub fn run_with_components(
    config: Config,
    verification: &mut [Box<dyn VerificationPolicy>],
    predictor: &mut dyn Predictor,
    listeners: &mut dyn ListenerFactory,
) -> Result<Report, String> {
    config.validate()?;
    if verification.len() != 6 {
        return Err("six local verification policies required".into());
    }
    let mut memories: [Memory; 6] = std::array::from_fn(|id| Memory::neutral(id as u8));
    let mut r = Report {
        version: "series-007-v1".into(),
        implementation: "caller-supplied-unversioned".into(),
        config: config.clone(),
        objective_history: vec![],
        episodes: vec![],
        feedback: vec![],
        forecasts: vec![],
        final_memory: memories.clone(),
    };
    for (index, c) in config.incidents.iter().enumerate() {
        let episode = index as u8;
        r.objective_history.push(Event::Begin(episode));
        let before = memories.clone();
        if index + 1 == config.incidents.len() {
            for &source in &config.forecast_sources {
                let input = PredictionInput::from_credit(
                    source,
                    memories[usize::from(config.forecast_observer)]
                        .local
                        .credits[usize::from(source)],
                );
                let forecast = predictor.predict(input);
                if forecast.input != input {
                    return Err("predictor changed recorded local input".into());
                }
                Probability::new(forecast.frequency.numerator, forecast.frequency.denominator)?;
                Probability::new(forecast.neutral.numerator, forecast.neutral.denominator)?;
                r.forecasts.push(forecast);
            }
        }
        let log = Rc::new(RefCell::new(vec![]));
        let mut policies = e5::Policies::from_config(c);
        policies.listeners = memories
            .iter()
            .map(|m| {
                Box::new(Adapter {
                    episode,
                    kind: listeners.create(m.local.observer),
                    credits: m.local.credits,
                    log: log.clone(),
                }) as Box<dyn e5::Listener>
            })
            .collect();
        let mut incident = e5::run_with(c.clone(), &mut policies)?;
        incident.implementation =
            "007 listener adapter; unchanged005 resolution and006 weights".into();
        let cognition = log.borrow().clone();
        if cognition.len() != incident.updates.len()
            || cognition
                .iter()
                .zip(&incident.updates)
                .any(|(a, b)| a.agent != b.input.id || a.weighted != b.assessment.after)
        {
            return Err("007 cognition snapshot mismatch".into());
        }
        r.objective_history.push(Event::IncidentFinished(episode));
        let mut ingestions = vec![];
        for (id, memory) in memories.iter_mut().enumerate() {
            let d = incident
                .decisions
                .iter()
                .find(|d| d.input.id == id as u8 && d.input.phase == Phase::Judgment)
                .ok_or("missing007 ballot")?;
            let view = &incident.updates[d.update].input;
            let interpretation = e5::Listener::assess(&mut e5::ListenerKind::Revision, view);
            let ingestion = memory.ingest(
                e6::IngestView {
                    episode,
                    observer: id as u8,
                    received: view.received.clone(),
                    interpretation,
                },
                config.acceptance,
            )?;
            r.objective_history.push(Event::Ingested {
                episode,
                observer: id as u8,
                evicted: ingestion.evicted.clone(),
            });
            ingestions.push(ingestion);
        }
        r.episodes.push(Episode {
            id: episode,
            memory_before: before,
            incident,
            cognition,
            ingestions,
            memory_after: memories.clone(),
        });
        for (plan, f) in config
            .feedback
            .iter()
            .enumerate()
            .filter(|(_, f)| f.after_episode == episode)
        {
            let actual_actor = config.incidents[usize::from(f.actual_about)].actor();
            let claimed_episode_actor = config
                .incidents
                .get(usize::from(f.token.episode))
                .filter(|_| f.token.episode <= episode)
                .map(|c| c.actor());
            let factual_accuracy = match f.token.kind {
                TokenKind::Outcome(actor) => claimed_episode_actor.map(|real| real == actor),
                TokenKind::Verdict(_) => None,
            };
            for &recipient in &f.recipients {
                let id = usize::from(recipient);
                let record = r.feedback.len();
                let input = VerificationView {
                    delivery: record,
                    completed_episode: episode,
                    session: config.session,
                    memory: memories[id].clone(),
                    received: f.token.clone(),
                };
                let effect = verification[id].accept(&input);
                effect.after.validate()?;
                if effect.after.local.observer != recipient {
                    return Err("verification changed observer identity".into());
                }
                r.objective_history.push(Event::Delivered(record));
                memories[id] = effect.after.clone();
                r.feedback.push(FeedbackAudit {
                    plan,
                    source: f.source.clone(),
                    fault: f.fault.clone(),
                    after_episode: episode,
                    actual_about: f.actual_about,
                    actual_actor,
                    claimed_episode_actor,
                    factual_accuracy,
                    authentic: f.token.authentic(),
                    recipient,
                    input,
                    effect,
                });
                r.objective_history
                    .push(Event::VerificationResolved(record));
            }
        }
        r.episodes[index].memory_after = memories.clone();
        r.objective_history.push(Event::End(episode));
    }
    r.final_memory = memories;
    Ok(r)
}
