use social_deduction::*;

fn config(kind: Baseline) -> Config {
    Config {
        seed: 7,
        policies: [kind; 6],
    }
}
#[test]
fn controls_and_termination() {
    for kind in [Baseline::Silent, Baseline::Truthful, Baseline::Random] {
        let r = run(config(kind));
        assert_eq!(r.decisions.len(), 18);
        assert!(r.decisions.iter().all(|d| d.rejection.is_none()));
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
        assert!(r.public_claims.len() <= 12);
        if kind == Baseline::Silent {
            assert!(r.public_claims.is_empty());
            assert_eq!(r.verdict, None);
        }
        if kind == Baseline::Truthful {
            assert_eq!(r.public_claims.len(), 3);
            assert_eq!(r.verdict, None);
            for c in &r.public_claims {
                let d = r
                    .decisions
                    .iter()
                    .find(|d| d.input.id == c.speaker)
                    .unwrap();
                assert_eq!(Some(c.candidate), d.input.belief);
            }
            assert_ne!(r.public_claims[0].candidate, r.public_claims[2].candidate);
        }
    }
}
#[test]
fn deterministic_serialized_replay() {
    for seed in 0..32 {
        let c = Config {
            seed,
            policies: [Baseline::Random; 6],
        };
        assert_eq!(
            serde_json::to_vec(&run(c.clone())).unwrap(),
            serde_json::to_vec(&run(c)).unwrap()
        );
    }
}
#[test]
fn isolation_and_immutable_history() {
    let r = run(config(Baseline::Truthful));
    for d in &r.decisions {
        if d.input.id >= 3 {
            assert_eq!(d.input.observation, None);
            assert_eq!(d.input.belief, None);
        }
        let json = serde_json::to_value(&d.input).unwrap();
        assert!(json.get("actor").is_none());
        assert!(json.get("objective_history").is_none());
        for c in &d.input.claims {
            assert_eq!(c, &r.public_claims[c.id]);
            let json = serde_json::to_value(c).unwrap();
            assert!(json.get("intent").is_none());
            assert!(json.get("belief").is_none());
        }
    }
    assert!(r.decisions[0].input.claims.is_empty());
    assert_eq!(r.decisions[3].input.claims.len(), 3);
    // Ballots never appear in another individual's policy input.
    assert!(
        r.decisions[12..]
            .windows(2)
            .all(|w| w[0].input.claims == w[1].input.claims)
    );
}
struct Fixed(Action);
impl Policy for Fixed {
    fn choose(&mut self, _: &AgentView) -> Choice {
        Choice {
            action: self.0.clone(),
            intent: Intent::Unspecified,
        }
    }
}
fn fixed(action: Action) -> Vec<Box<dyn Policy>> {
    (0..6)
        .map(|_| Box::new(Fixed(action.clone())) as Box<dyn Policy>)
        .collect()
}
#[test]
fn invalid_actions_have_no_effect_and_terminate() {
    let r = run_with(
        config(Baseline::Silent),
        &mut fixed(Action::Claim {
            candidate: 99,
            response_to: None,
        }),
    );
    assert_eq!(
        r.decisions.iter().filter(|d| d.rejection.is_some()).count(),
        18
    );
    assert!(r.public_claims.is_empty());
    assert_eq!(r.verdict, None);
    let view = &run(config(Baseline::Silent)).decisions[0].input;
    assert!(validate(view, &Action::Vote(None)).is_err());
    assert!(
        validate(
            view,
            &Action::Claim {
                candidate: 0,
                response_to: Some(100)
            }
        )
        .is_err()
    );
    let mut exhausted = view.clone();
    exhausted.opportunities_left = 0;
    assert!(validate(&exhausted, &Action::Silence).is_err());
    let mut judgment = view.clone();
    judgment.phase = Phase::Judgment;
    assert!(validate(&judgment, &Action::Vote(Some(99))).is_err());
}
#[test]
fn votes_resolve_without_forced_verdict() {
    assert_eq!(verdict([3, 3, 0, 0, 0, 0]), None);
    assert_eq!(verdict([0; 6]), None);
    assert_eq!(verdict([4, 2, 0, 0, 0, 0]), Some(0));
    assert_eq!(verdict([4, 4, 0, 0, 0, 0]), None);
    let r = run_with(config(Baseline::Silent), &mut fixed(Action::Vote(Some(0))));
    assert_eq!(r.verdict, Some(0));
    let actor = r
        .objective_history
        .iter()
        .find_map(|e| {
            if let Event::Incident { actor } = e {
                Some(*actor)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(r.correct, Some(actor == 0));
    assert_eq!(
        r.decisions.iter().filter(|d| d.rejection.is_some()).count(),
        12
    );
}
#[test]
fn random_can_respond_to_public_claims() {
    let r = run(config(Baseline::Random));
    assert!(r.public_claims.iter().any(|c| c.response_to.is_some()));
    for c in &r.public_claims {
        assert!(c.response_to.is_none_or(|id| id < c.id));
    }
}
