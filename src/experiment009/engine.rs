use super::*;
use crate::{
    Phase, experiment004 as e4, experiment005 as e5, experiment006 as e6, experiment007 as e7,
};
use e7::VerificationPolicy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incident {
    pub game: e5::Config,
    pub artifacts: Vec<Source>,
    pub forced: Vec<(u8, InvestigationAction)>,
    pub access_order: [u8; 6],
    pub communication: [CommunicationKind; 6],
    pub injected: Vec<(u8, u8)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub session: u64,
    pub incidents: Vec<Incident>,
    pub policy: [PolicyKind; 6],
    pub budgets: [u8; 6],
    pub listener: e6::ListenerKind,
    pub feedback: bool,
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "series-009-v1"
            || self.incidents.is_empty()
            || self.incidents.len() > 4
            || self.budgets.iter().any(|b| *b > 9)
        {
            return Err("invalid bounded009 series".into());
        }
        for c in &self.incidents {
            c.game.validate()?;
            if !c.game.later.is_empty()
                || !c.game.injections.is_empty()
                || c.game.listeners != [e5::ListenerKind::Revision; 6]
                || !e4::unique_ids(&c.access_order)
                || c.artifacts.len() > 4
                || c.injected.len() > 6
                || c.forced.len() > 6
            {
                return Err("invalid009 incident schedule".into());
            }
            let mut ids = std::collections::BTreeSet::new();
            let mut keys = std::collections::BTreeSet::new();
            for a in &c.artifacts {
                a.public().validate()?;
                if !keys.insert(a.method_key.clone()) {
                    return Err("duplicate semantic source key".into());
                }
                if !ids.insert(a.descriptor.id)
                    || !e4::unique_ids(&a.inspectors)
                    || a.candidate.is_some_and(|x| x >= 6)
                {
                    return Err("invalid artifact identity/access/content".into());
                }
                if a.separate_acquisition_disclosed && !matches!(a.actual_root, Root::Artifact(_)) {
                    return Err("separate acquisition disclosure inconsistent with world".into());
                }
                match a.actual_root {
                    Root::Artifact(id)
                        if id == a.descriptor.id && a.descriptor.visible_parent.is_none() => {}
                    Root::Observation(id)
                        if id < 6
                            && c.game.observations[usize::from(id)] == a.candidate
                            && a.descriptor.visible_parent.is_none_or(|p| p == id) => {}
                    _ => {
                        return Err(
                            "artifact dependency/content inconsistent at initialization".into()
                        );
                    }
                }
            }
            let mut forced_agents = std::collections::BTreeSet::new();
            for &(observer, action) in &c.forced {
                if observer >= 6 || !forced_agents.insert(observer) {
                    return Err("invalid/duplicate forced observer".into());
                }
                if let InvestigationAction::Inspect(source) = action
                    && !c
                        .artifacts
                        .iter()
                        .any(|a| a.descriptor.id == source && a.inspectors.contains(&observer))
                {
                    return Err("invalid forced source".into());
                }
            }
            let mut recipients = std::collections::BTreeSet::new();
            for &(recipient, source) in &c.injected {
                if !recipients.insert(recipient)
                    || !c
                        .artifacts
                        .iter()
                        .any(|a| a.descriptor.id == source && a.inspectors.contains(&recipient))
                {
                    return Err("invalid/duplicate controller finding".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub initial: u8,
    pub remaining: u8,
    pub spent: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Investigation {
    pub assessment_input: EvidenceView,
    pub assessment: Assessment,
    pub input: InvestigationView,
    pub choice: InvestigationChoice,
    pub rejection: Option<String>,
    pub delivery: Option<usize>,
    pub forced: bool,
    pub proposal: Option<InvestigationChoice>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryOrigin {
    Selected(usize),
    Forced(usize),
    Intervention(usize),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    pub origin: DeliveryOrigin,
    pub finding: Finding,
    pub actual_root: Root,
    pub correct: Option<bool>,
    pub before: EvidenceView,
    pub before_assessment: Assessment,
    pub after: EvidenceView,
    pub after_assessment: Assessment,
    pub balance_before: u8,
    pub balance_after: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cognition {
    pub input: EvidenceView,
    pub assessment: Assessment,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Communication {
    pub input: CommunicationView,
    pub choice: CommunicationChoice,
    pub decision: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ancestry {
    pub message: usize,
    pub actual_roots: Vec<Root>,
    pub report_basis: Option<ReportBasis>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Begin(u8),
    Stage {
        episode: u8,
        stage: Stage,
    },
    Investigation {
        episode: u8,
        decision: usize,
    },
    Finding {
        episode: u8,
        delivery: usize,
    },
    Ingest {
        episode: u8,
        observer: u8,
        evicted: Vec<e6::Reference>,
    },
    Verified {
        episode: u8,
        observer: u8,
    },
    End(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub id: u8,
    pub incident: e5::Report,
    pub cognition: Vec<Cognition>,
    pub investigation: Vec<Investigation>,
    pub deliveries: Vec<Delivery>,
    pub communication: Vec<Communication>,
    pub reports: Vec<ReportedFinding>,
    pub ancestry: Vec<Ancestry>,
    pub memory_before: [e7::Memory; 6],
    pub ingestions: Vec<e6::Ingestion>,
    pub verification: Vec<(e7::VerificationView, e7::VerificationEffect)>,
    pub memory_after: [e7::Memory; 6],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub version: String,
    pub implementation: String,
    pub config: Config,
    pub objective_history: Vec<Event>,
    pub episodes: Vec<Episode>,
    pub ledgers: [Ledger; 6],
    pub final_memory: [e7::Memory; 6],
}
pub struct Components {
    pub investigators: Vec<Box<dyn InvestigationPolicy>>,
    pub speakers: Vec<Box<dyn CommunicationPolicy>>,
    pub listeners: Vec<Box<dyn EvidenceListener>>,
}
impl Components {
    pub fn from_config(c: &Config) -> Self {
        Self {
            investigators: (0..6)
                .map(|id| {
                    Box::new(Policy::new(c.policy[id], c.session, id as u8))
                        as Box<dyn InvestigationPolicy>
                })
                .collect(),
            speakers: (0..6)
                .map(|id| {
                    Box::new(Speaker::new(c.incidents[0].communication[id]))
                        as Box<dyn CommunicationPolicy>
                })
                .collect(),
            listeners: (0..6)
                .map(|_| {
                    Box::new(EvidenceAware {
                        testimony: c.listener,
                    }) as Box<dyn EvidenceListener>
                })
                .collect(),
        }
    }
}
fn assess(p: &mut dyn EvidenceListener, v: &EvidenceView) -> Result<Assessment, String> {
    let a = p.assess(v);
    if e5::Belief::from_scores(a.after.scores) != a.after
        || a.base.after != a.after
        || a.base.interpretations.len() != v.incident.received.len()
        || a.base.interpretations.iter().any(|i| {
            !v.incident.received.iter().any(|m| m.id == i.message)
                || i.checked
                    .iter()
                    .any(|id| !v.incident.received.iter().any(|m| m.id == *id))
        })
    {
        return Err("inconsistent/inaccessible009 listener assessment".into());
    }
    Ok(a)
}
struct Local {
    originals: Vec<e5::IndividualState>,
    observations: Vec<Vec<e5::Observation>>,
    current: Vec<e5::Belief>,
    received: Vec<Vec<e5::Message>>,
    sent: Vec<Vec<e5::Message>>,
    findings: Vec<Vec<Finding>>,
    reports: Vec<Vec<ReportedFinding>>,
}
impl Local {
    fn evidence(&self, id: usize, credits: [e6::Credit; 6]) -> EvidenceView {
        EvidenceView {
            incident: e5::ListenerView {
                id: id as u8,
                original: self.originals[id].clone(),
                observations: self.observations[id].clone(),
                prior: self.current[id].clone(),
                received: self.received[id].clone(),
            },
            credits,
            findings: self.findings[id].clone(),
            reports: self.reports[id].clone(),
        }
    }
    fn communication(&self, c: &Incident, id: usize, slot: usize) -> CommunicationView {
        CommunicationView {
            incident: e5::AgentView {
                id: id as u8,
                roster: (0..6).collect(),
                phase: if slot < 12 {
                    Phase::Discussion
                } else {
                    Phase::Judgment
                },
                opportunities_left: if slot < 6 { 2 } else { 1 },
                original: self.originals[id].clone(),
                observations: self.observations[id].clone(),
                current: self.current[id].clone(),
                received: self.received[id].clone(),
                sent: self.sent[id].clone(),
                permitted_recipients: if slot < 6 {
                    c.game.routes[id].clone()
                } else {
                    c.game.second_routes[id].clone()
                },
                revision_benefit: c.game.revision_benefit[id],
                revision_cost: c.game.revision_cost[id],
            },
            findings: self.findings[id].clone(),
            reports: self.reports[id].clone(),
        }
    }
}
fn delivery(
    ep: &mut Episode,
    l: &mut Local,
    artifact: &Source,
    observer: u8,
    origin: DeliveryOrigin,
    ledger: &mut Ledger,
    p: &mut dyn EvidenceListener,
) -> Result<usize, String> {
    let id = usize::from(observer);
    let before = l.evidence(id, ep.memory_before[id].local.credits);
    let before_assessment = assess(p, &before)?;
    let balance_before = ledger.remaining;
    if !matches!(origin, DeliveryOrigin::Intervention(_)) {
        if ledger.remaining < artifact.cost {
            return Err("resolver effort insufficient".into());
        }
        ledger.remaining = ledger
            .remaining
            .checked_sub(artifact.cost)
            .ok_or("resource underflow")?;
        ledger.spent = ledger
            .spent
            .checked_add(artifact.cost)
            .ok_or("resource overflow")?;
    }
    let finding = Finding {
        source: artifact.descriptor.clone(),
        candidate: artifact.candidate,
        acquisition: Acquisition {
            episode: ep.id,
            observer,
            local: l.findings[id].len(),
        },
    };
    l.findings[id].push(finding.clone());
    let after = l.evidence(id, ep.memory_before[id].local.credits);
    let after_assessment = assess(p, &after)?;
    l.current[id] = after_assessment.after.clone();
    let index = ep.deliveries.len();
    ep.deliveries.push(Delivery {
        origin,
        finding,
        actual_root: artifact.actual_root,
        correct: artifact.candidate.map(|c| c == ep.incident.config.actor()),
        before,
        before_assessment,
        after,
        after_assessment,
        balance_before,
        balance_after: ledger.remaining,
    });
    Ok(index)
}
pub fn run(config: Config) -> Result<Report, String> {
    config.validate()?;
    let mut p = Components::from_config(&config);
    let mut r = run_inner(config, &mut p, true)?;
    r.implementation = "builtin-009-v1".into();
    Ok(r)
}
pub fn run_with(config: Config, p: &mut Components) -> Result<Report, String> {
    run_inner(config, p, false)
}
fn run_inner(config: Config, p: &mut Components, builtin: bool) -> Result<Report, String> {
    config.validate()?;
    if [p.investigators.len(), p.speakers.len(), p.listeners.len()]
        .iter()
        .any(|n| *n != 6)
    {
        return Err("six009 policies per mechanism required".into());
    }
    let mut memories: [e7::Memory; 6] = std::array::from_fn(|id| e7::Memory::neutral(id as u8));
    let mut r = Report {
        version: "series-009-v1".into(),
        implementation: "caller-supplied-unversioned".into(),
        config: config.clone(),
        objective_history: vec![],
        episodes: vec![],
        ledgers: std::array::from_fn(|id| Ledger {
            initial: config.budgets[id],
            remaining: config.budgets[id],
            spent: 0,
        }),
        final_memory: memories.clone(),
    };
    for (episode, c) in config.incidents.iter().enumerate() {
        let episode = episode as u8;
        if builtin {
            p.speakers = c
                .communication
                .iter()
                .map(|kind| Box::new(Speaker::new(*kind)) as Box<dyn CommunicationPolicy>)
                .collect();
        }
        r.objective_history.push(Event::Begin(episode));
        let originals: Vec<_> = (0..6)
            .map(|id| e5::IndividualState {
                observation: c.game.observations[id],
                briefing: if c.game.participation_briefing && id as u8 == c.game.actor() {
                    Some(e5::Briefing::Participated)
                } else {
                    None
                },
                belief: if c.game.participation_briefing && id as u8 == c.game.actor() {
                    Some(id as u8)
                } else {
                    c.game.observations[id]
                },
                incentives: c.game.incentives[id],
            })
            .collect();
        let observations: Vec<_> = (0..6)
            .map(|id| {
                vec![e5::Observation {
                    acquisition: id,
                    candidate: c.game.observations[id],
                }]
            })
            .collect();
        let current = originals
            .iter()
            .enumerate()
            .map(|(id, o)| e5::initial_belief(id as u8, o))
            .collect();
        let mut l = Local {
            originals,
            observations,
            current,
            received: vec![vec![]; 6],
            sent: vec![vec![]; 6],
            findings: vec![vec![]; 6],
            reports: vec![vec![]; 6],
        };
        let incident = e5::Report {
            rules_version: "incident-009-v1".into(),
            revision_version: "revision-005-v1".into(),
            listener_version: "evidence-listener-008-v1 + experience-listener-006-v1".into(),
            policy_version: "communication-008-v1".into(),
            implementation: "009 scheduler / unchanged005 legal communication".into(),
            config: c.game.clone(),
            objective_history: vec![
                e5::Event::Transition(Phase::Incident),
                e5::Event::Incident {
                    actor: c.game.actor(),
                },
                e5::Event::Transition(Phase::Observation),
            ],
            updates: vec![],
            decisions: vec![],
            testimony: vec![],
            injection_inputs: vec![],
            tally: [0; 6],
            verdict: None,
            correct: None,
            retained_received_copies: 0,
            retained_sent_copies: 0,
        };
        let mut ep = Episode {
            id: episode,
            incident,
            cognition: vec![],
            investigation: vec![],
            deliveries: vec![],
            communication: vec![],
            reports: vec![],
            ancestry: vec![],
            memory_before: memories.clone(),
            ingestions: vec![],
            verification: vec![],
            memory_after: memories.clone(),
        };
        for id in 0..6 {
            ep.incident.objective_history.push(e5::Event::Assigned {
                agent: id as u8,
                original: l.originals[id].clone(),
                observation: l.observations[id][0].clone(),
            });
        }
        for slot in 0..18 {
            if slot == 6 {
                r.objective_history.push(Event::Stage {
                    episode,
                    stage: Stage::Investigation,
                });
                // All policies select from pre-delivery local snapshots. Results cannot
                // influence another individual's choice before discussion resumes.
                for &agent in &c.access_order {
                    let id = usize::from(agent);
                    let ev = l.evidence(id, memories[id].local.credits);
                    let a = assess(&mut *p.listeners[id], &ev)?;
                    l.current[id] = a.after.clone();
                    let catalog: Vec<_> = c
                        .artifacts
                        .iter()
                        .filter(|source| source.inspectors.contains(&agent))
                        .map(Source::public)
                        .collect();
                    let input = InvestigationView {
                        id: agent,
                        episode,
                        episodes_remaining: (config.incidents.len() - usize::from(episode) - 1)
                            as u8,
                        evidence: ev.clone(),
                        belief: a.after.clone(),
                        testimony: config.listener,
                        catalog,
                        balance: r.ledgers[id].remaining,
                        stage: Stage::Investigation,
                        used: false,
                    };
                    let mut choice = p.investigators[id].choose(&input);
                    let proposal = choice.clone();
                    let forced = c.forced.iter().find(|(id, _)| *id == agent);
                    if let Some((_, action)) = forced {
                        choice.action = *action;
                        choice.mechanism =
                            "research-only forced choice; recorded ordinary proposal was replaced"
                                .into();
                    }
                    let rejection = validate_investigation(&input, choice.action).err();
                    let decision = ep.investigation.len();
                    ep.investigation.push(Investigation {
                        assessment_input: ev,
                        assessment: a,
                        input,
                        choice,
                        rejection,
                        delivery: None,
                        forced: forced.is_some(),
                        proposal: forced.map(|_| proposal),
                    });
                    r.objective_history
                        .push(Event::Investigation { episode, decision });
                }
                for decision in 0..ep.investigation.len() {
                    let d = &ep.investigation[decision];
                    if d.rejection.is_none()
                        && let InvestigationAction::Inspect(source) = d.choice.action
                    {
                        let agent = d.input.id;
                        let origin = if d.forced {
                            DeliveryOrigin::Forced(decision)
                        } else {
                            DeliveryOrigin::Selected(decision)
                        };
                        let artifact = c
                            .artifacts
                            .iter()
                            .find(|a| a.descriptor.id == source && a.inspectors.contains(&agent))
                            .ok_or("resolver unavailable artifact")?;
                        let index = delivery(
                            &mut ep,
                            &mut l,
                            artifact,
                            agent,
                            origin,
                            &mut r.ledgers[usize::from(agent)],
                            &mut *p.listeners[usize::from(agent)],
                        )?;
                        ep.investigation[decision].delivery = Some(index);
                        r.objective_history.push(Event::Finding {
                            episode,
                            delivery: index,
                        });
                    }
                }
                for (injection, &(agent, source)) in c.injected.iter().enumerate() {
                    let artifact = c
                        .artifacts
                        .iter()
                        .find(|a| a.descriptor.id == source)
                        .unwrap();
                    let index = delivery(
                        &mut ep,
                        &mut l,
                        artifact,
                        agent,
                        DeliveryOrigin::Intervention(injection),
                        &mut r.ledgers[usize::from(agent)],
                        &mut *p.listeners[usize::from(agent)],
                    )?;
                    r.objective_history.push(Event::Finding {
                        episode,
                        delivery: index,
                    });
                }
            }
            if slot == 0 || slot == 6 || slot == 12 {
                r.objective_history.push(Event::Stage {
                    episode,
                    stage: if slot == 0 {
                        Stage::FirstDiscussion
                    } else if slot == 6 {
                        Stage::SecondDiscussion
                    } else {
                        Stage::Judgment
                    },
                });
                if slot != 6 {
                    ep.incident
                        .objective_history
                        .push(e5::Event::Transition(if slot < 12 {
                            Phase::Discussion
                        } else {
                            Phase::Judgment
                        }));
                }
            }
            let agent = c.game.order[slot % 6];
            let id = usize::from(agent);
            let input = l.evidence(id, memories[id].local.credits);
            let assessment = assess(&mut *p.listeners[id], &input)?;
            let update = ep.incident.updates.len();
            ep.incident.updates.push(e5::Update {
                input: input.incident.clone(),
                assessment: assessment.base.clone(),
                before_correct: input.incident.prior.candidate.map(|x| x == c.game.actor()),
                after_correct: assessment.after.candidate.map(|x| x == c.game.actor()),
            });
            ep.cognition.push(Cognition {
                input,
                assessment: assessment.clone(),
            });
            ep.incident
                .objective_history
                .push(e5::Event::Updated { update });
            l.current[id] = assessment.after.clone();
            let view = l.communication(c, id, slot);
            let selected = if slot < 12 {
                p.speakers[id].choose(&view)
            } else {
                CommunicationChoice {
                    choice: e5::JudgmentPolicy::choose(
                        &mut c.game.judgments[id].clone(),
                        &view.incident,
                    ),
                    report: None,
                }
            };
            let mut rejection = validate_communication(&view, &selected).err();
            let mut lineage = None;
            if rejection.is_none() && matches!(selected.choice.action, e5::Action::Speak { .. }) {
                let resolved = if let Some((_, basis)) = &selected.report {
                    let body_target = match &selected.choice.action {
                        e5::Action::Speak { body, .. } => body.target(),
                        _ => None,
                    };
                    let (parents, roots, depth) = match basis {
                        ReportBasis::Acquired(_) => (
                            vec![],
                            vec![e5::Root::Construction(ep.incident.testimony.len())],
                            0,
                        ),
                        ReportBasis::Received(parent) => {
                            let p = &ep.incident.testimony[*parent].lineage;
                            (vec![*parent], p.roots.clone(), p.depth + 1)
                        }
                    };
                    if depth > 3 {
                        Err("finding report depth exceeds three".into())
                    } else {
                        Ok(e5::Lineage {
                            basis: e5::Basis::Inference,
                            evidence_parents: parents,
                            roots,
                            depth,
                            revision_of: body_target,
                        })
                    }
                } else {
                    e5::causal_lineage(
                        &view.incident,
                        &selected.choice,
                        &assessment.base,
                        &ep.incident.testimony,
                    )
                };
                match resolved {
                    Ok(a) => lineage = Some(a),
                    Err(e) => rejection = Some(e),
                }
            }
            let accepted = rejection.is_none();
            let decision = ep.incident.decisions.len();
            let classification = e5::classify(
                &view.incident,
                &selected.choice,
                c.game.actor(),
                &ep.incident.testimony,
                accepted,
            );
            ep.incident.decisions.push(e5::Decision {
                update,
                input: view.incident.clone(),
                choice: selected.choice.clone(),
                rejection,
                classification,
            });
            ep.communication.push(Communication {
                input: view.clone(),
                choice: selected.clone(),
                decision,
            });
            ep.incident
                .objective_history
                .push(e5::Event::Decided { decision });
            let mut message = None;
            let mut ballot = None;
            if accepted {
                match &selected.choice.action {
                    e5::Action::Speak {
                        body,
                        attribution,
                        recipients,
                        ..
                    } => {
                        let mid = ep.incident.testimony.len();
                        message = Some(mid);
                        let wire = e5::Message {
                            id: mid,
                            speaker: agent,
                            body: body.clone(),
                            attribution: *attribution,
                        };
                        let mut actual_roots = vec![];
                        if let Some((finding, basis)) = &selected.report {
                            let reported = ReportedFinding {
                                message: mid,
                                finding: finding.clone(),
                                basis: *basis,
                            };
                            ep.reports.push(reported.clone());
                            // Observer ancestry resolves acquired or received roots, but
                            // neither roots nor private intent enter the wire payload.
                            match basis {
                                ReportBasis::Acquired(a) => {
                                    if let Some(d) =
                                        ep.deliveries.iter().find(|d| d.finding.acquisition == *a)
                                    {
                                        actual_roots.push(d.actual_root);
                                    }
                                }
                                ReportBasis::Received(parent) => {
                                    actual_roots
                                        .extend(ep.ancestry[*parent].actual_roots.iter().copied());
                                }
                            }
                            for &recipient in recipients {
                                l.reports[usize::from(recipient)].push(reported.clone());
                            }
                        } else {
                            let legacy = lineage.as_ref().unwrap();
                            for root in &legacy.roots {
                                if let e5::Root::Observation(owner) = root {
                                    actual_roots.push(Root::Observation(*owner as u8));
                                }
                            }
                            for parent in &legacy.evidence_parents {
                                actual_roots
                                    .extend(ep.ancestry[*parent].actual_roots.iter().copied());
                            }
                            for finding in &view.findings {
                                let key = finding
                                    .source
                                    .visible_parent
                                    .map(|owner| Perceived::Source(e5::Group::Speaker(owner)))
                                    .unwrap_or(Perceived::Artifact(finding.source.id));
                                if assessment.contributions.iter().any(|g| {
                                    g.group == key
                                        && g.weight > 0
                                        && g.candidate == view.incident.current.candidate
                                }) && let Some(d) = ep
                                    .deliveries
                                    .iter()
                                    .find(|d| d.finding.acquisition == finding.acquisition)
                                {
                                    actual_roots.push(d.actual_root);
                                }
                            }
                        }
                        actual_roots.sort();
                        actual_roots.dedup();
                        ep.ancestry.push(Ancestry {
                            message: mid,
                            actual_roots,
                            report_basis: selected.report.as_ref().map(|x| x.1),
                        });
                        ep.incident.testimony.push(e5::Testimony {
                            message: wire.clone(),
                            recipients: recipients.clone(),
                            origin: e5::Origin::Selected { decision },
                            lineage: lineage.unwrap(),
                            intent: selected.choice.intent,
                            belief: view.incident.current.candidate,
                            classification,
                            correct: body.candidate().map(|x| x == c.game.actor()),
                        });
                        l.sent[id].push(wire.clone());
                        for &recipient in recipients {
                            l.received[usize::from(recipient)].push(wire.clone());
                            ep.incident.objective_history.push(e5::Event::Delivered {
                                message: mid,
                                recipient,
                            });
                        }
                    }
                    e5::Action::Vote(Some(c)) => {
                        ep.incident.tally[usize::from(*c)] += 1;
                        ballot = Some(*c);
                    }
                    _ => {}
                }
            }
            ep.incident.objective_history.push(e5::Event::Resolved {
                decision,
                accepted,
                message,
                ballot,
            });
        }
        ep.incident.verdict = crate::verdict(ep.incident.tally);
        ep.incident.correct = ep.incident.verdict.map(|x| x == c.game.actor());
        ep.incident.retained_received_copies = l.received.iter().map(Vec::len).sum();
        ep.incident.retained_sent_copies = l.sent.iter().map(Vec::len).sum();
        ep.incident
            .objective_history
            .push(e5::Event::Transition(Phase::Terminated));
        ep.incident.objective_history.push(e5::Event::Outcome {
            tally: ep.incident.tally,
            verdict: ep.incident.verdict,
            correct: ep.incident.correct,
        });
        r.objective_history.push(Event::Stage {
            episode,
            stage: Stage::Complete,
        });
        for (id, memory) in memories.iter_mut().enumerate() {
            // Explicit environment reports and their revision branches do not
            // create independently graded speaker experiences.
            let mut input = l.evidence(id, memory.local.credits).incident;
            let roots: Vec<_> = e5::Listener::assess(&mut e5::ListenerKind::Revision, &input)
                .interpretations
                .iter()
                .filter(|i| l.reports[id].iter().any(|p| p.message == i.message))
                .filter_map(|i| i.root)
                .collect();
            let interpreted = e5::Listener::assess(&mut e5::ListenerKind::Revision, &input);
            input.received.retain(|m| {
                !interpreted
                    .interpretations
                    .iter()
                    .any(|i| i.message == m.id && i.root.is_some_and(|root| roots.contains(&root)))
            });
            let interpretation = e5::Listener::assess(&mut e5::ListenerKind::Revision, &input);
            let ingestion = memory.ingest(
                e6::IngestView {
                    episode,
                    observer: id as u8,
                    received: input.received,
                    interpretation,
                },
                e7::VerificationKind::ConflictAware,
            )?;
            r.objective_history.push(Event::Ingest {
                episode,
                observer: id as u8,
                evicted: ingestion.evicted.clone(),
            });
            ep.ingestions.push(ingestion);
            if config.feedback {
                let input = e7::VerificationView {
                    delivery: usize::from(episode) * 6 + id,
                    completed_episode: episode,
                    session: config.session,
                    memory: memory.clone(),
                    received: e7::Token::outcome(config.session, episode, 0, c.game.actor()),
                };
                let effect = e7::VerificationKind::ConflictAware.accept(&input);
                effect.after.validate()?;
                *memory = effect.after.clone();
                ep.verification.push((input, effect));
                r.objective_history.push(Event::Verified {
                    episode,
                    observer: id as u8,
                });
            }
        }
        ep.memory_after = memories.clone();
        r.episodes.push(ep);
        r.objective_history.push(Event::End(episode));
    }
    if r.ledgers.iter().any(|l| l.initial != l.remaining + l.spent) {
        return Err("effort conservation violated".into());
    }
    r.final_memory = memories;
    Ok(r)
}
