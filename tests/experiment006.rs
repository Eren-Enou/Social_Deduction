use social_deduction::{Phase, experiment005 as e5, experiment006::*};

fn fixture(f: Fixture) -> Report {
    run(fixture_config(f, 0, ListenerKind::Credibility)).unwrap()
}
fn score(r: &Report, observer: usize, source: usize) -> i8 {
    last_episode(r).memory_before[observer].credits[source].score
}
fn unknown(received: Vec<e5::Message>) -> e5::ListenerView {
    e5::ListenerView {
        id: 5,
        original: e5::IndividualState {
            observation: None,
            briefing: None,
            belief: None,
            incentives: e5::Incentives::ACCURACY,
        },
        observations: vec![],
        prior: e5::Belief::default(),
        received,
    }
}
fn assertion(id: usize, speaker: u8, candidate: u8) -> e5::Message {
    e5::Message {
        id,
        speaker,
        body: e5::Body::Assert(candidate),
        attribution: e5::Attribution::Firsthand,
    }
}
fn revision(id: usize, target: usize, candidate: Option<u8>) -> e5::Message {
    e5::Message {
        id,
        speaker: 0,
        body: e5::Body::Revise {
            target,
            replacement: candidate,
            forwarded: None,
        },
        attribution: e5::Attribution::Unspecified,
    }
}
fn experience(messages: Vec<e5::Message>) -> LocalMemory {
    let v = unknown(messages);
    let interpretation = e5::Listener::assess(&mut e5::ListenerKind::Revision, &v);
    ingest(
        &LocalMemory::neutral(5),
        IngestView {
            episode: 0,
            observer: 5,
            received: v.received,
            interpretation,
        },
    )
    .unwrap()
    .after
}
fn verify(memory: LocalMemory, actor: u8) -> FeedbackEffect {
    UpdateKind::Evidence.update(&FeedbackView {
        delivery: 0,
        memory,
        received: Feedback::VerifiedOutcome {
            episode: 0,
            actor,
            issuer: "lab-verification-006-v1".into(),
        },
    })
}

#[test]
fn neutral_support_and_all_revision_standings_match_frozen005() {
    for messages in [
        vec![assertion(0, 0, 1)],
        vec![assertion(0, 0, 1), revision(1, 0, Some(2))],
        vec![assertion(0, 0, 1), revision(1, 0, None)],
        vec![
            assertion(0, 0, 1),
            revision(1, 0, Some(2)),
            revision(2, 0, Some(3)),
        ],
        vec![revision(0, 99, Some(2))],
    ] {
        let v = unknown(messages);
        let frozen = e5::Listener::assess(&mut e5::ListenerKind::Revision, &v);
        let (weighted, weights, unweighted) = ListenerKind::Credibility.assess(CredibilityView {
            incident: &v,
            credits: &[Credit::default(); 6],
        });
        assert_eq!(weighted.after, frozen.after);
        assert_eq!(unweighted, frozen.after);
        assert_eq!(weighted.interpretations, frozen.interpretations);
        assert_eq!(weighted.groups, frozen.groups);
        assert!(weights.iter().all(|w| w.weight == 1));
    }
    let r = fixture(Fixture::Neutral);
    assert_eq!(judgment(&r, 5).choice.action, e5::Action::Vote(None));
}

#[test]
fn scores_are_local_bounded_and_require_received_authentication() {
    for f in [
        Fixture::Neutral,
        Fixture::MissingFeedback,
        Fixture::WrongVerdict,
        Fixture::UnverifiedClaim,
    ] {
        let r = fixture(f);
        assert!(r.final_memory.iter().all(|m| {
            m.credits
                .iter()
                .all(|c| c.score == 0 && c.verified_count() == 0)
        }));
        assert_eq!(judgment(&r, 5).choice.action, e5::Action::Vote(None));
    }
    let r = fixture(Fixture::WrongVerdict);
    assert_eq!(r.episodes[0].incident.correct, Some(false));
    assert!(
        r.feedback
            .iter()
            .all(|f| matches!(f.input.received, Feedback::Unverified { .. }))
    );
    let a = fixture(Fixture::Accurate);
    let i = fixture(Fixture::Inaccurate);
    assert_eq!(score(&a, 5, 0), 2);
    assert_eq!(score(&i, 5, 0), -1);
    assert_eq!(
        judgment(&a, 5).choice.action,
        e5::Action::Vote(Some(last_episode(&a).incident.config.actor()))
    );
    assert_ne!(
        judgment(&i, 5).choice.action,
        e5::Action::Vote(Some(last_episode(&i).incident.config.actor()))
    );
    assert!(
        a.episodes
            .iter()
            .flat_map(|e| &e.cognition)
            .flat_map(|c| &c.weights)
            .all(|w| w.weight <= 3)
    );
}

#[test]
fn different_deliveries_produce_different_observers_of_identical_testimony() {
    for f in [Fixture::DifferentHistories, Fixture::SelectiveFeedback] {
        let r = fixture(f);
        assert!(score(&r, 3, 0) > 0);
        assert!(score(&r, 5, 0) < 0);
        assert_eq!(
            judgment(&r, 3).input.received,
            judgment(&r, 5).input.received
        );
        assert_ne!(judgment(&r, 3).input.current, judgment(&r, 5).input.current);
        assert_ne!(judgment(&r, 3).choice.action, judgment(&r, 5).choice.action);
    }
}

#[test]
fn correction_behavior_original_accuracy_and_final_reliability_are_separate() {
    let r = fixture(Fixture::Corrected);
    let c = last_episode(&r).memory_before[5].credits[0];
    assert_eq!(
        (c.score, c.original_inaccurate, c.accurate, c.corrections),
        (1, 1, 1, 1)
    );
    let misleading = fixture(Fixture::MisleadingCorrection);
    let c = last_episode(&misleading).memory_before[5].credits[0];
    assert_eq!(
        (c.score, c.original_inaccurate, c.inaccurate, c.corrections),
        (-1, 1, 1, 1)
    );
    let uncorrected = fixture(Fixture::Uncorrected);
    assert_eq!(
        last_episode(&uncorrected).memory_before[5].credits[0].corrections,
        0
    );
    assert_eq!(score(&uncorrected, 5, 0), -1);
    for messages in [
        vec![assertion(0, 0, 1), revision(1, 0, None)],
        vec![
            assertion(0, 0, 1),
            revision(1, 0, Some(2)),
            revision(2, 0, Some(3)),
        ],
    ] {
        let memory = experience(messages);
        let e = verify(memory, 2);
        assert_eq!(e.after.credits[0].score, 0);
        assert_eq!(e.after.credits[0].verified_count(), 0);
        assert_eq!(e.after.credits[0].original_inaccurate, 1);
    }
}

#[test]
fn visible_relays_do_not_multiply_source_credit_or_reveal_hidden_origin() {
    let r = fixture(Fixture::RelayOrigin);
    let m = &last_episode(&r).memory_before[5];
    assert_eq!(m.records.len(), 1);
    assert_eq!(m.credits[0].inaccurate, 1);
    assert_eq!(m.credits[1].score, 0);
    assert_eq!(m.credits[2].score, 0);
    assert_eq!(m.credits[1].relay_inaccurate, 1);
    assert_eq!(m.credits[2].relay_inaccurate, 1);
    assert!(r.episodes[0].ingestions[5].duplicate_reports_prevented >= 2);
    for f in [Fixture::OpaqueRelay, Fixture::Misattributed] {
        let r = fixture(f);
        let m = &last_episode(&r).memory_before[5];
        assert!(m.records.iter().all(|r| r.source.is_none()));
        assert!(
            m.credits
                .iter()
                .all(|c| c.score == 0 && c.verified_count() == 0)
        );
        assert!(
            m.records
                .iter()
                .flat_map(|r| &r.received)
                .all(|m| m.speaker != 0)
        );
    }
}

#[test]
fn delayed_and_duplicate_feedback_are_local_idempotent_and_conflict_safe() {
    let delayed = fixture(Fixture::DelayedFeedback);
    let ordinary = fixture(Fixture::Accurate);
    assert_eq!(
        last_episode(&delayed).memory_before[5].credits,
        last_episode(&ordinary).memory_before[5].credits
    );
    assert_eq!(delayed.episodes[0].memory_after[5].credits[0].score, 0);
    let first = verify(experience(vec![assertion(0, 0, 1)]), 1);
    let duplicate = verify(first.after.clone(), 1);
    assert_eq!(duplicate.after, first.after);
    assert_eq!(duplicate.duplicate_verifications, 1);
    let conflict = verify(first.after.clone(), 2);
    assert_eq!(conflict.after, first.after);
    assert_eq!(conflict.integrity_conflicts, 1);
    let memory = experience(vec![assertion(0, 0, 1)]);
    let effect = UpdateKind::Evidence.update(&FeedbackView {
        delivery: 0,
        memory: memory.clone(),
        received: Feedback::VerifiedOutcome {
            episode: 0,
            actor: 1,
            issuer: "participant".into(),
        },
    });
    assert_eq!(effect.after, memory);
    assert_eq!(effect.integrity_conflicts, 1);
}

#[test]
fn verified_reversal_and_fifo_eviction_preserve_only_declared_consequences() {
    let r = fixture(Fixture::Reversal);
    assert_eq!(
        r.episodes[..5]
            .iter()
            .map(|e| e.memory_after[5].credits[0].score)
            .collect::<Vec<_>>(),
        vec![1, 2, 1, 0, -1]
    );
    let r = fixture(Fixture::Eviction);
    assert!(
        r.final_memory
            .iter()
            .all(|m| m.records.len() <= MEMORY_CAPACITY)
    );
    assert!(
        r.episodes[7].ingestions[5]
            .evicted
            .iter()
            .any(|k| k.episode == 0)
    );
    let f = r
        .feedback
        .iter()
        .find(|f| f.recipient == 5 && f.input.received.episode() == 0)
        .unwrap();
    assert!(f.effect.no_retained_experience);
    assert_eq!(f.effect.after, f.input.memory);
    assert_eq!(f.effect.after.credits[0].score, -2);
    assert_eq!(f.effect.after.credits[0].inaccurate, 6);
}

#[test]
fn trusted_falsehood_can_harm_future_judgment_and_never_becomes_truth() {
    let r = fixture(Fixture::TrustedFalse);
    assert_eq!(score(&r, 5, 0), 2);
    assert_eq!(last_episode(&r).incident.correct, Some(false));
    assert_eq!(
        judgment(&r, 5).choice.action,
        e5::Action::Vote(Some((last_episode(&r).incident.config.actor() + 1) % 6))
    );
    assert_eq!(
        r.final_memory[5].credits[0],
        last_episode(&r).memory_before[5].credits[0]
    );
}

#[test]
fn identities_reset_namespaces_and_callback_snapshots_are_consistent() {
    let r = fixture(Fixture::Accurate);
    for (index, e) in r.episodes.iter().enumerate() {
        assert_eq!(usize::from(e.id), index);
        assert_eq!(e.incident.decisions.len(), 18);
        assert_eq!(e.cognition.len(), e.incident.updates.len());
        for (n, (c, u)) in e.cognition.iter().zip(&e.incident.updates).enumerate() {
            assert_eq!(c.update, n);
            assert_eq!(c.agent, u.input.id);
            assert_eq!(
                c.credit_input,
                e.memory_before[usize::from(c.agent)].credits
            );
            assert_eq!(c.weighted, u.assessment.after);
            assert!(u.input.received.iter().all(|m| {
                e.incident
                    .testimony
                    .iter()
                    .any(|t| t.message == *m && t.recipients.contains(&u.input.id))
            }));
            let serialized = serde_json::to_value(&u.input).unwrap();
            assert!(serialized.get("episodes").is_none());
            assert!(serialized.get("objective_history").is_none());
            assert!(serialized.get("credits").is_none());
        }
        for ingestion in &e.ingestions {
            assert!(
                ingestion
                    .after
                    .records
                    .iter()
                    .flat_map(|r| &r.received)
                    .all(|m| m.reference.episode <= e.id)
            );
            assert_eq!(ingestion.input.episode, e.id);
        }
        assert!(e.incident.decisions.iter().all(|d| d.rejection.is_none()));
        let first = &e.incident.decisions[0];
        assert!(first.input.received.is_empty());
        assert!(first.input.sent.is_empty());
        assert_eq!(first.input.phase, Phase::Discussion);
    }
    assert_eq!(r.episodes[0].incident.testimony[0].message.id, 0);
    assert_eq!(r.episodes[1].incident.testimony[0].message.id, 0);
    assert!(r.final_memory[5].records.iter().any(|e| e.key
        == Reference {
            episode: 0,
            local: 0
        }));
    assert!(r.final_memory[5].records.iter().any(|e| e.key
        == Reference {
            episode: 1,
            local: 0
        }));
}

#[test]
fn hidden_history_and_hidden_intent_cannot_change_equal_local_experiences() {
    let c = fixture_config(Fixture::MissingFeedback, 0, ListenerKind::Credibility);
    let a = run(c.clone()).unwrap();
    let mut changed = c;
    changed.incidents[0].seed += 1;
    changed.incidents[0].observations[4] = Some(4);
    changed.incidents[0].judgments[4] = e5::JudgmentKind::Abstain;
    let b = run(changed).unwrap();
    assert_ne!(
        a.episodes[0].incident.decisions,
        b.episodes[0].incident.decisions
    );
    assert_eq!(
        last_episode(&a).memory_before[5],
        last_episode(&b).memory_before[5]
    );
    assert_eq!(judgment(&a, 5), judgment(&b, 5));
    let mut c = fixture_config(Fixture::InjectedExperience, 0, ListenerKind::Credibility);
    let sincere = run(c.clone()).unwrap();
    for episode in &mut c.incidents[..2] {
        episode.injections[0].choice.intent = e5::Intent::Contrary;
    }
    let contrary = run(c).unwrap();
    assert_eq!(sincere.final_memory, contrary.final_memory);
    assert_eq!(judgment(&sincere, 5), judgment(&contrary, 5));
    assert_ne!(
        sincere.episodes[0].incident.testimony,
        contrary.episodes[0].incident.testimony
    );
    let selected = fixture(Fixture::Accurate);
    assert_eq!(
        last_episode(&selected).memory_before[5].credits,
        last_episode(&sincere).memory_before[5].credits
    );
    assert_eq!(judgment(&selected, 5), judgment(&sincere, 5));
}

#[test]
fn explicit_abstention_and_silence_remain_legal_with_positive_credit() {
    let mut c = fixture_config(Fixture::Accurate, 0, ListenerKind::Credibility);
    c.incidents.last_mut().unwrap().judgments = [e5::JudgmentKind::Abstain; 6];
    let r = run(c).unwrap();
    assert_eq!(score(&r, 5, 0), 2);
    assert_eq!(last_episode(&r).incident.verdict, None);
    for d in &last_episode(&r).incident.decisions {
        assert!(d.rejection.is_none());
        if d.input.phase == Phase::Judgment {
            assert_eq!(d.choice.action, e5::Action::Vote(None));
        } else if d.input.id == 5 {
            assert_eq!(d.choice.action, e5::Action::Silence);
        }
    }
}

#[test]
fn cli_runs_json_example_and_refuses_to_overwrite_evidence() {
    let directory =
        std::env::temp_dir().join(format!("social-deduction-006-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let config = directory.join("config.json");
    let output = directory.join("report.json");
    // This test's paths are unique to its process; existing files still fail safely.
    let exe = env!("CARGO_BIN_EXE_experiment006");
    assert!(
        std::process::Command::new(exe)
            .arg("example")
            .arg(&config)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new(exe)
            .arg("run")
            .arg(&config)
            .arg(&output)
            .status()
            .unwrap()
            .success()
    );
    let before = std::fs::read(&output).unwrap();
    let report: Report = serde_json::from_slice(&before).unwrap();
    assert_eq!(report, fixture(Fixture::Accurate));
    let result = std::process::Command::new(exe)
        .arg("run")
        .arg(&config)
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(std::fs::read(&output).unwrap(), before);
    std::fs::remove_file(&config).unwrap();
    std::fs::remove_file(&output).unwrap();
    std::fs::remove_dir(&directory).unwrap();
}

#[test]
fn retain_updater_is_replaceable_and_bad_config_is_rejected() {
    let mut c = fixture_config(Fixture::Accurate, 0, ListenerKind::Credibility);
    c.updater = UpdateKind::Retain;
    let r = run(c.clone()).unwrap();
    assert_eq!(score(&r, 5, 0), 0);
    assert_eq!(judgment(&r, 5).choice.action, e5::Action::Vote(None));
    assert!(
        r.final_memory[5]
            .records
            .iter()
            .any(|r| r.verified.is_some())
    );
    c.incidents.clear();
    assert!(run(c).is_err());
    let c = fixture_config(Fixture::Accurate, 0, ListenerKind::Credibility);
    let mut too_many = c.clone();
    too_many.incidents = vec![c.incidents[0].clone(); 9];
    assert!(run(too_many).is_err());
    let mut bad = c.clone();
    bad.feedback[0].about_episode = 1;
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.feedback[0].recipients = vec![0, 0];
    assert!(run(bad).is_err());
    let mut bad = c;
    bad.feedback[0].recipients = vec![6];
    assert!(run(bad).is_err());
}

#[test]
fn source_identity_and_order_controls_preserve_equivalent_reliance() {
    let a = fixture(Fixture::Accurate);
    let p = fixture(Fixture::IdentityPermuted);
    let o = fixture(Fixture::OrderReversed);
    assert_eq!(score(&a, 5, 0), score(&p, 5, 3));
    assert_eq!(judgment(&a, 5).input.current, judgment(&p, 5).input.current);
    assert_eq!(judgment(&a, 5).input.current, judgment(&o, 5).input.current);
    assert_eq!(judgment(&a, 5).choice.action, judgment(&p, 5).choice.action);
    assert_eq!(judgment(&a, 5).choice.action, judgment(&o, 5).choice.action);
}

#[test]
fn private_anchor_and_own_echo_survive_weighting_without_added_independence() {
    let mut v = unknown(vec![assertion(0, 0, 1), assertion(1, 2, 2)]);
    v.original.observation = Some(2);
    let mut credits = [Credit::default(); 6];
    credits[0].score = 2;
    credits[2].score = -2;
    let (a, _, _) = ListenerKind::Credibility.assess(CredibilityView {
        incident: &v,
        credits: &credits,
    });
    assert_eq!(a.after.scores[2], 2);
    assert_eq!(a.after.scores[1], 3);
    let mut echo = assertion(2, 1, 3);
    echo.attribution = e5::Attribution::NamedSource(5);
    v.received = vec![echo];
    credits[5].score = 2;
    let (a, w, _) = ListenerKind::Credibility.assess(CredibilityView {
        incident: &v,
        credits: &credits,
    });
    assert!(w.is_empty());
    assert_eq!(a.after.scores[3], 0);
}

#[test]
fn primary_contracts_and_every_serialized_series_replay() {
    let e = evaluate().unwrap();
    assert_eq!((e.series, e.incidents, e.replay_matches), (352, 1056, 352));
    assert_eq!(e.fixtures.len(), 44);
    assert_eq!(e.listener_pairs.len(), 176);
    assert_eq!(e.controls.len(), 96);
    assert!(e.fixtures.iter().all(|r| r.reports.len() == 8
        && r.counts.rejected == 0
        && r.counts.max_memory_records <= 8));
    for p in &e.listener_pairs {
        assert!(
            p.only_listener_differs
                && p.identical_current_config
                && p.identical_current_private_inputs
                && p.identical_memory_before
        );
        if p.fixture != Fixture::Interaction {
            assert!(p.identical_current_wire && !p.communication_changed);
        }
        if matches!(
            p.fixture,
            Fixture::Neutral
                | Fixture::MissingFeedback
                | Fixture::WrongVerdict
                | Fixture::OpaqueRelay
                | Fixture::Misattributed
                | Fixture::UnverifiedClaim
        ) {
            assert!(!p.target_support_changed && !p.target_vote_changed && !p.verdict_changed);
        }
    }
    for row in &e.fixtures {
        for r in &row.reports {
            assert_eq!(
                r.objective_history
                    .iter()
                    .filter(|v| matches!(v, Event::Begin { .. }))
                    .count(),
                r.episodes.len()
            );
            assert_eq!(
                r.objective_history
                    .iter()
                    .filter(|v| matches!(v, Event::End { .. }))
                    .count(),
                r.episodes.len()
            );
            for (n, f) in r.feedback.iter().enumerate() {
                assert_eq!(f.input.delivery, n);
                assert_eq!(f.input.memory.observer, f.recipient);
                if let Feedback::VerifiedOutcome { episode, actor, .. } = f.input.received {
                    assert_eq!(
                        actor,
                        r.episodes[usize::from(episode)].incident.config.actor()
                    );
                }
                let position = r
                    .objective_history
                    .iter()
                    .position(|v| *v == Event::FeedbackDelivered { record: n })
                    .unwrap();
                assert_eq!(
                    r.objective_history[position + 1],
                    Event::MemoryUpdated { record: n }
                );
            }
        }
    }
}
