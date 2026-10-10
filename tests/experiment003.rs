use social_deduction::{Phase, experiment003::*};

fn unknown_view() -> ListenerView {
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
fn message(id: usize, speaker: u8, speech: Speech) -> Message {
    Message {
        id,
        speaker,
        speech,
        response_to: None,
    }
}

#[test]
fn listener_accepts_conflicts_ignores_repeats_and_retains_private_evidence() {
    let mut view = unknown_view();
    let initial = ListenerKind::Evidence.assess(&view);
    assert_eq!(initial.after.candidate, None);
    view.messages.push(message(0, 0, Speech::Assert(1)));
    let one = ListenerKind::Evidence.assess(&view);
    assert_eq!(one.after.candidate, Some(1));
    assert_eq!(one.after.margin, 1);
    assert_eq!(one.messages[0].handling, Handling::AddsEvidence);
    view.prior = one.after.clone();
    view.messages.push(message(1, 0, Speech::Assert(1)));
    view.messages.push(message(2, 0, Speech::Assert(2)));
    let repeated = ListenerKind::Evidence.assess(&view);
    assert_eq!(repeated.after, one.after);
    assert!(
        repeated.messages[1..]
            .iter()
            .all(|m| m.handling == Handling::RepeatIgnored)
    );
    view.messages.push(message(3, 1, Speech::Assert(2)));
    let conflict = ListenerKind::Evidence.assess(&view);
    assert_eq!(conflict.after.candidate, None);
    assert_eq!(conflict.after.margin, 0);
    assert_eq!(conflict.messages[3].handling, Handling::ConflictsPrior);
    view.messages.push(message(4, 2, Speech::Uncertain));
    view.messages.push(message(5, 5, Speech::Assert(3)));
    let ignored = ListenerKind::Evidence.assess(&view);
    assert_eq!(ignored.after, conflict.after);
    assert_eq!(ignored.messages[4].handling, Handling::UncertaintyIgnored);
    assert_eq!(ignored.messages[5].handling, Handling::SelfIgnored);
    // Uncertainty does not reserve a speaker's first assertion.
    view.messages.push(message(6, 2, Speech::Assert(1)));
    assert_eq!(
        ListenerKind::Evidence.assess(&view).after.candidate,
        Some(1)
    );
    let retain = ListenerKind::Retain.assess(&view);
    assert_eq!(retain.after, view.prior);
    assert!(
        retain
            .messages
            .iter()
            .all(|m| m.handling == Handling::RetainPolicyIgnored)
    );
    let mut private = unknown_view();
    private.original.observation = Some(3);
    private.original.belief = Some(3);
    private.prior = initial_belief(5, &private.original);
    private.messages.push(message(0, 0, Speech::Assert(1)));
    let retained = ListenerKind::Evidence.assess(&private);
    assert_eq!(retained.after.candidate, Some(3));
    assert_eq!(retained.after.margin, 1);
    private.original.briefing = Some(Briefing::Participated);
    private.original.belief = Some(5);
    private.messages = (0..5)
        .map(|id| message(usize::from(id), id, Speech::Assert(1)))
        .collect();
    assert_eq!(
        ListenerKind::Evidence.assess(&private).after.candidate,
        Some(5)
    );
}
#[test]
fn testimony_changes_executed_votes_and_can_cause_wrong_verdicts() {
    for seed in 0..16 {
        let absent = run(fixture_config(Fixture::Absent, seed)).unwrap();
        assert_eq!(judgment(&absent, 5).choice.action, Action::Vote(None));
        for fixture in [Fixture::OneTruth, Fixture::OneMistake, Fixture::OneLie] {
            let r = run(fixture_config(fixture, seed)).unwrap();
            let actor = r.config.actor();
            let expected = if fixture == Fixture::OneTruth {
                actor
            } else {
                (actor + 1) % 6
            };
            assert_eq!(judgment(&r, 5).input.current.candidate, Some(expected));
            assert_eq!(judgment(&r, 5).choice.action, Action::Vote(Some(expected)));
            assert_eq!(r.verdict, Some(expected));
            assert_eq!(r.correct, Some(fixture == Fixture::OneTruth));
            assert!(
                r.updates
                    .iter()
                    .any(|u| u.input.id == 5 && u.candidate_changed)
            );
            assert!(matches!(r.testimony[0].origin, Origin::Intervention { .. }));
            assert_eq!(
                r.testimony[0].classification,
                match fixture {
                    Fixture::OneTruth => Classification::SincereTruth,
                    Fixture::OneMistake => Classification::SincereMistake,
                    _ => Classification::IntentionalContradiction,
                }
            );
            // Scripted assertions are absent from individual decision choices.
            assert!(
                r.decisions[..12]
                    .iter()
                    .all(|d| d.choice.action == Action::Silence)
            );
        }
        for fixture in [Fixture::Conflict, Fixture::Uncertain] {
            let r = run(fixture_config(fixture, seed)).unwrap();
            assert_eq!(r.verdict, None);
            assert_eq!(judgment(&r, 5).choice.action, Action::Vote(None));
        }
        let truth = run(fixture_config(Fixture::OneTruth, seed)).unwrap();
        let repeat = run(fixture_config(Fixture::Repeat, seed)).unwrap();
        assert_eq!(
            judgment(&truth, 5).input.current,
            judgment(&repeat, 5).input.current
        );
        assert_eq!(truth.tally, repeat.tally);
    }
}
#[test]
fn hidden_intentions_cannot_change_identical_listener_inputs() {
    for seed in 0..16 {
        let truth = run(fixture_config(Fixture::HiddenTruth, seed)).unwrap();
        let lie = run(fixture_config(Fixture::HiddenContradiction, seed)).unwrap();
        assert_eq!(truth.testimony[0].message, lie.testimony[0].message);
        assert_eq!(
            truth.testimony[0].classification,
            Classification::SincereTruth
        );
        assert_eq!(
            lie.testimony[0].classification,
            Classification::IntentionalContradiction
        );
        assert_ne!(
            truth.testimony[0].speaker_belief,
            lie.testimony[0].speaker_belief
        );
        for (a, b) in truth.updates.iter().zip(&lie.updates) {
            if a.input.id == 5 {
                assert_eq!(a.input, b.input);
                assert_eq!(a.assessment, b.assessment);
            }
        }
        assert_eq!(judgment(&truth, 5).choice, judgment(&lie, 5).choice);
    }
}
#[test]
fn typed_views_and_private_inputs_are_isolated() {
    let config = fixture_config(Fixture::Absent, 1);
    let mut changed = config.clone();
    changed.incentives[0] = Incentives::BLAME;
    changed.observation_override.as_mut().unwrap()[0] = Some(4);
    let a = run(config).unwrap();
    let b = run(changed).unwrap();
    for (ad, bd) in a.decisions.iter().zip(&b.decisions) {
        if ad.input.id == 5 {
            assert_eq!(ad.input, bd.input);
            assert_eq!(ad.choice, bd.choice);
        }
    }
    let r = run(fixture_config(Fixture::OneLie, 0)).unwrap();
    for u in &r.updates {
        let input = serde_json::to_value(&u.input).unwrap();
        assert_eq!(input.as_object().unwrap().len(), 4);
        for hidden in [
            "actor",
            "config",
            "testimony",
            "objective_history",
            "classification",
        ] {
            assert!(input.get(hidden).is_none());
        }
        for m in &u.input.messages {
            let public = serde_json::to_value(m).unwrap();
            assert_eq!(public.as_object().unwrap().len(), 4);
            for hidden in [
                "intent",
                "speaker_belief",
                "origin",
                "recipients",
                "classification",
                "assertion_matches_truth",
            ] {
                assert!(public.get(hidden).is_none());
            }
        }
    }
}
#[test]
fn trace_records_preserve_observations_history_and_delivery_causality() {
    for fixture in [
        Fixture::Truth,
        Fixture::ReverseOrder,
        Fixture::OneLie,
        Fixture::Conflict,
    ] {
        let r = run(fixture_config(fixture, 3)).unwrap();
        let mut delivered = vec![vec![]; 6];
        let mut prior = vec![Belief::default(); 6];
        let mut originals = vec![None; 6];
        let mut decision_count = 0;
        let mut update_count = 0;
        for event in &r.objective_history {
            match event {
                Event::Assigned {
                    agent,
                    original,
                    initial,
                    ..
                } => {
                    originals[usize::from(*agent)] = Some(original.clone());
                    prior[usize::from(*agent)] = initial.clone();
                }
                Event::Delivered { message, recipient } => {
                    delivered[usize::from(*recipient)].push(r.testimony[*message].message.clone())
                }
                Event::Updated { update } => {
                    assert_eq!(*update, update_count);
                    update_count += 1;
                    let u = &r.updates[*update];
                    let id = usize::from(u.input.id);
                    assert_eq!(u.input.original, *originals[id].as_ref().unwrap());
                    assert_eq!(u.input.prior, prior[id]);
                    assert_eq!(u.input.messages, delivered[id]);
                    assert!(
                        u.assessment.messages.iter().all(|a| u
                            .input
                            .messages
                            .iter()
                            .any(|m| m.id == a.message_id))
                    );
                    assert_eq!(ListenerKind::Evidence.assess(&u.input), u.assessment);
                    prior[id] = u.assessment.after.clone();
                }
                Event::Resolved {
                    decision,
                    accepted,
                    message,
                    ballot,
                } => {
                    assert_eq!(*decision, decision_count);
                    decision_count += 1;
                    let d = &r.decisions[*decision];
                    let u = &r.updates[d.update];
                    assert_eq!(d.input.current, u.assessment.after);
                    assert_eq!(d.input.original, u.input.original);
                    assert_eq!(d.input.messages, u.input.messages);
                    assert_eq!(*accepted, d.rejection.is_none());
                    if let Some(mid) = message {
                        assert_eq!(
                            r.testimony[*mid].origin,
                            Origin::Selected {
                                decision: *decision
                            }
                        );
                        assert_eq!(r.testimony[*mid].speaker_belief, d.input.current.candidate);
                    }
                    if let Some(vote) = ballot {
                        assert_eq!(d.choice.action, Action::Vote(Some(*vote)));
                    }
                }
                _ => {}
            }
        }
        assert_eq!(decision_count, 18);
        assert_eq!(update_count, 18);
        assert!(
            r.updates
                .iter()
                .any(|u| u.input.prior != u.assessment.after)
        );
        let phases: Vec<_> = r
            .objective_history
            .iter()
            .filter_map(|event| {
                if let Event::Transition(phase) = event {
                    Some(*phase)
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
    }
}
#[test]
fn belief_revision_is_connected_to_replaceable_judgment() {
    let revised = run(fixture_config(Fixture::Truth, 0)).unwrap();
    let original = run(fixture_config(Fixture::OriginalVote, 0)).unwrap();
    assert_eq!(revised.updates, original.updates);
    assert_eq!(revised.testimony, original.testimony);
    assert!(
        revised.decisions[12..]
            .iter()
            .zip(&original.decisions[12..])
            .any(|(a, b)| a.choice.action != b.choice.action)
    );
    let mut config = fixture_config(Fixture::OneTruth, 0);
    config.judgments = [JudgmentKind::Abstain; 6];
    let abstaining = run(config).unwrap();
    assert_eq!(abstaining.tally, [0; 6]);
    assert_eq!(abstaining.verdict, None);
}
struct Illegal;
impl CommunicationPolicy for Illegal {
    fn choose(&mut self, _: &DecisionView) -> Choice {
        Choice {
            action: Action::Speak {
                speech: Speech::Assert(99),
                response_to: None,
            },
            intent: Intent::ReportBelief,
            scores: vec![],
            mechanism: "illegal test control".into(),
        }
    }
}
impl JudgmentPolicy for Illegal {
    fn choose(&mut self, _: &DecisionView) -> Choice {
        Choice {
            action: Action::Vote(Some(99)),
            intent: Intent::NoCommunication,
            scores: vec![],
            mechanism: "illegal test control".into(),
        }
    }
}
#[test]
fn invalid_actions_configs_and_references_cannot_affect_resolution() {
    let config = fixture_config(Fixture::Absent, 0);
    let mut policies = Policies::from_config(&config);
    policies.speakers = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn CommunicationPolicy>)
        .collect();
    policies.judges = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn JudgmentPolicy>)
        .collect();
    let report = run_with(config.clone(), &mut policies).unwrap();
    assert_eq!(report.decisions.len(), 18);
    assert!(report.decisions.iter().all(|d| d.rejection.is_some()));
    assert!(report.testimony.is_empty());
    assert_eq!(report.tally, [0; 6]);
    assert_eq!(report.verdict, None);
    let view = report.decisions[0].input.communication_view();
    assert!(social_deduction::experiment002::validate(&view, &Action::Vote(None)).is_err());
    assert!(
        social_deduction::experiment002::validate(
            &view,
            &Action::Speak {
                speech: Speech::Uncertain,
                response_to: Some(99)
            }
        )
        .is_err()
    );
    let mut bad = config.clone();
    bad.layout = [0; 6];
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.speaking_order = [0; 6];
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.version = "bad".into();
    assert!(run(bad).is_err());
    let mut bad = config.clone();
    bad.observation_override = Some([Some(99); 6]);
    assert!(run(bad).is_err());
    let mut bad = fixture_config(Fixture::OneTruth, 0);
    bad.injections[0].recipients = vec![5, 5];
    assert!(run(bad).is_err());
    policies.listeners.pop();
    assert!(run_with(config, &mut policies).is_err());
}
#[test]
fn permutations_are_explicit_and_deterministic() {
    for seed in 0..16 {
        for fixture in [Fixture::Truth, Fixture::Relabeled, Fixture::ReverseOrder] {
            let config = fixture_config(fixture, seed);
            let r = run(config.clone()).unwrap();
            assert_eq!(
                serde_json::to_vec(&r).unwrap(),
                serde_json::to_vec(&run(config).unwrap()).unwrap()
            );
            assert_eq!(
                r.decisions[..6]
                    .iter()
                    .map(|d| d.input.id)
                    .collect::<Vec<_>>(),
                r.config.speaking_order
            );
            assert!(r.decisions.iter().all(|d| d.rejection.is_none()));
        }
    }
}
#[test]
fn cli_example_matches_library_and_refuses_existing_output() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/experiments/003/truth.json");
    let config: Config =
        serde_json::from_str(include_str!("../experiments/003/truth.json")).unwrap();
    let expected = run(config).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_experiment003"))
        .args(["run", path])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Report>(&output.stdout).unwrap(),
        expected
    );
    let attempted = std::process::Command::new(env!("CARGO_BIN_EXE_experiment003"))
        .args(["run", path, path])
        .output()
        .unwrap();
    assert!(!attempted.status.success());
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        include_str!("../experiments/003/truth.json")
    );
}

#[test]
fn selected_mistaken_and_deceptive_speech_influences_without_injection() {
    for seed in 0..16 {
        for kind in [PolicyKind::Truthful, PolicyKind::DeceptiveControl] {
            let mut config = fixture_config(Fixture::Absent, seed);
            let actor = config.actor();
            let wrong = (actor + 1) % 6;
            config.observation_override = Some([
                Some(if kind == PolicyKind::Truthful {
                    wrong
                } else {
                    actor
                }),
                None,
                None,
                None,
                None,
                None,
            ]);
            config.communication[0] = kind;
            let r = run(config).unwrap();
            assert_eq!(r.testimony.len(), 1);
            assert!(matches!(r.testimony[0].origin, Origin::Selected { .. }));
            assert_eq!(
                r.testimony[0].classification,
                if kind == PolicyKind::Truthful {
                    Classification::SincereMistake
                } else {
                    Classification::IntentionalContradiction
                }
            );
            assert_eq!(judgment(&r, 5).input.current.candidate, Some(wrong));
            assert_eq!(judgment(&r, 5).choice.action, Action::Vote(Some(wrong)));
            assert_eq!(r.correct, Some(false));
            if kind == PolicyKind::Truthful {
                assert!(
                    r.decisions
                        .iter()
                        .all(|d| d.choice.intent != Intent::ContradictBelief)
                );
            }
        }
    }
}

#[test]
fn primary_evaluation_matches_frozen_evidence_and_counterfactual_contracts() {
    let evaluation = evaluate().unwrap();
    let frozen = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/evidence/experiment-003.json"
    ))
    .unwrap();
    assert_eq!(serde_json::to_string_pretty(&evaluation).unwrap(), frozen);
    assert_eq!(evaluation.runs, 272);
    assert_eq!(evaluation.replay_matches, 272);
    assert_eq!(evaluation.pairs.len(), 176);
    for fixture in &evaluation.fixtures {
        assert_eq!(fixture.counts.rejected_decisions, 0);
        assert_eq!(
            fixture.counts.candidate_changes + fixture.counts.unchanged_candidates,
            288
        );
        for report in &fixture.reports {
            assert_eq!(report.updates.len(), 18);
            assert_eq!(report.decisions.len(), 18);
        }
    }
    for pair in &evaluation.pairs {
        assert!(pair.matched_original_state);
        if pair.control == Fixture::HiddenTruth {
            assert!(pair.identical_final_listener_inputs);
            assert!(
                !pair.candidate_changed
                    && !pair.vote_changed
                    && !pair.scores_changed
                    && !pair.verdict_changed
            );
        }
        if pair.control == Fixture::OneTruth {
            assert!(
                !pair.candidate_changed
                    && !pair.scores_changed
                    && !pair.margin_changed
                    && !pair.vote_changed
                    && !pair.verdict_changed
            );
        }
        if pair.control == Fixture::Absent {
            match pair.treatment {
                Fixture::Conflict => {
                    assert!(!pair.candidate_changed && !pair.vote_changed && !pair.verdict_changed);
                    assert!(pair.scores_changed);
                }
                Fixture::Uncertain => assert!(
                    !pair.candidate_changed
                        && !pair.scores_changed
                        && !pair.vote_changed
                        && !pair.verdict_changed
                ),
                _ => assert!(pair.candidate_changed && pair.vote_changed && pair.verdict_changed),
            }
        }
    }
    assert!(
        evaluation
            .relabel_checks
            .iter()
            .all(|c| c.beliefs_equivariant && c.votes_equivariant && c.verdict_equivariant)
    );
}
