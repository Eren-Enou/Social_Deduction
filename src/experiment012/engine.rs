// Narrowly derived from the frozen 010 scheduler: new method speech and local social selection hooks.
use super::*;
use crate::{
    Phase, experiment005 as e5, experiment006 as e6, experiment007 as e7, experiment009 as e9,
    experiment010 as e10,
};
use e7::VerificationPolicy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub initial: u8,
    pub remaining: u8,
    pub spent: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Investigation {
    pub before_answer_input: InvestigationView,
    pub before_answer_choice: InvestigationChoice,
    pub assessment_input: EvidenceView,
    pub assessment: Assessment,
    pub input: InvestigationView,
    pub choice: InvestigationChoice,
    pub rejection: Option<String>,
    pub delivery: Option<usize>,
    pub forced: bool,
    pub proposal: Option<InvestigationChoice>,
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
    Request {
        episode: u8,
        decision: usize,
    },
    RequestDelivered {
        episode: u8,
        decision: usize,
    },
    Response {
        episode: u8,
        decision: usize,
    },
    ResponseDelivered {
        episode: u8,
        decision: usize,
    },
    Capacity {
        episode: u8,
        record: usize,
    },
    Social {
        episode: u8,
        decision: usize,
    },
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
    MethodAcquired {
        episode: u8,
        observer: u8,
        ingestion: usize,
    },
    MethodFeedback {
        episode: u8,
        observer: u8,
        feedback: usize,
    },
    End(u8),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Episode {
    pub inquiry_before: [InquiryMemory; 6],
    pub inquiry_after: [InquiryMemory; 6],
    pub requests: Vec<RequestDecision>,
    pub responses: Vec<ResponseDecision>,
    pub capacity: Vec<CapacityRecord>,
    pub before_answers: [SocialMemory; 6],
    pub social_before: [SocialMemory; 6],
    pub social_after: [SocialMemory; 6],
    pub social_decisions: Vec<SocialDecision>,
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
    pub method_before: [MethodMemory; 6],
    pub method_ingestions: Vec<MethodIngestion>,
    pub indirect_ingestions: Vec<IndirectIngestion>,
    pub method_feedback: Vec<MethodFeedbackAudit>,
    pub method_after: [MethodMemory; 6],
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
    pub final_method_memory: [MethodMemory; 6],
    pub final_social_memory: [SocialMemory; 6],
    pub final_inquiry_memory: [InquiryMemory; 6],
}
pub struct Components {
    pub requesters: Vec<Box<dyn RequestPolicy>>,
    pub responders: Vec<Box<dyn ResponsePolicy>>,
    pub sharers: Vec<Box<dyn SharingPolicy>>,
    pub investigators: Vec<Box<dyn InvestigationPolicy>>,
    pub speakers: Vec<Box<dyn CommunicationPolicy>>,
    pub listeners: Vec<Box<dyn EvidenceListener>>,
    pub updaters: Vec<Box<dyn ReliabilityUpdater>>,
}
impl Components {
    pub fn from_config(c: &Config) -> Self {
        Self {
            requesters: (0..6)
                .map(|id| {
                    Box::new(Requester::new(c.request[id], c.session, id as u8))
                        as Box<dyn RequestPolicy>
                })
                .collect(),
            responders: c
                .response
                .iter()
                .map(|k| Box::new(*k) as Box<dyn ResponsePolicy>)
                .collect(),
            sharers: c
                .sharing
                .iter()
                .map(|k| Box::new(*k) as Box<dyn SharingPolicy>)
                .collect(),
            updaters: (0..6)
                .map(|_| Box::new(CountUpdater) as Box<dyn ReliabilityUpdater>)
                .collect(),
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
        return Err("inconsistent/inaccessible010 listener assessment".into());
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
#[allow(clippy::too_many_arguments)]
fn inquiry_view(
    c: &Incident,
    l: &Local,
    id: usize,
    slot: usize,
    episode: u8,
    window: Window,
    capacity: u8,
    balance: u8,
    personal: &[MethodMemory; 6],
    social: &[SocialMemory; 6],
    memory: &[InquiryMemory; 6],
    credits: &[e7::Memory; 6],
) -> InquiryView {
    let mut incident = l.communication(
        c,
        id,
        if window == Window::First {
            slot
        } else {
            6 + slot % 6
        },
    );
    incident.incident.opportunities_left = capacity;
    let catalog = c
        .artifacts
        .iter()
        .filter(|a| a.inspectors.contains(&(id as u8)))
        .map(|a| PublicMethod {
            source: a.public(),
            family: c
                .methods
                .iter()
                .find(|(s, _)| *s == a.descriptor.id)
                .unwrap()
                .1
                .clone(),
        })
        .collect();
    InquiryView {
        local: SharingView {
            reference: Reference {
                episode,
                slot: slot as u8,
            },
            incident,
            personal: personal[id].clone(),
            social: social[id].clone(),
            catalog,
            credits: credits[id].local.credits,
        },
        memory: memory[id].clone(),
        window,
        capacity,
        balance,
        permitted_request_type: None,
    }
}
#[allow(clippy::too_many_arguments)]
fn response_opportunity(
    config: &Config,
    c: &Incident,
    l: &Local,
    id: usize,
    slot: usize,
    episode: u8,
    window: Window,
    used: &mut [u8; 6],
    ledgers: &[Ledger; 6],
    personal: &[MethodMemory; 6],
    social: &[SocialMemory; 6],
    memory: &[InquiryMemory; 6],
    credits: &[e7::Memory; 6],
    p: &mut Components,
    ep: &mut Episode,
    events: &mut Vec<Event>,
    ancestry: &std::collections::BTreeMap<Reference, (Vec<Reference>, u8)>,
) -> Result<(), String> {
    let input = inquiry_view(
        c,
        l,
        id,
        slot,
        episode,
        window,
        2 - used[id],
        ledgers[id].remaining,
        personal,
        social,
        memory,
        credits,
    );
    let proposal = p.responders[id].choose(&input);
    let injected =
        config.answer_injections.iter().enumerate().find(|(_, i)| {
            i.episode == episode && usize::from(i.speaker) == id && i.window == window
        });
    let (choice, origin) = injected
        .map(|(n, i)| (i.choice.clone(), Origin::Intervention(n)))
        .unwrap_or_else(|| (proposal.clone(), Origin::Selected));
    let mut rejection = validate_response(&input, &choice).err();
    let mut resolved = None;
    if rejection.is_none()
        && let Some(sharing) = response_sharing(&input, &choice)
    {
        let mut local = input.local.clone();
        local.incident.incident.permitted_recipients =
            vec![choice.selected.as_ref().unwrap().recipient];
        let d = resolve_report(local, sharing, origin, ancestry);
        rejection = d.rejection.clone();
        resolved = Some(d);
    }
    let before = used[id];
    let submitted = choice.selected.is_some();
    // An exhausted submission is rejected without creating a third action.
    if submitted && before < 2 {
        used[id] += 1;
    }
    ep.capacity.push(CapacityRecord {
        reference: input.local.reference,
        agent: id as u8,
        window,
        before,
        after: used[id],
        submitted: submitted && before < 2,
        rejected: rejection.is_some(),
        exhausted: before >= 2,
        foregone: None,
        foregone_input: None,
    });
    events.push(Event::Capacity {
        episode,
        record: ep.capacity.len() - 1,
    });
    ep.responses.push(ResponseDecision {
        input,
        choice,
        proposal: injected.map(|_| proposal),
        origin,
        rejection,
        delivered: vec![],
        social: resolved,
    });
    events.push(Event::Response {
        episode,
        decision: ep.responses.len() - 1,
    });
    Ok(())
}
fn deliver_answers(
    config: &Config,
    ep: &mut Episode,
    memory: &mut [InquiryMemory; 6],
    social: &mut [SocialMemory; 6],
    ancestry: &mut std::collections::BTreeMap<Reference, (Vec<Reference>, u8)>,
    events: &mut Vec<Event>,
) -> Result<(), String> {
    for (decision, d) in ep.responses.iter_mut().enumerate() {
        if d.rejection.is_some() || !d.delivered.is_empty() {
            continue;
        }
        let Some(a) = d.choice.selected.clone() else {
            continue;
        };
        let sender = usize::from(a.speaker);
        let recipient = usize::from(a.recipient);
        let own = remember_response(&memory[sender], a.clone(), true)?;
        memory[sender] = own.after.clone();
        d.delivered.push(own);
        let received = remember_response(&memory[recipient], a.clone(), false)?;
        memory[recipient] = received.after.clone();
        d.delivered.push(received);
        if let Some(resolved) = &mut d.social
            && let Some(message) = resolved.message.clone()
        {
            ancestry.insert(
                message.reference,
                (resolved.actual_origins.clone(), resolved.actual_depth),
            );
            let own = ingest(&social[sender], message.clone(), true)?;
            social[sender] = own.after.clone();
            resolved.deliveries.push(own);
            let mut recipients = vec![a.recipient];
            if let Origin::Intervention(n) = d.origin {
                recipients.extend(&config.answer_injections[n].wider);
            }
            recipients.sort();
            recipients.dedup();
            for recipient in recipients {
                let effect = ingest(&social[usize::from(recipient)], message.clone(), false)?;
                social[usize::from(recipient)] = effect.after.clone();
                resolved.deliveries.push(effect);
            }
        }
        events.push(Event::ResponseDelivered {
            episode: ep.id,
            decision,
        });
    }
    Ok(())
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
    r.implementation = "builtin-012-v1".into();
    Ok(r)
}
pub fn run_with(config: Config, p: &mut Components) -> Result<Report, String> {
    run_inner(config, p, false)
}
fn run_inner(config: Config, p: &mut Components, builtin: bool) -> Result<Report, String> {
    config.validate()?;
    if [
        p.sharers.len(),
        p.requesters.len(),
        p.responders.len(),
        p.investigators.len(),
        p.speakers.len(),
        p.listeners.len(),
        p.updaters.len(),
    ]
    .iter()
    .any(|n| *n != 6)
    {
        return Err("six010 policies per mechanism required".into());
    }
    let mut memories: [e7::Memory; 6] = std::array::from_fn(|id| e7::Memory::neutral(id as u8));
    let mut method_memories: [MethodMemory; 6] =
        std::array::from_fn(|id| MethodMemory::neutral(id as u8));
    let mut social_memories: [SocialMemory; 6] =
        std::array::from_fn(|id| SocialMemory::neutral(id as u8));
    let mut inquiry_memories: [InquiryMemory; 6] =
        std::array::from_fn(|id| InquiryMemory::neutral(id as u8));
    let mut social_ancestry = std::collections::BTreeMap::new();
    let mut r = Report {
        version: "series-012-v1".into(),
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
        final_method_memory: method_memories.clone(),
        final_social_memory: social_memories.clone(),
        final_inquiry_memory: inquiry_memories.clone(),
    };
    for (episode, c) in config.incidents.iter().enumerate() {
        let episode = episode as u8;
        if builtin {
            let sharing = config
                .sharing_schedule
                .get(usize::from(episode))
                .unwrap_or(&config.sharing);
            p.sharers = sharing
                .iter()
                .map(|k| Box::new(*k) as Box<dyn SharingPolicy>)
                .collect();
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
            rules_version: "incident-012-v1".into(),
            revision_version: "revision-005-v1".into(),
            listener_version: "evidence-listener-008-v1 + experience-listener-006-v1".into(),
            policy_version: "communication-008-v1".into(),
            implementation: "012 scheduler / unchanged005 legal communication".into(),
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
        let mut used = [0u8; 6];
        let mut ep = Episode {
            inquiry_before: inquiry_memories.clone(),
            inquiry_after: inquiry_memories.clone(),
            requests: vec![],
            responses: vec![],
            capacity: vec![],
            before_answers: social_memories.clone(),
            social_before: social_memories.clone(),
            social_after: social_memories.clone(),
            social_decisions: vec![],
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
            method_before: method_memories.clone(),
            method_ingestions: vec![],
            indirect_ingestions: vec![],
            method_feedback: vec![],
            method_after: method_memories.clone(),
        };
        for id in 0..6 {
            ep.incident.objective_history.push(e5::Event::Assigned {
                agent: id as u8,
                original: l.originals[id].clone(),
                observation: l.observations[id][0].clone(),
            });
        }
        for physical_slot in 0..24 {
            let slot = if physical_slot < 12 {
                physical_slot
            } else {
                physical_slot - 6
            };
            if physical_slot == 6 {
                ep.before_answers = social_memories.clone();
                for (decision, d) in ep.requests.iter_mut().enumerate() {
                    if d.rejection.is_none()
                        && let Some(q) = &d.choice.selected
                    {
                        let target = usize::from(q.recipient);
                        d.recipient_counts = Some(
                            method_memories[target]
                                .summaries
                                .get(&q.family)
                                .copied()
                                .unwrap_or_default(),
                        );
                        let effect = remember_request(&inquiry_memories[target], q.clone(), false)?;
                        inquiry_memories[target] = effect.after.clone();
                        d.delivered = Some(effect);
                        r.objective_history
                            .push(Event::RequestDelivered { episode, decision });
                    }
                }
            }
            if physical_slot == 12 {
                deliver_answers(
                    &config,
                    &mut ep,
                    &mut inquiry_memories,
                    &mut social_memories,
                    &mut social_ancestry,
                    &mut r.objective_history,
                )?;
            }
            if (6..12).contains(&physical_slot) {
                let agent = c.game.order[physical_slot % 6];
                let id = usize::from(agent);
                if inquiry_memories[id]
                    .received
                    .iter()
                    .any(|q| !inquiry_memories[id].answered.contains(&q.reference))
                    && !config.late_responses
                {
                    let ev = l.evidence(id, memories[id].local.credits);
                    l.current[id] = assess(&mut *p.listeners[id], &ev)?.after;
                    response_opportunity(
                        &config,
                        c,
                        &l,
                        id,
                        physical_slot,
                        episode,
                        Window::Response,
                        &mut used,
                        &r.ledgers,
                        &method_memories,
                        &social_memories,
                        &inquiry_memories,
                        &memories,
                        p,
                        &mut ep,
                        &mut r.objective_history,
                        &social_ancestry,
                    )?;
                }
                continue;
            }
            if physical_slot == 12 {
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
                    let local = e9::InvestigationView {
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
                    let methods = local
                        .catalog
                        .iter()
                        .map(|s| PublicMethod {
                            source: s.clone(),
                            family: c
                                .methods
                                .iter()
                                .find(|(source, _)| *source == s.descriptor.id)
                                .unwrap()
                                .1
                                .clone(),
                        })
                        .collect();
                    let input = InvestigationView {
                        personal: e10::InvestigationView {
                            local,
                            methods,
                            experience: method_memories[id].summaries.clone(),
                        },
                        social: social_memories[id].clone(),
                        listener: config.social[id],
                    };
                    input.validate()?;
                    let mut before_answer_input = input.clone();
                    before_answer_input.social = ep.before_answers[id].clone();
                    let before_answer_choice =
                        Policy::new(config.policy[id], config.session, agent)
                            .choose(&before_answer_input);
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
                        before_answer_input,
                        before_answer_choice,
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
                        let agent = d.input.local.id;
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
            if physical_slot == 12 {
                for d in &ep.deliveries {
                    let id = usize::from(d.finding.acquisition.observer);
                    let method = c
                        .methods
                        .iter()
                        .find(|(source, _)| *source == d.finding.source.id)
                        .unwrap()
                        .1
                        .clone();
                    let input = MethodIngestView {
                        memory: method_memories[id].clone(),
                        method,
                        finding: d.finding.clone(),
                        origin: d.origin,
                    };
                    let effect = p.updaters[id].ingest(&input)?;
                    validate_method_ingestion(&input, &effect)?;
                    method_memories[id] = effect.after.clone();
                    r.objective_history.push(Event::MethodAcquired {
                        episode,
                        observer: id as u8,
                        ingestion: ep.method_ingestions.len(),
                    });
                    ep.method_ingestions.push(effect);
                }
            }
            if slot == 0 || slot == 6 || slot == 12 {
                r.objective_history.push(Event::Stage {
                    episode,
                    stage: if slot == 0 {
                        Stage::FirstDiscussion
                    } else if physical_slot == 12 {
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
            if physical_slot == 18 && config.late_responses {
                deliver_answers(
                    &config,
                    &mut ep,
                    &mut inquiry_memories,
                    &mut social_memories,
                    &mut social_ancestry,
                    &mut r.objective_history,
                )?;
            }
            if (12..18).contains(&physical_slot)
                && config.late_responses
                && inquiry_memories[id]
                    .received
                    .iter()
                    .any(|q| !inquiry_memories[id].answered.contains(&q.reference))
            {
                response_opportunity(
                    &config,
                    c,
                    &l,
                    id,
                    physical_slot,
                    episode,
                    Window::Final,
                    &mut used,
                    &r.ledgers,
                    &method_memories,
                    &social_memories,
                    &inquiry_memories,
                    &memories,
                    p,
                    &mut ep,
                    &mut r.objective_history,
                    &social_ancestry,
                )?;
                continue;
            }
            let mut view = l.communication(c, id, slot);
            if slot < 12 {
                view.incident.opportunities_left = 2 - used[id];
            }
            let exhausted = slot < 12 && used[id] >= 2;
            let mut inquiry_consumed = false;
            let mut query_submitted = false;
            let mut query_rejected = false;
            if physical_slot < 6 && episode >= config.start_episode && !exhausted {
                let mut input = inquiry_view(
                    c,
                    &l,
                    id,
                    physical_slot,
                    episode,
                    Window::First,
                    2 - used[id],
                    r.ledgers[id].remaining,
                    &method_memories,
                    &social_memories,
                    &inquiry_memories,
                    &memories,
                );
                input.permitted_request_type = config.request_type;
                let choice = p.requesters[id].choose(&input);
                let rejection = choice
                    .selected
                    .as_ref()
                    .and_then(|q| validate_request(&input, q).err());
                query_submitted = choice.selected.is_some();
                query_rejected = rejection.is_some();
                inquiry_consumed = query_submitted;
                let mut sent = None;
                if rejection.is_none()
                    && let Some(q) = &choice.selected
                {
                    let effect = remember_request(&inquiry_memories[id], q.clone(), true)?;
                    inquiry_memories[id] = effect.after.clone();
                    sent = Some(effect);
                }
                r.objective_history.push(Event::Request {
                    episode,
                    decision: ep.requests.len(),
                });
                ep.requests.push(RequestDecision {
                    input,
                    choice,
                    rejection,
                    delivered: None,
                    sent,
                    recipient_counts: None,
                });
            }
            let mut method_consumed = false;
            if slot < 12 && !exhausted && !inquiry_consumed {
                let catalog = c
                    .artifacts
                    .iter()
                    .filter(|a| a.inspectors.contains(&agent))
                    .map(|a| PublicMethod {
                        source: a.public(),
                        family: c
                            .methods
                            .iter()
                            .find(|(source, _)| *source == a.descriptor.id)
                            .unwrap()
                            .1
                            .clone(),
                    })
                    .collect();
                let input = SharingView {
                    reference: Reference {
                        episode,
                        slot: physical_slot as u8,
                    },
                    incident: view.clone(),
                    personal: method_memories[id].clone(),
                    social: social_memories[id].clone(),
                    catalog,
                    credits: memories[id].local.credits,
                };
                let injected = config
                    .injections
                    .iter()
                    .enumerate()
                    .find(|(_, i)| i.episode == episode && usize::from(i.slot) == slot);
                let (choice, origin) = if let Some((index, i)) = injected {
                    (i.choice.clone(), Origin::Intervention(index))
                } else if config
                    .report_round
                    .is_some_and(|round| usize::from(round) != slot / 6)
                {
                    (SharingKind::SilentMethod.choose(&input), Origin::Selected)
                } else {
                    (p.sharers[id].choose(&input), Origin::Selected)
                };
                method_consumed = !matches!(choice.action, MethodAction::IncidentSpeech);
                let mut resolved = resolve_report(input, choice, origin, &social_ancestry);
                if let Some(message) = resolved.message.clone() {
                    social_ancestry.insert(
                        message.reference,
                        (resolved.actual_origins.clone(), resolved.actual_depth),
                    );
                    let own = ingest(&social_memories[id], message.clone(), true)?;
                    social_memories[id] = own.after.clone();
                    resolved.deliveries.push(own);
                    if let MethodAction::Report { recipients, .. } = &resolved.choice.action {
                        for &recipient in recipients {
                            let delivery = ingest(
                                &social_memories[usize::from(recipient)],
                                message.clone(),
                                false,
                            )?;
                            social_memories[usize::from(recipient)] = delivery.after.clone();
                            resolved.deliveries.push(delivery);
                        }
                    }
                }
                r.objective_history.push(Event::Social {
                    episode,
                    decision: ep.social_decisions.len(),
                });
                ep.social_decisions.push(resolved);
            }
            let foregone_input = exhausted.then(|| {
                let mut legal = view.clone();
                legal.incident.opportunities_left = 1;
                legal
            });
            let foregone = foregone_input
                .as_ref()
                .map(|legal| Speaker::new(c.communication[id]).choose(legal));
            let selected = if slot < 12 && (method_consumed || inquiry_consumed || exhausted) {
                CommunicationChoice {
                    choice: e5::Choice::silence(
                        "012 method action consumed this speech opportunity; see social decision",
                    ),
                    report: None,
                }
            } else if slot < 12 {
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
            let mut validation_view = view.clone();
            if exhausted {
                validation_view.incident.opportunities_left = 1;
            }
            let mut rejection = validate_communication(&validation_view, &selected).err();
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
            if slot < 12 {
                let social = ep.social_decisions.last().filter(|d| {
                    d.input.reference.episode == episode
                        && usize::from(d.input.reference.slot) == physical_slot
                });
                let method_submitted = method_consumed
                    && social
                        .is_some_and(|d| matches!(d.choice.action, MethodAction::Report { .. }));
                let submitted = query_submitted
                    || method_submitted
                    || matches!(selected.choice.action, e5::Action::Speak { .. });
                let before = used[id];
                if submitted {
                    used[id] += 1;
                }
                ep.capacity.push(CapacityRecord {
                    reference: Reference {
                        episode,
                        slot: physical_slot as u8,
                    },
                    agent,
                    window: if physical_slot < 6 {
                        Window::First
                    } else {
                        Window::Final
                    },
                    before,
                    after: used[id],
                    submitted,
                    rejected: query_rejected
                        || social.is_some_and(|d| d.rejection.is_some())
                        || rejection.is_some(),
                    exhausted,
                    foregone,
                    foregone_input,
                });
                r.objective_history.push(Event::Capacity {
                    episode,
                    record: ep.capacity.len() - 1,
                });
            }
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
            for report in &l.reports[id] {
                if report.finding.acquisition.observer == id as u8 {
                    continue;
                }
                let family = c
                    .artifacts
                    .iter()
                    .find(|a| {
                        a.descriptor.id == report.finding.source.id
                            && a.inspectors.contains(&(id as u8))
                    })
                    .and_then(|a| {
                        c.methods
                            .iter()
                            .find(|(source, _)| *source == a.descriptor.id)
                    })
                    .map(|(_, family)| family.clone());
                let received = IndirectExperience {
                    episode,
                    message: report.message,
                    method: family,
                    finding: report.finding.clone(),
                };
                let effect = ingest_indirect(&method_memories[id], received)?;
                method_memories[id] = effect.after.clone();
                ep.indirect_ingestions.push(effect);
            }
        }
        for (plan, f) in config
            .feedback
            .iter()
            .enumerate()
            .filter(|(_, f)| f.after_episode == episode)
        {
            for &observer in &f.recipients {
                let id = usize::from(observer);
                let delivery = plan * 6 + id;
                let input = e7::VerificationView {
                    delivery,
                    completed_episode: episode,
                    session: config.session,
                    memory: memories[id].clone(),
                    received: f.token.clone(),
                };
                let effect = e7::VerificationKind::ConflictAware.accept(&input);
                effect.after.validate()?;
                memories[id] = effect.after.clone();
                ep.verification.push((input, effect));
                r.objective_history
                    .push(Event::Verified { episode, observer });
                let input = MethodFeedbackView {
                    memory: method_memories[id].clone(),
                    received: f.token.clone(),
                    delivery,
                    completed_episode: episode,
                    session: config.session,
                };
                let effect = p.updaters[id].verify(&input)?;
                validate_method_feedback(&input, &effect)?;
                method_memories[id] = effect.after.clone();
                let actual_actor = config.incidents[usize::from(f.about_episode)].game.actor();
                let correct = match f.token.kind {
                    e7::TokenKind::Outcome(candidate) => Some(candidate == actual_actor),
                    _ => None,
                };
                r.objective_history.push(Event::MethodFeedback {
                    episode,
                    observer,
                    feedback: ep.method_feedback.len(),
                });
                ep.method_feedback.push(MethodFeedbackAudit {
                    observer,
                    plan,
                    actual_actor,
                    correct,
                    effect,
                });
            }
        }
        ep.inquiry_after = inquiry_memories.clone();
        ep.social_after = social_memories.clone();
        ep.method_after = method_memories.clone();
        ep.memory_after = memories.clone();
        r.episodes.push(ep);
        r.objective_history.push(Event::End(episode));
    }
    if r.ledgers.iter().any(|l| l.initial != l.remaining + l.spent) {
        return Err("effort conservation violated".into());
    }
    r.final_memory = memories;
    r.final_method_memory = method_memories;
    r.final_social_memory = social_memories;
    r.final_inquiry_memory = inquiry_memories;
    Ok(r)
}
