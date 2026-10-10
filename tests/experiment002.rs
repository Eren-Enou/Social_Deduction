use social_deduction::{Phase, experiment002::*};

fn first(report: &Report, agent: u8) -> &Decision {
    &report.decisions[usize::from(agent)]
}

#[test]
fn participation_is_private_and_overrides_only_own_belief() {
    for seed in 0..32 {
        let actor = actor_for_seed(seed);
        let r = run(fixture_config(Fixture::Truthful, seed)).unwrap();
        for id in 0..6 {
            let own = &first(&r, id).input.own;
            assert_eq!(own.briefing.is_some(), id == actor);
            let observation = match id {
                0 | 1 => Some(actor),
                2 => Some((actor + 1) % 6),
                _ => None,
            };
            assert_eq!(own.observation, observation);
            assert_eq!(own.belief, if id == actor { Some(id) } else { observation });
        }
        let disabled = run(fixture_config(Fixture::NoBriefing, seed)).unwrap();
        assert!(
            disabled
                .decisions
                .iter()
                .all(|d| d.input.own.briefing.is_none())
        );
    }
}
#[test]
fn other_private_incentives_cannot_change_local_view() {
    let before = fixture_config(Fixture::Silent, 0);
    let mut after = before.clone();
    after.incentives[4] = Incentives::BLAME;
    let b = run(before).unwrap();
    let a = run(after).unwrap();
    for (bd, ad) in b.decisions.iter().zip(&a.decisions) {
        if bd.input.id != 4 {
            assert_eq!(bd.input, ad.input);
            assert_eq!(bd.choice, ad.choice);
        }
    }
    for d in &b.decisions {
        let json = serde_json::to_value(&d.input).unwrap();
        assert_eq!(json.as_object().unwrap().len(), 6);
        assert!(json.get("actor").is_none());
        assert!(json.get("config").is_none());
        assert!(json.get("objective_history").is_none());
    }
    let r = run(fixture_config(Fixture::Blame, 0)).unwrap();
    for d in &r.decisions {
        for m in &d.input.messages {
            assert_eq!(m, &r.public_messages[m.id]);
            let json = serde_json::to_value(m).unwrap();
            assert_eq!(json.as_object().unwrap().len(), 4);
            for secret in [
                "belief",
                "intent",
                "briefing",
                "incentives",
                "classification",
                "assertion_matches_truth",
            ] {
                assert!(json.get(secret).is_none());
            }
        }
    }
}
#[test]
fn incentives_change_actor_decisions_without_mandatory_deception() {
    for seed in 0..32 {
        let actor = actor_for_seed(seed);
        let a = run(fixture_config(Fixture::Accuracy, seed)).unwrap();
        let b = run(fixture_config(Fixture::Blame, seed)).unwrap();
        let q = run(fixture_config(Fixture::Quiet, seed)).unwrap();
        assert_eq!(
            first(&a, actor).classification,
            Classification::SincereTruth
        );
        assert_eq!(
            first(&b, actor).classification,
            Classification::IntentionalContradiction
        );
        assert_eq!(first(&q, actor).classification, Classification::Silence);
        assert!(
            first(&a, actor)
                .choice
                .scores
                .iter()
                .any(|s| s.intent == Intent::ContradictBelief)
        );
        assert_eq!(a.tally, b.tally);
        assert_eq!(a.tally, q.tally);
        assert_eq!(a.verdict, b.verdict);
        assert_eq!(a.verdict, q.verdict);
        let targeted = run(fixture_config(Fixture::ActorOnlyBlame, seed)).unwrap();
        for id in 0..6 {
            if id != actor {
                assert_eq!(first(&a, id).choice, first(&targeted, id).choice);
                assert_eq!(first(&a, id).input.own, first(&targeted, id).input.own);
            }
        }
        assert!(a.decisions.iter().all(|d| d.rejection.is_none()));
        assert!(b.decisions.iter().all(|d| d.rejection.is_none()));
    }
}
#[test]
fn classifications_require_belief_and_intent_not_falsehood_alone() {
    let seed = (0..32).find(|&s| actor_for_seed(s) != 2).unwrap();
    let report = run(fixture_config(Fixture::Truthful, seed)).unwrap();
    let witness = first(&report, 2);
    assert_eq!(witness.classification, Classification::SincereMistake);
    assert_eq!(witness.assertion_matches_truth, Some(false));
    let actor = actor_for_seed(seed);
    let mut choice = witness.choice.clone();
    choice.intent = Intent::ContradictBelief;
    assert_eq!(
        classify(&witness.input, &choice, actor, true),
        Classification::Unsubstantiated
    );
    // A deliberate contradiction can happen to be objectively correct.
    choice.action = Action::Speak {
        speech: Speech::Assert(actor),
        response_to: None,
    };
    assert_eq!(
        classify(&witness.input, &choice, actor, true),
        Classification::IntentionalContradiction
    );
    assert_eq!(
        classify(&witness.input, &choice, actor, false),
        Classification::Rejected
    );
    let mut unknown = witness.input.clone();
    unknown.own.belief = None;
    assert_eq!(
        classify(&unknown, &choice, actor, true),
        Classification::Unsubstantiated
    );
}
#[test]
fn utility_is_fully_recorded_and_ties_are_conservative() {
    let r = run(fixture_config(Fixture::Accuracy, 0)).unwrap();
    for d in &r.decisions[..6] {
        assert_eq!(d.choice.scores, options(&d.input));
        let mut best = &d.choice.scores[0];
        for score in &d.choice.scores[1..] {
            if score.score > best.score {
                best = score;
            }
        }
        assert_eq!(d.choice.action, best.action);
        assert_eq!(d.choice.intent, best.intent);
    }
    let mut view = r.decisions[0].input.clone();
    view.own.incentives = Incentives {
        accuracy: 0,
        avoid_blame: 0,
        contradiction_cost: 0,
        speech_cost: 0,
    };
    assert_eq!(
        BaselinePolicy::new(PolicyKind::IncentiveSensitive)
            .choose(&view)
            .action,
        Action::Silence
    );
    view.own.incentives = Incentives {
        accuracy: u32::MAX,
        avoid_blame: u32::MAX,
        contradiction_cost: u32::MAX,
        speech_cost: u32::MAX,
    };
    assert!(
        options(&view)
            .iter()
            .all(|s| s.score >= -3 * i64::from(u32::MAX))
    );
}
struct Fixed(Action);
impl Policy for Fixed {
    fn choose(&mut self, _: &AgentView) -> Choice {
        Choice {
            action: self.0.clone(),
            intent: Intent::NoCommunication,
            scores: vec![],
            mechanism: "test control".into(),
        }
    }
}
fn fixed(action: Action) -> Vec<Box<dyn Policy>> {
    (0..6)
        .map(|_| Box::new(Fixed(action.clone())) as Box<dyn Policy>)
        .collect()
}
#[test]
fn legality_resolution_and_termination_remain_bounded() {
    let config = fixture_config(Fixture::Silent, 0);
    let r = run_with(
        config.clone(),
        &mut fixed(Action::Speak {
            speech: Speech::Assert(99),
            response_to: None,
        }),
    )
    .unwrap();
    assert_eq!(r.decisions.len(), 18);
    assert!(
        r.decisions
            .iter()
            .all(|d| d.classification == Classification::Rejected)
    );
    assert!(r.public_messages.is_empty());
    assert_eq!(r.tally, [0; 6]);
    let mut view = r.decisions[0].input.clone();
    assert!(validate(&view, &Action::Vote(None)).is_err());
    assert!(
        validate(
            &view,
            &Action::Speak {
                speech: Speech::Uncertain,
                response_to: Some(99)
            }
        )
        .is_err()
    );
    view.opportunities_left = 0;
    assert!(validate(&view, &Action::Silence).is_err());
    view.opportunities_left = 1;
    view.phase = Phase::Judgment;
    assert!(validate(&view, &Action::Vote(Some(99))).is_err());
    let voted = run_with(config.clone(), &mut fixed(Action::Vote(Some(3)))).unwrap();
    assert_eq!(voted.tally, [0, 0, 0, 6, 0, 0]);
    assert_eq!(voted.verdict, Some(3));
    assert_eq!(voted.correct, Some(actor_for_seed(0) == 3));
    let phases: Vec<_> = voted
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
    bad.version = "unknown".into();
    assert!(run(bad).is_err());
    assert!(run_with(config, &mut []).is_err());
}
#[test]
fn full_evaluation_replay_controls_and_matched_pairs() {
    let evaluation = evaluate().unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&evaluation).unwrap(),
        include_str!("../evidence/experiment-002.json")
    );
    assert_eq!(evaluation.runs, 256);
    assert_eq!(evaluation.replay_matches, 256);
    for fixture in &evaluation.fixtures {
        assert_eq!(fixture.counts.rejected, 0);
        assert_eq!(fixture.counts.unsubstantiated, 0);
        for report in &fixture.reports {
            assert_eq!(report.decisions.len(), 18);
            assert!(report.public_messages.len() <= 12);
            for d in &report.decisions {
                for m in &d.input.messages {
                    assert_eq!(m, &report.public_messages[m.id]);
                }
            }
            assert!(
                report.decisions[12..]
                    .windows(2)
                    .all(|w| w[0].input.messages == w[1].input.messages)
            );
        }
        if fixture.fixture == Fixture::Silent {
            assert_eq!(fixture.counts.silence, 384);
            assert_eq!(fixture.counts.verdicts, 0);
            assert!(fixture.reports.iter().all(|r| r.public_messages.is_empty()));
        }
    }
    assert_eq!(evaluation.pairs.len(), 96);
    for pair in &evaluation.pairs {
        assert!(
            pair.only_incentives_differ
                && pair.actor_choice_changed
                && pair.ballots_unchanged
                && pair.outcome_unchanged
        );
    }
}

#[test]
fn cli_example_matches_library_and_protects_existing_files() {
    let config_path = concat!(env!("CARGO_MANIFEST_DIR"), "/experiments/002/accuracy.json");
    let config: Config =
        serde_json::from_str(include_str!("../experiments/002/accuracy.json")).unwrap();
    let expected = run(config).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_experiment002"))
        .args(["run", config_path])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Report>(&output.stdout).unwrap(),
        expected
    );
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "socialdeduction-002-{}-{unique}.json",
        std::process::id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    use std::io::Write;
    file.write_all(b"preserve this evidence").unwrap();
    drop(file);
    let attempted = std::process::Command::new(env!("CARGO_BIN_EXE_experiment002"))
        .args(["run", config_path])
        .arg(&path)
        .output()
        .unwrap();
    assert!(!attempted.status.success());
    assert_eq!(std::fs::read(&path).unwrap(), b"preserve this evidence");
    std::fs::remove_file(path).unwrap();
}
#[test]
fn experiment001_frozen_reports_remain_identical() {
    let frozen: serde_json::Value =
        serde_json::from_str(include_str!("../evidence/experiment-001.json")).unwrap();
    let reports = frozen["reports"].as_array().unwrap();
    assert_eq!(reports.len(), 128);
    for expected in reports {
        let config: social_deduction::Config =
            serde_json::from_value(expected["config"].clone()).unwrap();
        assert_eq!(
            serde_json::to_value(social_deduction::run(config)).unwrap(),
            *expected
        );
    }
    // Verify the original CLI's complete serialized evaluation, including summary.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_social-deduction"))
        .arg("evaluate")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim_end(),
        include_str!("../evidence/experiment-001.json")
    );
}
