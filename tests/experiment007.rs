use social_deduction::{Phase, experiment005 as e5, experiment006 as e6, experiment007::*};
fn fixture(d: Diagnostic, p: VerificationKind) -> Report {
    run(diagnostic_config(d, 0, p, e6::ListenerKind::Credibility)).unwrap()
}
fn last(r: &Report) -> &Episode {
    r.episodes.last().unwrap()
}
fn score(r: &Report, observer: usize, source: usize) -> i8 {
    last(r).memory_before[observer].local.credits[source].score
}

#[test]
fn authenticated_falsehood_rewards_errors_and_penalizes_truth_without_omniscience() {
    for p in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
        let reward = fixture(Diagnostic::RewardFalse, p);
        assert_eq!(score(&reward, 5, 0), 1);
        assert_eq!(last(&reward).incident.correct, Some(false));
        let f = reward.feedback.iter().find(|f| f.recipient == 5).unwrap();
        assert!(f.authentic);
        assert_eq!(f.factual_accuracy, Some(false));
        assert_eq!(f.effect.acceptance, Acceptance::Accepted);
        let penalized = fixture(Diagnostic::PenalizeTrue, p);
        assert_eq!(score(&penalized, 5, 0), -1);
        assert_eq!(last(&penalized).incident.correct, Some(false));
        let message = &last(&reward).incident.testimony[0];
        assert_eq!(message.intent, e5::Intent::Sincere);
    }
}
#[test]
fn authentication_binding_and_announcements_remain_separate_from_factual_accuracy() {
    for d in [
        Diagnostic::Unauthenticated,
        Diagnostic::ReplayWrongSession,
        Diagnostic::UnknownEpisode,
        Diagnostic::Verdict,
        Diagnostic::Missing,
    ] {
        let r = fixture(d, VerificationKind::ConflictAware);
        assert_eq!(score(&r, 5, 0), 0, "{d:?}");
        assert!(
            r.feedback
                .iter()
                .filter(|f| f.recipient == 5)
                .all(|f| matches!(
                    f.effect.acceptance,
                    Acceptance::RejectedAuthentication
                        | Acceptance::RejectedAssociation
                        | Acceptance::Pending
                        | Acceptance::Unverified
                ))
        );
    }
    let baseline = fixture(Diagnostic::ReplayWrongSession, VerificationKind::Frozen006);
    assert_eq!(score(&baseline, 5, 0), 1);
    let r = fixture(Diagnostic::Misassociated, VerificationKind::ConflictAware);
    assert_eq!(score(&r, 5, 0), 1);
    assert!(
        r.feedback
            .iter()
            .filter(|f| f.plan == 0)
            .all(|f| f.effect.acceptance == Acceptance::RejectedAssociation
                && f.effect.after.local.credits == f.input.memory.local.credits)
    );
    assert!(
        r.feedback
            .iter()
            .filter(|f| f.plan == 1 && f.recipient == 5)
            .all(|f| f.effect.acceptance == Acceptance::Accepted)
    );
}
#[test]
fn contradiction_quarantines_retained_evidence_in_either_truth_order() {
    for d in [
        Diagnostic::ConflictTrueFirst,
        Diagnostic::ConflictFalseFirst,
    ] {
        let r = fixture(d, VerificationKind::ConflictAware);
        assert_eq!(score(&r, 5, 0), 0);
        let f = r.feedback.iter().rfind(|f| f.recipient == 5).unwrap();
        assert_eq!(f.effect.acceptance, Acceptance::Quarantined);
        assert_eq!(f.effect.after.local.credits[0].verified_count(), 0);
        assert!(matches!(
            f.effect.after.receipts[0].state,
            VerificationState::Quarantined { .. }
        ));
        let mut follow = f.input.clone();
        follow.memory = f.effect.after.clone();
        assert_eq!(
            VerificationKind::ConflictAware.accept(&follow).acceptance,
            Acceptance::Quarantined
        );
    }
    assert_eq!(
        score(
            &fixture(Diagnostic::ConflictTrueFirst, VerificationKind::Frozen006),
            5,
            0
        ),
        1
    );
    assert_eq!(
        score(
            &fixture(Diagnostic::ConflictFalseFirst, VerificationKind::Frozen006),
            5,
            0
        ),
        -1
    );
}
#[test]
fn delayed_contradiction_changes_future_state_and_preserves_past_decisions() {
    let c = diagnostic_config(
        Diagnostic::DelayedContradiction,
        0,
        VerificationKind::ConflictAware,
        e6::ListenerKind::Credibility,
    );
    let with = run(c.clone()).unwrap();
    let mut without = c;
    without.feedback.pop();
    let without = run(without).unwrap();
    assert_eq!(with.episodes[1].incident, without.episodes[1].incident);
    assert_eq!(with.episodes[1].incident.correct, Some(false));
    assert_eq!(with.episodes[2].memory_before[5].local.credits[0].score, 0);
    assert_eq!(
        without.episodes[2].memory_before[5].local.credits[0].score,
        1
    );
    assert_ne!(
        last(&with).incident.decisions,
        last(&without).incident.decisions
    );
}
#[test]
fn bounded_eviction_cannot_reconstruct_or_retract_forgotten_accepted_detail() {
    let r = fixture(Diagnostic::EvictedConflict, VerificationKind::ConflictAware);
    let f = r
        .feedback
        .iter()
        .find(|f| f.after_episode == 7 && f.recipient == 5)
        .unwrap();
    assert_eq!(f.effect.acceptance, Acceptance::Pending);
    assert!(f.effect.unverifiable_eviction);
    assert_eq!(f.effect.after.local.credits, f.input.memory.local.credits);
    assert!(
        f.input
            .memory
            .local
            .records
            .iter()
            .all(|r| r.key.episode != 0)
    );
    assert!(
        r.episodes[7].ingestions[5]
            .evicted
            .iter()
            .any(|r| r.episode == 0)
    );
    assert!(
        r.final_memory
            .iter()
            .all(|m| m.local.records.len() <= 8 && m.tokens.len() <= 32 && m.receipts.len() <= 8)
    );
}
#[test]
fn repeats_cannot_manufacture_independent_verifications() {
    for p in [VerificationKind::Frozen006, VerificationKind::ConflictAware] {
        for d in [Diagnostic::Duplicate, Diagnostic::Repeated] {
            let r = fixture(d, p);
            let c = last(&r).memory_before[5].local.credits[0];
            assert_eq!((c.score, c.accurate, c.original_accurate), (1, 1, 1));
            assert_eq!(
                r.feedback
                    .iter()
                    .rfind(|f| f.recipient == 5)
                    .unwrap()
                    .effect
                    .acceptance,
                Acceptance::Repeated
            );
        }
    }
}
#[test]
fn frozen_acceptance_calls_preserved006_mechanism() {
    let original = e6::run(e6::fixture_config(
        e6::Fixture::Accurate,
        0,
        e6::ListenerKind::Credibility,
    ))
    .unwrap();
    let wrapped = fixture(Diagnostic::Accurate, VerificationKind::Frozen006);
    for (a, b) in original.episodes.iter().zip(&wrapped.episodes) {
        for (old, new) in a.memory_after.iter().zip(&b.memory_after) {
            assert_eq!(old, &new.local);
        }
        assert_eq!(a.incident.decisions, b.incident.decisions);
        assert_eq!(a.cognition, b.cognition);
    }
}
#[test]
fn equal_local_inputs_ignore_hidden_truth_corruption_and_intent_labels() {
    let c = diagnostic_config(
        Diagnostic::PenalizeTrue,
        0,
        VerificationKind::ConflictAware,
        e6::ListenerKind::Credibility,
    );
    let a = run(c.clone()).unwrap();
    let mut changed = c;
    changed.incidents[0].seed += 1;
    changed.feedback[0].fault = "different hidden diagnostic label".into();
    let b = run(changed).unwrap();
    assert_ne!(a.feedback[0].actual_actor, b.feedback[0].actual_actor);
    for (x, y) in a.feedback.iter().zip(&b.feedback) {
        assert_eq!(x.input, y.input);
        assert_eq!(x.effect, y.effect);
    }
    assert_eq!(last(&a).incident.decisions, last(&b).incident.decisions);
    assert_eq!(a.forecasts, b.forecasts);
    let mut c = diagnostic_config(
        Diagnostic::Missing,
        0,
        VerificationKind::ConflictAware,
        e6::ListenerKind::Credibility,
    );
    let a = run(c.clone()).unwrap();
    c.incidents[0].seed += 1;
    c.incidents[0].observations[4] = Some(3);
    let b = run(c).unwrap();
    assert_eq!(last(&a).memory_before[5], last(&b).memory_before[5]);
    assert_eq!(
        e5::judgment(&last(&a).incident, 5),
        e5::judgment(&last(&b).incident, 5)
    );
}
#[test]
fn selective_deliveries_and_actual_policy_inputs_are_local_and_chronological() {
    let r = fixture(Diagnostic::Selective, VerificationKind::ConflictAware);
    assert_eq!(score(&r, 3, 0), 2);
    assert_eq!(score(&r, 5, 0), 0);
    assert!(r.feedback.iter().all(|f| f.recipient == 3));
    for e in &r.episodes {
        assert_eq!(e.incident.decisions.len(), 18);
        for (c, u) in e.cognition.iter().zip(&e.incident.updates) {
            assert_eq!(
                c.credit_input,
                e.memory_before[usize::from(c.agent)].local.credits
            );
            assert_eq!(c.weighted, u.assessment.after);
            let json = serde_json::to_value(&u.input).unwrap();
            assert!(json.get("actual_actor").is_none());
            assert!(json.get("factual_accuracy").is_none());
            assert!(json.get("episodes").is_none());
        }
    }
    for (i, f) in r.feedback.iter().enumerate() {
        assert_eq!(f.input.delivery, i);
        let pos = r
            .objective_history
            .iter()
            .position(|e| *e == Event::Delivered(i))
            .unwrap();
        assert_eq!(r.objective_history[pos + 1], Event::VerificationResolved(i));
    }
}
#[test]
fn forecasts_are_constructed_before_future_claim_and_hold_outcomes_out_of_inputs() {
    let c = diagnostic_config(
        Diagnostic::Accurate,
        0,
        VerificationKind::ConflictAware,
        e6::ListenerKind::Credibility,
    );
    let a = run(c.clone()).unwrap();
    let mut changed = c;
    let future = changed.incidents.last_mut().unwrap();
    future.observations[0] = Some((future.actor() + 1) % 6);
    let b = run(changed).unwrap();
    assert_eq!(a.forecasts, b.forecasts);
    assert_ne!(last(&a).incident.testimony, last(&b).incident.testimony);
    for f in &a.forecasts {
        let credit = last(&a).memory_before[5].local.credits[usize::from(f.input.source)];
        assert_eq!(
            f.input,
            PredictionInput::from_credit(f.input.source, credit)
        );
        assert!(Probability::new(f.frequency.numerator, f.frequency.denominator).is_ok());
    }
}
#[test]
fn rational_losses_bins_and_empty_denominators_are_correct() {
    let half = Probability::new(1, 2).unwrap();
    assert_eq!(half.brier(true), 250000);
    assert_eq!(half.brier(false), 250000);
    assert_eq!(half.log_loss(true), 693147);
    assert_eq!(Probability::new(1, 4).unwrap().log_loss(true), 1386294);
    assert_eq!(Probability::new(3, 4).unwrap().log_loss(true), 287682);
    assert_eq!(Probability::new(0, 1).unwrap().log_loss(true), 6931471);
    assert_eq!(Probability::new(1, 1).unwrap().brier(false), 1000000);
    assert!(Probability::new(1, 0).is_err());
    assert!(Probability::new(2, 1).is_err());
    for (n, bin) in [(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 4)] {
        assert_eq!(Probability::new(n, 5).unwrap().bin(), bin);
    }
    let mut c = Calibration::default();
    assert_eq!(c.ece(), None);
    c.add(half, true);
    c.add(half, false);
    assert_eq!(c.ece(), Some(0));
    assert_eq!(c.brier(), Some(250000));
    assert_eq!(c.bins[2].samples, 2);
    let stats = Reliability::default();
    assert_eq!(stats.metrics()["vote_accuracy"], None);
}
#[test]
fn generator_varies_real_inputs_and_keeps_development_disjoint() {
    let mut actors = std::collections::BTreeSet::new();
    let mut orders = std::collections::BTreeSet::new();
    let mut counts = std::collections::BTreeSet::new();
    let mut histories = std::collections::BTreeSet::new();
    let mut regimes = std::collections::BTreeSet::new();
    for seed in 0..16 {
        let w = Generator.generate(seed);
        assert_eq!(w, Generator.generate(seed));
        let c = generated_config(
            &w,
            Fault::Clean,
            VerificationKind::ConflictAware,
            e6::ListenerKind::Credibility,
        );
        c.validate().unwrap();
        assert!(!w.sources.contains(&w.target));
    }
    for seed in 1000..1064 {
        let w = Generator.generate(seed);
        actors.insert(w.incidents.last().unwrap().actor());
        orders.insert(w.incidents[0].order);
        counts.insert(w.sources.len());
        histories.insert(w.incidents.len());
        regimes.insert(w.changed);
    }
    assert_eq!(actors.len(), 6);
    assert!(orders.len() > 16);
    assert_eq!(counts.len(), 3);
    assert_eq!(histories.len(), 5);
    assert_eq!(regimes.len(), 2);
}
#[test]
fn canonical_bytes_preserve_order_reject_floats_and_use_standard_sha256() {
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        canonical_bytes(&serde_json::json!({"b":2,"a":1})).unwrap(),
        b"{\"a\":1,\"b\":2}"
    );
    assert!(canonical_bytes(&serde_json::json!({"x":0.5})).is_err());
    assert_ne!(
        canonical_digest(&[1, 2]).unwrap(),
        canonical_digest(&[2, 1]).unwrap()
    );
    let r = fixture(Diagnostic::Accurate, VerificationKind::ConflictAware);
    assert_eq!(
        report_digests(&r).unwrap(),
        report_digests(&fixture(
            Diagnostic::Accurate,
            VerificationKind::ConflictAware
        ))
        .unwrap()
    );
}
#[test]
fn arbitrary_bad_configs_and_updater_outputs_are_rejected() {
    let c = diagnostic_config(
        Diagnostic::Accurate,
        0,
        VerificationKind::ConflictAware,
        e6::ListenerKind::Credibility,
    );
    let mut bad = c.clone();
    bad.feedback[0].recipients = vec![6];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.feedback = vec![c.feedback[0].clone(); 33];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.incidents = vec![c.incidents[0].clone(); 9];
    assert!(run(bad).is_err());
    struct WrongObserver;
    impl VerificationPolicy for WrongObserver {
        fn accept(&mut self, v: &VerificationView) -> VerificationEffect {
            let mut e = VerificationKind::Frozen006.accept(v);
            e.after.local.observer = 6;
            e
        }
    }
    let mut policies: Vec<Box<dyn VerificationPolicy>> = (0..6)
        .map(|_| Box::new(WrongObserver) as Box<dyn VerificationPolicy>)
        .collect();
    assert!(run_with(c, &mut policies, &mut FrequencyPredictor).is_err());
}
#[test]
fn legal_silence_abstention_and_full_series_snapshots_remain_bounded() {
    let mut c = diagnostic_config(
        Diagnostic::Accurate,
        0,
        VerificationKind::ConflictAware,
        e6::ListenerKind::Credibility,
    );
    c.incidents.last_mut().unwrap().judgments = [e5::JudgmentKind::Abstain; 6];
    let r = run(c).unwrap();
    assert_eq!(last(&r).incident.verdict, None);
    assert_eq!(score(&r, 5, 0), 2);
    for d in &last(&r).incident.decisions {
        assert!(d.rejection.is_none());
        if d.input.phase == Phase::Judgment {
            assert_eq!(d.choice.action, e5::Action::Vote(None));
        } else if d.input.id == 5 {
            assert_eq!(d.choice.action, e5::Action::Silence);
        }
    }
}

#[test]
fn comparator_checks_complete_manifests_and_environment_identity() {
    // Synthetic metadata tests the comparator, not actual Linux execution.
    let mut a = PlatformReplay {
        execution: Execution {
            os: "windows".into(),
            architecture: "x86_64".into(),
            rustc: "rustc 1.99.0".into(),
            cargo: "cargo 1.99.0".into(),
            revision: "test-revision".into(),
            lock_digest: "test-lock".into(),
        },
        manifest: ReplayManifest {
            version: "replay-manifest-007-v1".into(),
            series: 1,
            incidents: 1,
            evaluation_digest: "test-evaluation".into(),
            metric_digest: "test-metrics".into(),
            series_sections: vec![
                report_digests(&fixture(
                    Diagnostic::Accurate,
                    VerificationKind::ConflictAware,
                ))
                .unwrap(),
            ],
        },
    };
    let mut b = a.clone();
    assert!(compare_platforms(&a, &b).is_err());
    b.execution.os = "linux".into();
    compare_platforms(&a, &b).unwrap();
    b.manifest.series_sections[0].insert("feedback".into(), "changed".into());
    assert!(compare_platforms(&a, &b).is_err());
    b.manifest = a.manifest.clone();
    a.execution.revision = "another-revision".into();
    assert!(compare_platforms(&a, &b).is_err());
}

#[test]
fn cli_creates_reproducible_reports_and_refuses_overwriting() {
    let directory =
        std::env::temp_dir().join(format!("social-deduction-007-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let config = directory.join("config.json");
    let report = directory.join("report.json");
    let binary = env!("CARGO_BIN_EXE_experiment007");
    let execute = |args: &[&std::ffi::OsStr]| {
        std::process::Command::new(binary)
            .args(args)
            .output()
            .unwrap()
    };
    assert!(
        execute(&["example".as_ref(), config.as_os_str()])
            .status
            .success()
    );
    assert!(
        execute(&["run".as_ref(), config.as_os_str(), report.as_os_str()])
            .status
            .success()
    );
    let bytes = std::fs::read(&report).unwrap();
    let parsed: Report = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        bytes,
        serde_json::to_vec(&run(parsed.config).unwrap()).unwrap()
    );
    assert!(
        !execute(&["run".as_ref(), config.as_os_str(), report.as_os_str()])
            .status
            .success()
    );
    assert_eq!(bytes, std::fs::read(&report).unwrap());
    let generated = execute(&[
        "scenario".as_ref(),
        "generated".as_ref(),
        "Clean".as_ref(),
        "1000".as_ref(),
        "ConflictAware".as_ref(),
        "Credibility".as_ref(),
    ]);
    assert!(generated.status.success());
    let config_from_cli: Config = serde_json::from_slice(&generated.stdout).unwrap();
    assert_eq!(
        config_from_cli,
        generated_config(
            &Generator.generate(1000),
            Fault::Clean,
            VerificationKind::ConflictAware,
            e6::ListenerKind::Credibility
        )
    );
    std::fs::remove_file(config).unwrap();
    std::fs::remove_file(report).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn primary_evaluation_replays_all_worlds_and_preserves_matched_inputs() {
    let e = evaluate().unwrap();
    assert_eq!(e.rows.len(), 1792);
    assert_eq!(e.replay_matches, 1792);
    assert_eq!(e.groups.len(), 88);
    assert_eq!(e.comparisons.len(), 44);
    assert!(e.rows.iter().all(|r| r.rejected == 0));
    for c in &e.comparisons {
        assert_eq!(c.worlds, 64);
        assert_eq!(c.identical_current_wire, 64);
        assert_eq!(c.identical_current_private_inputs, 64);
    }
    assert_eq!(
        serde_json::to_vec(&e).unwrap(),
        std::fs::read("evidence/experiment-007.json").unwrap()
    );
    let platform: PlatformReplay =
        serde_json::from_slice(&std::fs::read("evidence/experiment-007-windows.json").unwrap())
            .unwrap();
    assert_eq!(platform.execution.os, "windows");
    assert_eq!(
        platform.manifest,
        ReplayManifest::from_evaluation(&e).unwrap()
    );
}
