use social_deduction::{experiment007 as e7, experiment010::*};

#[test]
fn scoring_separates_missingness_labels_count_strata_and_voluntary_selection() {
    let mut c = diagnostic_config(
        Diagnostic::AccurateRecurring,
        0,
        PolicyKind::FirstAccessible,
        false,
    );
    c.incidents.truncate(1);
    c.feedback.retain(|f| f.after_episode == 0);
    for a in &mut c.incidents[0].artifacts {
        a.candidate = None;
    }
    let r = run(c).unwrap();
    let fs = forecasts(&r).unwrap();
    assert!(fs.iter().all(|f| f.observer_label.correct.is_none()));
    let s = forecast_score(&fs, PredictorKind::Neutral, false, false, None, None);
    assert_eq!(s.accuracy.samples, 0);
    assert_eq!(s.accuracy.brier(), None);
    assert_eq!(s.missingness.samples, 12);
    assert_eq!(s.missingness.brier(), Some(250_000));
    assert_eq!(
        s.missingness.bins.iter().map(|b| b.samples).sum::<u64>(),
        12
    );
    assert_eq!(
        forecast_score(&fs, PredictorKind::Neutral, true, false, None, None)
            .missingness
            .samples,
        6
    );
    assert_eq!(
        forecast_score(&fs, PredictorKind::Neutral, false, false, None, Some(0)),
        s
    );
    assert_eq!(
        forecast_score(&fs, PredictorKind::Neutral, false, false, None, Some(1))
            .missingness
            .samples,
        0
    );
    let mut changed = r.clone();
    changed.episodes[0].investigation[0]
        .choice
        .mechanism
        .push_str("-changed");
    assert_ne!(
        report_digests(&r).unwrap(),
        report_digests(&changed).unwrap()
    );
    assert_eq!(fs[0].local, forecasts(&changed).unwrap()[0].local);
}
#[test]
fn exact_score_boundaries_and_world_cluster_intervals() {
    let mut s = Score::default();
    s.add(
        Probability {
            numerator: 0,
            denominator: 1,
        },
        false,
    );
    s.add(
        Probability {
            numerator: 1,
            denominator: 1,
        },
        true,
    );
    assert_eq!(s.brier(), Some(0));
    assert_eq!(s.ece(), Some(0));
    assert_eq!(s.bins[0].samples, 1);
    assert_eq!(s.bins[4].samples, 1);
    assert_eq!(paired_interval(&[]).mean, None);
    let i = paired_interval(&[4, 4, 4]);
    assert_eq!(
        (i.mean, i.lower, i.upper, i.worlds),
        (Some(4), Some(4), Some(4), 3)
    );
    assert_eq!(paired_interval(&[-5, 7, 12]), paired_interval(&[-5, 7, 12]));
    assert!(canonical_digest(&serde_json::json!({"bad":0.5})).is_err());
    // Independent Python hashlib + sorted compact JSON reference, before primary evaluation.
    assert_eq!(
        canonical_digest(&serde_json::json!({"z":1,"a":2})).unwrap(),
        "c9edb8663f835d677f410a22d8e4e151a06eb31c0dc3d1e864f42f69f14bdae7"
    );
    assert_eq!(
        canonical_digest(&serde_json::json!({"z":1,"a":2})).unwrap(),
        canonical_digest(&serde_json::json!({"a":2,"z":1})).unwrap()
    );
}
#[test]
fn development_grid_has_full_replay_pairing_audit_and_frozen_control_equivalence() {
    let e = evaluate_seeds(0..2, 0..1).unwrap();
    assert_eq!(e.version, "development-evaluation-010-v1");
    assert_eq!(e.rows.len(), 308);
    assert_eq!(e.incidents, 2416);
    assert_eq!(e.replay_matches, 308);
    assert_eq!(e.independent_worlds, 2);
    assert_eq!(e.comparisons.len(), 252);
    assert_eq!(e.predictions.len(), 14400);
    let m = manifest(&e).unwrap();
    assert_eq!(m.series_sections.len(), 308);
    assert!(m.series_sections.iter().all(|s| s.len() == 7));
    for r in e
        .rows
        .iter()
        .filter(|r| r.population == "heldout" && r.policy == PolicyKind::AdvertisedQuality)
    {
        let b = e
            .rows
            .iter()
            .find(|b| {
                b.population == r.population
                    && b.seed == r.seed
                    && b.stream == r.stream
                    && b.policy == PolicyKind::Frozen009InformationValue
            })
            .unwrap();
        let mut rm = r.metrics.clone();
        let mut bm = b.metrics.clone();
        rm.full_report_bytes = 0;
        bm.full_report_bytes = 0;
        assert_eq!(rm, bm);
        assert_eq!(r.forecasts, b.forecasts);
        for (x, y) in r.trace.iter().zip(&b.trace) {
            assert_eq!(x.deliveries, y.deliveries);
            assert_eq!(x.messages, y.messages);
            assert_eq!(x.ballots, y.ballots);
            assert_eq!(x.learning, y.learning);
            assert_eq!(x.feedback, y.feedback);
            for (a, b) in x.choices.iter().zip(&y.choices) {
                assert_eq!(a.choice.action, b.choice.action);
                assert_eq!(a.choice.considered, b.choice.considered);
            }
        }
        assert_eq!(r.environment_digest, b.environment_digest);
    }
    assert!(
        e.rows
            .iter()
            .all(|r| r.trace.iter().all(|t| t.choices.len() == 6))
    );
    for c in &e.comparisons {
        assert_eq!(c.worlds, c.improved + c.worsened + c.unchanged);
        assert_eq!(c.utilities.len(), 9);
        assert_eq!(c.coverage.worlds, c.worlds);
    }
    assert!(e7::canonical_bytes(&m).is_ok());
}
#[test]
fn matched_training_is_executed_equal_and_fixed_current_controls_change_only_history() {
    for family in DIAGNOSTICS {
        let a = run(diagnostic_config(family, 0, MATCHED_POLICIES[0], true)).unwrap();
        for policy in &MATCHED_POLICIES[1..] {
            let b = run(diagnostic_config(family, 0, *policy, true)).unwrap();
            for (x, y) in a.episodes[..4].iter().zip(&b.episodes[..4]) {
                assert_eq!(x.deliveries, y.deliveries);
                assert_eq!(x.method_after, y.method_after);
                assert_eq!(x.memory_after, y.memory_after);
                assert_eq!(x.incident.testimony, y.incident.testimony);
            }
        }
    }
    for cause in [false, true] {
        for policy in MATCHED_POLICIES {
            let [a, b] = fixed_current_configs(0, policy, cause).map(|c| run(c).unwrap());
            for (x, y) in a.episodes[4]
                .investigation
                .iter()
                .zip(&b.episodes[4].investigation)
            {
                assert_eq!(x.input.local, y.input.local);
                assert_eq!(x.input.methods, y.input.methods);
                assert_ne!(x.input.experience, y.input.experience);
            }
            assert_eq!(a.config.incidents[4], b.config.incidents[4]);
        }
    }
}

#[test]
fn sufficient_executed_contradictory_experience_can_reverse_a_local_preference() {
    let mut c = diagnostic_config(
        Diagnostic::EqualQuality,
        0,
        PolicyKind::NeverInvestigate,
        false,
    );
    c.policy[5] = PolicyKind::ExperienceAdjusted;
    for (episode, i) in c.incidents.iter_mut().enumerate() {
        let actor = i.game.actor();
        i.artifacts[0].candidate = Some(if episode == 0 { actor } else { (actor + 1) % 6 });
        if episode < 6 {
            i.forced.push((
                5,
                InvestigationAction::Inspect(i.artifacts[0].descriptor.id),
            ));
        }
    }
    c.processes[0].accuracy_before = 4;
    c.processes[0].accuracy_after = 0;
    c.processes[0].shift_episode = Some(1);
    let r = run(c.clone()).unwrap();
    let first = &r.episodes[1].investigation[5];
    assert_eq!(
        first.proposal.as_ref().unwrap().action,
        InvestigationAction::Inspect(c.incidents[1].artifacts[0].descriptor.id)
    );
    let later = &r.episodes[6].investigation[5];
    assert!(!later.forced);
    assert_eq!(
        later.choice.action,
        InvestigationAction::Inspect(c.incidents[6].artifacts[1].descriptor.id)
    );
    assert_eq!(later.input.experience[&c.processes[0].family].success, 1);
    assert_eq!(later.input.experience[&c.processes[0].family].failure, 5);
    c.feedback.clear();
    let unverified = run(c).unwrap();
    assert_eq!(
        unverified.episodes[6].investigation[5].choice.action,
        InvestigationAction::Inspect(unverified.config.incidents[6].artifacts[0].descriptor.id)
    );
    assert!(
        unverified.final_method_memory[5]
            .summaries
            .values()
            .all(|s| s.graded() == 0)
    );
}
