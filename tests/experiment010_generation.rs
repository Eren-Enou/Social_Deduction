use social_deduction::{experiment007 as e7, experiment010::*};

#[test]
fn development_worlds_are_fixed_before_treatment_and_method_artifacts_recur() {
    for seed in 0..8 {
        let a = Generator.generate(seed);
        assert_eq!(a, Generator.generate(seed));
        a.base.validate().unwrap();
        assert_eq!(a.base.incidents.len(), 8);
        let c = generated_config(&a, PolicyKind::ExperienceAdjusted, FeedbackStream::Clean);
        let b = generated_config(&a, PolicyKind::FirstAccessible, FeedbackStream::MixedFault);
        assert_eq!(c.incidents, b.incidents);
        assert_eq!(c.processes, b.processes);
        assert_eq!(c.budgets, b.budgets);
        assert!((2..=4).contains(&c.processes.len()));
        for p in &c.processes {
            assert_eq!(p.shift_episode, (!seed.is_multiple_of(2)).then_some(4));
            assert!(
                c.incidents
                    .iter()
                    .all(|i| i.methods.iter().any(|(_, m)| *m == p.family))
            );
        }
        let r = run(c).unwrap();
        for (episode, e) in r.episodes.iter().enumerate() {
            for d in &e.deliveries {
                assert_eq!(d.finding.acquisition.episode, episode as u8);
            }
        }
    }
}
#[test]
fn all_diagnostic_and_matched_training_configs_resolve_legally() {
    for family in DIAGNOSTICS {
        for kind in POLICIES {
            for matched in [false, true] {
                let c = diagnostic_config(family, 0, kind, matched);
                c.validate().unwrap();
                let r = run(c).unwrap();
                assert_eq!(r.episodes.len(), 8);
                for l in &r.ledgers {
                    assert_eq!(l.initial, l.remaining + l.spent);
                }
                if matched {
                    assert!(
                        r.episodes[..4]
                            .iter()
                            .flat_map(|e| &e.investigation)
                            .all(|d| d.forced)
                    );
                    assert!(
                        r.episodes[4..]
                            .iter()
                            .flat_map(|e| &e.investigation)
                            .all(|d| !d.forced)
                    );
                }
                for m in &r.final_method_memory {
                    m.validate().unwrap();
                }
            }
        }
    }
}
#[test]
fn missing_and_false_verification_affect_only_legitimate_history() {
    let missing = run(diagnostic_config(
        Diagnostic::MissingFeedback,
        0,
        PolicyKind::FirstAccessible,
        false,
    ))
    .unwrap();
    assert!(
        missing
            .final_method_memory
            .iter()
            .all(|m| m.summaries.values().all(|c| c.graded() == 0))
    );
    let bad = run(diagnostic_config(
        Diagnostic::AuthenticatedFalse,
        0,
        PolicyKind::FirstAccessible,
        false,
    ))
    .unwrap();
    assert!(
        bad.episodes
            .iter()
            .flat_map(|e| &e.method_feedback)
            .all(|f| f.correct == Some(false))
    );
    assert!(
        bad.final_method_memory
            .iter()
            .any(|m| m.summaries.values().any(|c| c.graded() > 0))
    );
    let conflict = run(diagnostic_config(
        Diagnostic::ContradictoryVerification,
        0,
        PolicyKind::FirstAccessible,
        false,
    ))
    .unwrap();
    assert!(
        conflict
            .episodes
            .iter()
            .flat_map(|e| &e.method_feedback)
            .any(|f| f.effect.transport_effect.acceptance == e7::Acceptance::Quarantined)
    );
    assert!(
        conflict
            .final_method_memory
            .iter()
            .all(|m| m.summaries.values().all(|c| c.graded() == 0))
    );
}
#[test]
fn shared_visible_origin_discounts_learning_while_hidden_origin_is_unavailable() {
    let known = run(diagnostic_config(
        Diagnostic::KnownDependent,
        0,
        PolicyKind::FirstAccessible,
        false,
    ))
    .unwrap();
    let hidden = run(diagnostic_config(
        Diagnostic::HiddenDependent,
        0,
        PolicyKind::FirstAccessible,
        false,
    ))
    .unwrap();
    let k = &known.final_method_memory[5];
    let h = &hidden.final_method_memory[5];
    assert!(
        k.summaries.values().map(|c| c.observed()).sum::<u32>()
            < h.summaries.values().map(|c| c.observed()).sum::<u32>()
    );
    assert!(
        known
            .episodes
            .iter()
            .flat_map(|e| &e.method_ingestions)
            .any(|i| i.duplicate)
    );
    // Hidden roots are observer configuration only, absent from local updater and policy types.
    for i in hidden.episodes.iter().flat_map(|e| &e.method_ingestions) {
        let value = serde_json::to_string(&i.input).unwrap();
        assert!(!value.contains("actual_root"));
    }
}
#[test]
fn regime_parameters_change_future_draws_and_preserve_prior_artifacts() {
    let stationary = Generator.generate(2);
    let changed = Generator.generate(3);
    assert!(
        stationary
            .base
            .processes
            .iter()
            .all(|p| p.accuracy(0) == p.accuracy(7))
    );
    assert!(
        changed
            .base
            .processes
            .iter()
            .all(|p| p.accuracy(0) != p.accuracy(7))
    );
    let r = run(generated_config(
        &changed,
        PolicyKind::ExploratoryLearner,
        FeedbackStream::Clean,
    ))
    .unwrap();
    let earlier = r.episodes[..4].to_vec();
    let encoded = serde_json::to_vec(&r).unwrap();
    let restored: Report = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(restored.episodes[..4], earlier);
}

#[test]
fn executed_matched_histories_change_choices_with_identical_current_inputs() {
    for corrupt in [false, true] {
        let [a, b] = fixed_current_configs(0, PolicyKind::ExperienceAdjusted, corrupt);
        assert_eq!(a.incidents[4], b.incidents[4]);
        if corrupt {
            assert_eq!(a.incidents, b.incidents);
        }
        let a = run(a).unwrap();
        let b = run(b).unwrap();
        let a = a.episodes[4]
            .investigation
            .iter()
            .find(|d| d.input.local.id == 5)
            .unwrap();
        let b = b.episodes[4]
            .investigation
            .iter()
            .find(|d| d.input.local.id == 5)
            .unwrap();
        assert_eq!(a.input.local, b.input.local);
        assert_eq!(a.input.methods, b.input.methods);
        assert_ne!(a.input.experience, b.input.experience);
        assert_ne!(a.choice.action, b.choice.action);
        assert!(!a.forced);
        assert!(!b.forced);
    }
}
