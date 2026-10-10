use super::*;
use crate::{experiment004 as e4, experiment005 as e5};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackKind {
    Verification,
    ParticipantClaim(u8),
    CollectiveVerdict,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackPlan {
    pub after_episode: u8,
    pub about_episode: u8,
    pub recipients: Vec<u8>,
    pub kind: FeedbackKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub incidents: Vec<e5::Config>,
    pub listener: ListenerKind,
    pub updater: UpdateKind,
    pub feedback: Vec<FeedbackPlan>,
}
impl Config {
    pub fn new(incidents: Vec<e5::Config>) -> Self {
        Self {
            version: "config-006-v1".into(),
            incidents,
            listener: ListenerKind::Credibility,
            updater: UpdateKind::Evidence,
            feedback: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "config-006-v1" || self.incidents.is_empty() || self.incidents.len() > 8
        {
            return Err("invalid series version or incident bound".into());
        }
        for c in &self.incidents {
            c.validate()?;
            if c.listeners != [e5::ListenerKind::Revision; 6] {
                return Err("006 requires unchanged 005 revision standing".into());
            }
        }
        if self.feedback.len() > 24
            || self.feedback.iter().any(|f| {
                usize::from(f.after_episode) >= self.incidents.len()
                    || f.about_episode > f.after_episode
                    || !e4::unique_ids(&f.recipients)
                    || matches!(f.kind,FeedbackKind::ParticipantClaim(c) if c>=6)
            })
        {
            return Err("invalid scoped feedback plan".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub id: u8,
    pub memory_before: [LocalMemory; 6],
    pub incident: e5::Report,
    pub cognition: Vec<Cognition>,
    pub ingestions: Vec<Ingestion>,
    pub memory_after: [LocalMemory; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub plan: usize,
    pub delivered_after: u8,
    pub recipient: u8,
    pub input: FeedbackView,
    pub effect: FeedbackEffect,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Begin {
        episode: u8,
    },
    IncidentFinished {
        episode: u8,
    },
    Ingested {
        episode: u8,
        observer: u8,
        evicted: Vec<Reference>,
    },
    FeedbackDelivered {
        record: usize,
    },
    MemoryUpdated {
        record: usize,
    },
    End {
        episode: u8,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub rules_version: String,
    pub memory_version: String,
    pub verification_version: String,
    pub updater_version: String,
    pub listener_version: String,
    pub implementation: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub episodes: Vec<Episode>,
    pub feedback: Vec<FeedbackRecord>,
    pub final_memory: [LocalMemory; 6],
}
pub fn run(config: Config) -> Result<Report, String> {
    let mut updaters: Vec<Box<dyn CredibilityUpdater>> = (0..6)
        .map(|_| Box::new(config.updater) as Box<dyn CredibilityUpdater>)
        .collect();
    let mut r = run_with(config, &mut updaters)?;
    r.implementation = "builtin-006-v1".into();
    Ok(r)
}
pub fn run_with(
    config: Config,
    updaters: &mut [Box<dyn CredibilityUpdater>],
) -> Result<Report, String> {
    config.validate()?;
    if updaters.len() != 6 {
        return Err("six local updaters required".into());
    }
    let mut memories: [LocalMemory; 6] = std::array::from_fn(|id| LocalMemory::neutral(id as u8));
    let mut r = Report {
        rules_version: "multi-incident-006-v1 on incident-005-v1".into(),
        memory_version: "local-memory-006-v1".into(),
        verification_version: "lab-verification-006-v1 (trusted controller, no cryptography)"
            .into(),
        updater_version: "accuracy-update-006-v1".into(),
        listener_version: "credibility-listener-006-v1 / frozen revision-listener-005-v1".into(),
        implementation: "caller-supplied-unversioned".into(),
        config: config.clone(),
        objective_history: vec![],
        episodes: vec![],
        feedback: vec![],
        final_memory: memories.clone(),
    };
    for (index, c) in config.incidents.iter().enumerate() {
        let episode = index as u8;
        r.objective_history.push(Event::Begin { episode });
        let before = memories.clone();
        let log = Rc::new(RefCell::new(vec![]));
        let mut policies = e5::Policies::from_config(c);
        policies.listeners = memories
            .iter()
            .map(|m| {
                Box::new(super::cognition::Adapter {
                    episode,
                    kind: config.listener,
                    credit: m.credits,
                    log: log.clone(),
                }) as Box<dyn e5::Listener>
            })
            .collect();
        let mut incident = e5::run_with(c.clone(), &mut policies)?;
        incident.implementation="006-adapter: unchanged 005 resolver and built-in communication/judgment; local listener input audited separately".into();
        let cognition = log.borrow().clone();
        if cognition.len() != incident.updates.len()
            || cognition
                .iter()
                .zip(&incident.updates)
                .any(|(c, u)| c.agent != u.input.id || c.weighted != u.assessment.after)
        {
            return Err("cognition/resolver snapshot mismatch".into());
        }
        r.objective_history
            .push(Event::IncidentFinished { episode });
        let mut ingestions = vec![];
        for (id, memory) in memories.iter_mut().enumerate() {
            let judgment = incident
                .decisions
                .iter()
                .find(|d| d.input.id == id as u8 && d.input.phase == crate::Phase::Judgment)
                .ok_or("missing episode ballot")?;
            // Experience interpretation is frozen, unweighted and strictly local.
            let input = &incident.updates[judgment.update].input;
            let interpretation = e5::Listener::assess(&mut e5::ListenerKind::Revision, input);
            let ingestion = ingest(
                memory,
                IngestView {
                    episode,
                    observer: id as u8,
                    received: input.received.clone(),
                    interpretation,
                },
            )?;
            *memory = ingestion.after.clone();
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
            let about = &r.episodes[usize::from(f.about_episode)].incident;
            let received = match f.kind {
                FeedbackKind::Verification => Feedback::VerifiedOutcome {
                    episode: f.about_episode,
                    actor: about.config.actor(),
                    issuer: "lab-verification-006-v1".into(),
                },
                FeedbackKind::ParticipantClaim(c) => Feedback::Unverified {
                    episode: f.about_episode,
                    announcement: Announcement::ParticipantClaim(c),
                },
                FeedbackKind::CollectiveVerdict => Feedback::Unverified {
                    episode: f.about_episode,
                    announcement: Announcement::CollectiveVerdict(about.verdict),
                },
            };
            for &recipient in &f.recipients {
                let id = usize::from(recipient);
                let record = r.feedback.len();
                let input = FeedbackView {
                    delivery: record,
                    memory: memories[id].clone(),
                    received: received.clone(),
                };
                let effect = updaters[id].update(&input);
                effect.after.validate()?;
                if effect.after.observer != recipient {
                    return Err("updater changed observer identity".into());
                }
                r.objective_history
                    .push(Event::FeedbackDelivered { record });
                memories[id] = effect.after.clone();
                r.feedback.push(FeedbackRecord {
                    plan,
                    delivered_after: episode,
                    recipient,
                    input,
                    effect,
                });
                r.objective_history.push(Event::MemoryUpdated { record });
            }
        }
        r.episodes[index].memory_after = memories.clone();
        r.objective_history.push(Event::End { episode });
    }
    r.final_memory = memories;
    Ok(r)
}
