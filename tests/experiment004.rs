use social_deduction::{Phase, experiment003 as e3, experiment004::*};

fn unknown() -> ListenerView {
    ListenerView {
        id: 5,
        original: IndividualState {
            observation: None,
            briefing: None,
            belief: None,
            incentives: Incentives::ACCURACY,
        },
        prior: Belief::default(),
        messages: vec![],
    }
}
fn message(id: usize, speaker: u8, attribution: Attribution) -> Message {
    Message {
        id,
        speaker,
        speech: Speech::Assert(1),
        attribution,
    }
}

#[test]
fn known_independent_shared_and_opaque_sources_have_distinct_support() {
    let mut view = unknown();
    view.messages = vec![
        message(0, 0, Attribution::Firsthand),
        message(1, 1, Attribution::Reference(0)),
        message(2, 2, Attribution::Reference(0)),
    ];
    let known = ListenerKind::Provenance.assess(&view);
    let baseline = ListenerKind::SourceCount.assess(&view);
    assert_eq!(known.after.scores[1], 1);
    assert_eq!(baseline.after.scores[1], 3);
    assert_eq!(known.sources[1].treatment, Treatment::DuplicateDiscount);
    assert_eq!(known.sources[1].verification, Verification::VisibleLink);
    assert!(known.sources.iter().all(|s| !s.origin_verified));
    view.messages[1].attribution = Attribution::Firsthand;
    view.messages[2].attribution = Attribution::Firsthand;
    let declared_independent = ListenerKind::Provenance.assess(&view);
    assert_eq!(declared_independent.after.scores[1], 3);
    assert!(
        declared_independent
            .sources
            .iter()
            .all(|s| s.verification == Verification::ClaimOnly && !s.origin_verified)
    );
    view.messages = vec![
        message(1, 1, Attribution::Reference(0)),
        message(2, 2, Attribution::Reference(0)),
    ];
    let opaque = ListenerKind::Provenance.assess(&view);
    assert_eq!(opaque.after.scores[1], 1);
    assert_eq!(opaque.perceived_groups, vec![Group::OpaqueMessage(0)]);
    assert!(
        opaque
            .sources
            .iter()
            .all(|s| s.verification == Verification::MissingReference)
    );
    assert!(
        opaque
            .sources
            .iter()
            .all(|s| !s.checked_messages.contains(&0))
    );
    view.messages[0].attribution = Attribution::NamedSource(3);
    view.messages[1].attribution = Attribution::NamedSource(3);
    let named = ListenerKind::Provenance.assess(&view);
    assert_eq!(named.after.scores[1], 1);
    assert_eq!(named.perceived_groups, vec![Group::Speaker(3)]);
    assert!(named.sources.iter().all(|s| !s.origin_verified));
}
#[test]
fn baseline_is_the_unchanged_003_listener_and_corrections_are_not_added() {
    let mut view = unknown();
    view.messages = vec![
        message(0, 0, Attribution::Unspecified),
        message(1, 0, Attribution::NamedSource(1)),
        message(2, 1, Attribution::Reference(0)),
    ];
    let baseline = ListenerKind::SourceCount.assess(&view);
    let original = e3::Listener::assess(&mut e3::ListenerKind::Evidence, &source_count_view(&view));
    assert_eq!(baseline.after, original.after);
    let aware = ListenerKind::Provenance.assess(&view);
    assert_eq!(aware.after.scores[1], 1);
    assert_eq!(aware.sources[1].treatment, Treatment::RepeatIgnored);
    view.messages[1].speech = Speech::Assert(2);
    assert_eq!(ListenerKind::Provenance.assess(&view).after, aware.after);
    view.messages.push(Message {
        id: 3,
        speaker: 2,
        speech: Speech::Uncertain,
        attribution: Attribution::Firsthand,
    });
    assert_eq!(
        ListenerKind::Provenance.assess(&view).sources[3].treatment,
        Treatment::UncertaintyIgnored
    );
}
#[test]
fn malformed_cycles_forward_links_mismatches_and_depth_limits_are_uncertain() {
    let mut view = unknown();
    view.messages = vec![
        message(0, 0, Attribution::Reference(1)),
        message(1, 1, Attribution::Reference(0)),
    ];
    assert_eq!(
        resolve_source(&view.messages[0], &view.messages).1,
        Verification::MalformedReference
    );
    let assessed = ListenerKind::Provenance.assess(&view);
    assert!(
        assessed
            .sources
            .iter()
            .all(|s| s.treatment == Treatment::UncertainProvenance)
    );
    assert_eq!(assessed.after.candidate, None);
    view.messages = vec![
        message(0, 0, Attribution::Firsthand),
        message(1, 1, Attribution::Reference(0)),
    ];
    view.messages[1].speech = Speech::Assert(2);
    assert_eq!(
        resolve_source(&view.messages[1], &view.messages).1,
        Verification::InconsistentReference
    );
    view.messages = (0..5)
        .map(|id| {
            message(
                id,
                id as u8,
                if id == 0 {
                    Attribution::Firsthand
                } else {
                    Attribution::Reference(id - 1)
                },
            )
        })
        .collect();
    assert_eq!(
        resolve_source(&view.messages[3], &view.messages).0,
        Some(Group::Speaker(0))
    );
    assert_eq!(
        resolve_source(&view.messages[4], &view.messages).1,
        Verification::DepthExceeded
    );
}
#[test]
fn hidden_actual_roots_and_other_private_states_do_not_leak() {
    for seed in 0..16 {
        for listener in [ListenerKind::SourceCount, ListenerKind::Provenance] {
            let independent = run(fixture_config(Fixture::Independent, seed, listener)).unwrap();
            let hidden = run(fixture_config(Fixture::HiddenShared, seed, listener)).unwrap();
            let a: Vec<_> = independent
                .updates
                .iter()
                .filter(|u| u.input.id == 5)
                .collect();
            let b: Vec<_> = hidden.updates.iter().filter(|u| u.input.id == 5).collect();
            for (a, b) in a.iter().zip(&b) {
                assert_eq!(a.input, b.input);
                assert_eq!(a.assessment, b.assessment);
            }
            assert_eq!(a.last().unwrap().observer_received_roots.len(), 3);
            assert_eq!(b.last().unwrap().observer_received_roots.len(), 1);
            assert_eq!(
                judgment(&independent, 5).choice,
                judgment(&hidden, 5).choice
            );
            for update in &hidden.updates {
                let json = serde_json::to_value(&update.input).unwrap();
                assert_eq!(json.as_object().unwrap().len(), 4);
                for secret in [
                    "actor",
                    "lineage",
                    "observer_received_roots",
                    "config",
                    "testimony",
                ] {
                    assert!(json.get(secret).is_none());
                }
                for message in &update.input.messages {
                    let json = serde_json::to_value(message).unwrap();
                    assert_eq!(json.as_object().unwrap().len(), 4);
                    for secret in [
                        "basis",
                        "parents",
                        "roots",
                        "depth",
                        "strategy",
                        "classification",
                        "matches_truth",
                        "recipients",
                    ] {
                        assert!(json.get(secret).is_none());
                    }
                }
            }
        }
    }
}
#[test]
fn relay_requires_receipt_exact_copy_scoping_and_legal_reference() {
    let r = run(fixture_config(
        Fixture::OpaqueShared,
        0,
        ListenerKind::Provenance,
    ))
    .unwrap();
    let target = &judgment(&r, 5).input;
    assert!(!target.messages.iter().any(|m| m.id == 0));
    let mut action = Action::Speak {
        speech: Speech::Assert(1),
        attribution: Attribution::Reference(0),
        basis: Basis::Received(0),
        recipients: vec![3],
    };
    assert!(validate_action(target, &action).is_err());
    let source = &r.decisions[1].input;
    assert!(validate_action(source, &action).is_ok());
    if let Action::Speak { speech, .. } = &mut action {
        *speech = Speech::Assert(2);
    }
    assert!(validate_action(source, &action).is_err());
    if let Action::Speak {
        speech, recipients, ..
    } = &mut action
    {
        *speech = Speech::Assert(1);
        *recipients = vec![0];
    }
    assert!(validate_action(source, &action).is_err());
    if let Action::Speak {
        recipients,
        attribution,
        ..
    } = &mut action
    {
        *recipients = vec![3];
        *attribution = Attribution::Reference(99);
    }
    assert!(validate_action(source, &action).is_err());
    let no_message = run(fixture_config(
        Fixture::NoMessage,
        0,
        ListenerKind::Provenance,
    ))
    .unwrap();
    assert!(no_message.testimony.is_empty());
    let disagrees = run(fixture_config(
        Fixture::Disagreement,
        0,
        ListenerKind::Provenance,
    ))
    .unwrap();
    assert!(
        disagrees
            .decisions
            .iter()
            .filter(|d| d.input.id == 1 && d.input.phase == Phase::Discussion)
            .all(|d| d.choice.action == Action::Silence && !d.input.messages.is_empty())
    );
}
fn injection(speaker: u8, parent: Option<usize>, p: u8) -> Injection {
    Injection {
        speaker,
        choice: Choice {
            action: Action::Speak {
                speech: Speech::Assert(p),
                attribution: parent
                    .map(Attribution::Reference)
                    .unwrap_or(Attribution::Firsthand),
                basis: parent
                    .map(Basis::Received)
                    .unwrap_or(Basis::PrivateObservation),
                recipients: (0..6).collect(),
            },
            strategy: parent
                .map(|_| Strategy::RelayReceived)
                .unwrap_or(Strategy::Baseline(Intent::ReportBelief)),
            scores: vec![],
            mechanism: "explicit validation injection".into(),
        },
    }
}
#[test]
fn actual_chain_depth_and_inaccessible_injected_parents_are_validated() {
    let mut config = Config::new(0);
    config.observations[0] = Some(1);
    config.injections = (0..4)
        .map(|id| {
            injection(
                id,
                if id == 0 {
                    None
                } else {
                    Some(usize::from(id) - 1)
                },
                1,
            )
        })
        .collect();
    let r = run(config.clone()).unwrap();
    assert_eq!(r.testimony[3].lineage.depth, 3);
    assert_eq!(r.testimony[3].lineage.roots, vec![Root::Observation(0)]);
    config.injections.push(injection(4, Some(3), 1));
    assert!(run(config).unwrap_err().contains("depth"));
    let mut inaccessible = Config::new(0);
    inaccessible.observations[0] = Some(1);
    inaccessible.injections = vec![injection(0, None, 1), injection(1, Some(0), 1)];
    if let Action::Speak { recipients, .. } = &mut inaccessible.injections[0].choice.action {
        *recipients = vec![0];
    }
    assert!(run(inaccessible).unwrap_err().contains("not received"));
}
#[test]
fn false_origins_propagate_as_faithful_relays_and_change_votes() {
    for seed in 0..16 {
        for fixture in [Fixture::MistakenOrigin, Fixture::DeceptiveOrigin] {
            let b = run(fixture_config(fixture, seed, ListenerKind::SourceCount)).unwrap();
            let a = run(fixture_config(fixture, seed, ListenerKind::Provenance)).unwrap();
            assert_eq!(
                b.testimony[0].classification,
                if fixture == Fixture::MistakenOrigin {
                    CommunicationClass::SincereMistake
                } else {
                    CommunicationClass::IntentionalContradiction
                }
            );
            assert!(
                b.testimony[1..]
                    .iter()
                    .all(|t| t.classification == CommunicationClass::FaithfulRelay
                        && t.matches_truth == Some(false))
            );
            assert!(
                b.testimony[1..]
                    .iter()
                    .all(|t| matches!(t.origin, Origin::Selected { .. }))
            );
            assert_ne!(judgment(&b, 5).choice.action, judgment(&a, 5).choice.action);
            assert_eq!(b.correct, Some(false));
            assert_eq!(a.config.actor(), b.config.actor());
            assert!(a.updates.iter().all(|u| {
                u.input
                    .messages
                    .iter()
                    .all(|m| m.speech == Speech::Assert((a.config.actor() + 1) % 6))
            }));
        }
    }
}
#[test]
fn chronology_lineage_and_decision_inputs_remain_traceable_and_immutable() {
    for fixture in [
        Fixture::KnownShared,
        Fixture::Chain,
        Fixture::OpaqueShared,
        Fixture::InjectedShared,
    ] {
        let r = run(fixture_config(fixture, 0, ListenerKind::Provenance)).unwrap();
        let mut received = vec![vec![]; 6];
        let mut originals = vec![None; 6];
        let mut current = vec![Belief::default(); 6];
        let mut selected = BTreeSet::new();
        for event in &r.objective_history {
            match event {
                Event::Assigned {
                    agent,
                    original,
                    initial,
                } => {
                    originals[usize::from(*agent)] = Some(original.clone());
                    current[usize::from(*agent)] = initial.clone();
                }
                Event::DecisionMade { decision } => {
                    selected.insert(*decision);
                }
                Event::Delivered { message, recipient } => {
                    let t = &r.testimony[*message];
                    assert!(t.recipients.contains(recipient));
                    if let Origin::Selected { decision } = t.origin {
                        assert!(selected.contains(&decision));
                        let d = &r.decisions[decision];
                        assert_eq!(t.speaker_belief, d.input.current.candidate);
                        for &parent in &t.lineage.parents {
                            assert!(d.input.messages.iter().any(|m| m.id == parent));
                        }
                    }
                    assert!(
                        t.lineage
                            .parents
                            .iter()
                            .all(|&parent| parent < t.message.id)
                    );
                    for &parent in &t.lineage.parents {
                        assert!(
                            r.testimony[parent]
                                .lineage
                                .roots
                                .iter()
                                .all(|root| t.lineage.roots.contains(root))
                        );
                    }
                    received[usize::from(*recipient)].push(t.message.clone());
                }
                Event::Updated { update } => {
                    let u = &r.updates[*update];
                    let id = usize::from(u.input.id);
                    assert_eq!(u.input.messages, received[id]);
                    assert_eq!(u.input.original, *originals[id].as_ref().unwrap());
                    assert_eq!(u.input.prior, current[id]);
                    assert!(u.assessment.sources.iter().all(|s| {
                        u.input.messages.iter().any(|m| m.id == s.message)
                            && s.checked_messages
                                .iter()
                                .all(|id| u.input.messages.iter().any(|m| m.id == *id))
                    }));
                    current[id] = u.assessment.after.clone();
                }
                Event::Resolved {
                    decision, accepted, ..
                } => {
                    let d = &r.decisions[*decision];
                    assert_eq!(*accepted, d.rejection.is_none());
                    assert_eq!(d.input.current, r.updates[d.update].assessment.after);
                    assert_eq!(d.input.messages, r.updates[d.update].input.messages);
                }
                _ => {}
            }
        }
        assert_eq!(r.decisions.len(), 18);
        assert_eq!(r.updates.len(), 18);
        assert_eq!(
            received.iter().map(Vec::len).sum::<usize>(),
            r.retained_message_copies
        );
        if fixture == Fixture::InjectedShared {
            assert!(
                r.testimony
                    .iter()
                    .all(|t| matches!(t.origin, Origin::Injected { .. }))
            );
            assert!(
                r.decisions[..12]
                    .iter()
                    .all(|d| d.choice.action == Action::Silence)
            );
        }
    }
}
use std::collections::BTreeSet;
#[test]
fn primary_family_contracts_and_byte_replay_pass_before_freezing() {
    let e = evaluate().unwrap();
    let frozen = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/evidence/experiment-004.json"
    ))
    .unwrap();
    assert_eq!(serde_json::to_string_pretty(&e).unwrap(), frozen);
    assert_eq!(e.runs, 416);
    assert_eq!(e.replay_matches, 416);
    assert_eq!(e.listener_pairs.len(), 208);
    assert_eq!(e.hidden_origin_controls.len(), 32);
    assert_eq!(e.attribution_pairs.len(), 32);
    assert!(
        e.fixtures
            .iter()
            .all(|f| f.counts.rejected_decisions == 0 && f.counts.max_depth <= 3)
    );
    assert!(
        e.listener_pairs
            .iter()
            .all(|p| p.only_listeners_differ && p.identical_delivered_messages)
    );
    assert!(
        e.hidden_origin_controls
            .iter()
            .all(|c| c.identical_target_update_inputs
                && c.identical_target_assessments
                && c.identical_target_votes
                && c.independent_actual_chains == 3
                && c.hidden_actual_chains == 1)
    );
    for p in &e.listener_pairs {
        if matches!(
            p.fixture,
            Fixture::KnownShared
                | Fixture::Chain
                | Fixture::InjectedShared
                | Fixture::MistakenOrigin
                | Fixture::DeceptiveOrigin
        ) {
            assert_eq!(p.baseline_groups, 3);
            assert_eq!(p.provenance_groups, 1);
            assert!(p.target_scores_changed && p.target_vote_changed);
        }
        if matches!(
            p.fixture,
            Fixture::Independent
                | Fixture::HiddenShared
                | Fixture::UnspecifiedShared
                | Fixture::NoMessage
        ) {
            assert!(!p.target_scores_changed && !p.target_vote_changed);
        }
    }
    assert!(e.attribution_pairs.iter().all(|p| p.matched_initial_states
        && p.same_speakers_and_propositions
        && p.actual_roots_unchanged
        && p.target_groups_changed));
}

struct Illegal;
impl CommunicationPolicy for Illegal {
    fn choose(&mut self, _: &AgentView) -> Choice {
        Choice {
            action: Action::Speak {
                speech: Speech::Assert(99),
                attribution: Attribution::Firsthand,
                basis: Basis::PrivateObservation,
                recipients: vec![0],
            },
            strategy: Strategy::Baseline(Intent::ReportBelief),
            scores: vec![],
            mechanism: "invalid test action".into(),
        }
    }
}
impl JudgmentPolicy for Illegal {
    fn choose(&mut self, _: &AgentView) -> Choice {
        Choice {
            action: Action::Vote(Some(99)),
            strategy: Strategy::Baseline(Intent::NoCommunication),
            scores: vec![],
            mechanism: "invalid ballot".into(),
        }
    }
}
#[test]
fn rejected_actions_terminate_without_effect_and_configs_are_validated() {
    let config = fixture_config(Fixture::NoMessage, 0, ListenerKind::Provenance);
    let mut policies = Policies::from_config(&config);
    policies.speakers = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn CommunicationPolicy>)
        .collect();
    policies.judges = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn JudgmentPolicy>)
        .collect();
    let r = run_with(config.clone(), &mut policies).unwrap();
    assert_eq!(r.decisions.len(), 18);
    assert!(
        r.decisions
            .iter()
            .all(|d| d.classification == CommunicationClass::Rejected)
    );
    assert!(r.testimony.is_empty());
    assert_eq!(r.tally, [0; 6]);
    assert_eq!(r.verdict, None);
    let phases: Vec<_> = r
        .objective_history
        .iter()
        .filter_map(|e| {
            if let Event::Transition(p) = e {
                Some(*p)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        phases,
        [
            Phase::Incident,
            Phase::Observation,
            Phase::Discussion,
            Phase::Judgment,
            Phase::Terminated
        ]
    );
    let mut bad = config.clone();
    bad.version = "unsupported".into();
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.order = [0; 6];
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.routes[1] = vec![1, 1];
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.speakers[1] = SpeakerKind::Relay {
        source: 99,
        presentation: Presentation::Disclose,
    };
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.injections = vec![injection(0, None, 1); 13];
    assert!(run(bad).is_err());
    policies.listeners.pop();
    assert!(run_with(config, &mut policies).is_err());
}
#[test]
fn incorrect_attribution_is_not_corrected_using_actual_roots() {
    let r = run(fixture_config(
        Fixture::WrongAttribution,
        0,
        ListenerKind::Provenance,
    ))
    .unwrap();
    let target = judgment(&r, 5);
    let update = &r.updates[target.update];
    assert_eq!(update.observer_received_roots, vec![Root::Observation(0)]);
    assert_eq!(
        update.assessment.perceived_groups,
        vec![Group::Speaker(0), Group::Speaker(3)]
    );
    assert_eq!(target.input.current.candidate, None);
    assert_eq!(target.choice.action, Action::Vote(None));
    assert!(
        r.testimony[1..]
            .iter()
            .all(|t| t.attribution_compatible == Some(false)
                && t.classification == CommunicationClass::FaithfulRelay)
    );
    assert!(update.assessment.sources.iter().all(|s| !s.origin_verified));
    let mut self_view = unknown();
    self_view.id = 0;
    self_view.messages = vec![
        message(0, 0, Attribution::Firsthand),
        message(1, 1, Attribution::Reference(0)),
    ];
    let own = ListenerKind::Provenance.assess(&self_view);
    assert_eq!(own.after.candidate, None);
    assert_eq!(own.sources[1].treatment, Treatment::OwnOriginIgnored);
}
#[test]
fn cli_example_matches_library_and_cannot_overwrite_existing_files() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/experiments/004/known-shared.json"
    );
    let config: Config =
        serde_json::from_str(include_str!("../experiments/004/known-shared.json")).unwrap();
    assert_eq!(
        config,
        fixture_config(Fixture::KnownShared, 0, ListenerKind::Provenance)
    );
    let expected = run(config).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_experiment004"))
        .args(["run", path])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Report>(&output.stdout).unwrap(),
        expected
    );
    let attempted = std::process::Command::new(env!("CARGO_BIN_EXE_experiment004"))
        .args(["run", path, path])
        .output()
        .unwrap();
    assert!(!attempted.status.success());
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        include_str!("../experiments/004/known-shared.json")
    );
}
