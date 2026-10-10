use social_deduction::{experiment007 as e7, experiment010 as e10, experiment011::*};
use std::collections::BTreeMap;
fn config(f: Diagnostic, k: SocialKind) -> Config {
    diagnostic(f, 0, k)
}
fn message(slot: u8, speaker: u8, rating: Rating) -> Message {
    Message {
        version: "method-report-011-v1".into(),
        reference: Reference { episode: 0, slot },
        speaker,
        payload: Payload {
            family: MethodId::new("instrument-b"),
            rating,
            counts: Some(Counts {
                success: 100,
                nonmissing: 100,
                ..Counts::default()
            }),
            basis: ClaimedBasis::Summary,
            claimed_origin: Some(speaker),
            parent: None,
            acquisition: None,
            verification_claimed: true,
        },
        revision: None,
        retract: false,
    }
}
fn social(messages: Vec<Message>, personal: Counts, kind: SocialKind) -> SocialEstimate {
    kind.clone().assess(&SocialView {
        observer: 5,
        family: MethodId::new("instrument-b"),
        personal,
        memory: SocialMemory {
            received: messages,
            ..SocialMemory::neutral(5)
        },
        credits: [social_deduction::experiment006::Credit::default(); 6],
    })
}
#[test]
fn reports_are_bounded_typed_and_versioned() {
    let mut m = message(0, 0, Rating::Favorable);
    m.payload.validate().unwrap();
    m.payload.counts.as_mut().unwrap().success = 101;
    assert!(m.payload.validate().is_err());
    let mut memory = SocialMemory::neutral(5);
    m.payload.counts.as_mut().unwrap().success = 1;
    m.version = "other".into();
    memory.received.push(m);
    assert!(memory.validate().is_err());
    let mut c = config(Diagnostic::AccurateFirsthand, SocialKind::DirectOnly);
    c.version = "old".into();
    assert!(run(c).is_err());
}
#[test]
fn sincere_reports_capture_only_legitimately_retained_counts() {
    let r = run(config(
        Diagnostic::AccurateFirsthand,
        SocialKind::DirectOnly,
    ))
    .unwrap();
    for d in r
        .episodes
        .iter()
        .flat_map(|e| &e.social_decisions)
        .filter(|d| d.message.is_some() && d.choice.intent == MethodIntent::Sincere)
    {
        let m = d.message.as_ref().unwrap();
        let counts = d.input.personal.summaries[&m.payload.family];
        assert_eq!(m.payload.rating, rating(counts));
        assert_eq!(m.payload.counts, Some(counts));
        assert_eq!(d.input.personal.observer, m.speaker);
    }
    assert!(
        r.episodes[2]
            .social_decisions
            .iter()
            .any(|d| d.message.is_some())
    );
    assert!(
        r.episodes[0]
            .social_decisions
            .iter()
            .all(|d| d.message.is_none())
    );
}
#[test]
fn wire_has_no_private_intent_root_or_feedback_token() {
    let r = run(config(
        Diagnostic::DeliberatelyMisleading,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let d = r.episodes[2]
        .social_decisions
        .iter()
        .find(|d| d.message.is_some())
        .unwrap();
    assert_eq!(d.choice.intent, MethodIntent::Contrary);
    let wire = serde_json::to_value(d.message.as_ref().unwrap()).unwrap();
    for name in [
        "intent",
        "actual_origins",
        "personal",
        "token",
        "actual_actor",
    ] {
        assert!(wire.get(name).is_none());
    }
}
#[test]
fn claims_and_reported_verification_never_add_personal_samples() {
    let r = run(config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let d = r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert!(d.input.experience.is_empty());
    assert!(!d.input.social.received.is_empty());
    assert!(d.choice.social.iter().any(|s| s.positive > 0));
    assert!(r.episodes[2].method_before[5].summaries.is_empty());
    assert!(
        r.episodes[2].method_before[5]
            .tokens
            .iter()
            .all(|t| t.acceptance == e7::Acceptance::Pending)
    );
}
#[test]
fn scoped_undelivered_reports_have_no_recipient_effect() {
    let mut c = config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    );
    for i in &mut c.base.incidents {
        i.game.routes[0] = vec![0];
        i.game.second_routes[0] = vec![0];
    }
    let r = run(c).unwrap();
    assert!(r.final_social_memory[5].received.is_empty());
    for d in r
        .episodes
        .iter()
        .flat_map(|e| &e.investigation)
        .filter(|d| d.input.local.id == 5 && !d.forced)
    {
        assert_eq!(d.choice.action, d.choice.personal_proposal.action);
    }
}
#[test]
fn repeating_one_speaker_does_not_create_independent_support() {
    let a = social(
        vec![message(0, 0, Rating::Favorable)],
        Counts::default(),
        SocialKind::NaiveSocial,
    );
    let b = social(
        vec![
            message(0, 0, Rating::Favorable),
            message(1, 0, Rating::Favorable),
        ],
        Counts::default(),
        SocialKind::NaiveSocial,
    );
    assert_eq!(a.effective_accuracy, b.effective_accuracy);
    assert_eq!(b.positive, 1);
}
#[test]
fn visible_relays_deduplicate_without_discovering_hidden_origins() {
    let a = message(0, 0, Rating::Favorable);
    let mut b = message(1, 1, Rating::Favorable);
    let mut c = message(2, 2, Rating::Favorable);
    for m in [&mut b, &mut c] {
        m.payload.parent = Some(a.reference);
        m.payload.claimed_origin = Some(0);
        m.payload.basis = ClaimedBasis::Relay;
    }
    assert_eq!(
        social(
            vec![a.clone(), b.clone(), c.clone()],
            Counts::default(),
            SocialKind::NaiveSocial
        )
        .positive,
        3
    );
    assert_eq!(
        social(
            vec![a.clone(), b.clone(), c.clone()],
            Counts::default(),
            SocialKind::ProvenanceAwareSocial
        )
        .positive,
        1
    );
    for m in [&mut b, &mut c] {
        m.payload.parent = None;
        m.payload.claimed_origin = Some(m.speaker);
    }
    assert_eq!(
        social(
            vec![a, b, c],
            Counts::default(),
            SocialKind::ProvenanceAwareSocial
        )
        .positive,
        3
    );
}
#[test]
fn unknown_origins_are_assumptions_not_verified_truth() {
    let mut a = message(0, 0, Rating::Favorable);
    a.payload.claimed_origin = Some(4);
    let s = social(
        vec![a],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!(s.positive, 1);
    assert!(s.interpretations.iter().all(|i| !i.origin_verified));
}
#[test]
fn corrections_retractions_are_local_and_preserve_immutable_originals() {
    let original = message(0, 0, Rating::Favorable);
    let saved = original.clone();
    let mut correction = message(1, 0, Rating::Unfavorable);
    correction.revision = Some(original.reference);
    let before = social(
        vec![original.clone()],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    let after = social(
        vec![original.clone(), correction.clone()],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!((before.positive, after.negative), (1, 1));
    assert_eq!(original, saved);
    correction.retract = true;
    assert_eq!(
        social(
            vec![original, correction],
            Counts::default(),
            SocialKind::ProvenanceAwareSocial
        )
        .negative,
        0
    );
}
#[test]
fn foreign_and_unavailable_revision_targets_cannot_replace_a_claim() {
    let original = message(0, 0, Rating::Favorable);
    let mut foreign = message(1, 1, Rating::Unfavorable);
    foreign.revision = Some(original.reference);
    let s = social(
        vec![original, foreign.clone()],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!(s.positive, 1);
    assert!(
        s.interpretations
            .iter()
            .any(|i| i.standing == Standing::Unresolved)
    );
    assert_eq!(
        social(vec![foreign], Counts::default(), SocialKind::NaiveSocial).negative,
        0
    );
}
#[test]
fn inconsistent_independent_reports_preserve_uncertainty() {
    let s = social(
        vec![
            message(0, 0, Rating::Favorable),
            message(1, 1, Rating::Unfavorable),
        ],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!(
        s.effective_accuracy.numerator * 2,
        s.effective_accuracy.denominator
    );
    let mut b = message(1, 1, Rating::Unfavorable);
    b.payload.claimed_origin = Some(0);
    let s = social(
        vec![message(0, 0, Rating::Favorable), b],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!((s.positive, s.negative), (0, 0));
}
#[test]
fn social_influence_is_capped_and_cannot_overwrite_strong_personal_history() {
    let records = (0..5)
        .map(|id| message(id, id, Rating::Favorable))
        .collect();
    let s = social(
        records,
        Counts {
            failure: 50,
            nonmissing: 50,
            ..Counts::default()
        },
        SocialKind::NaiveSocial,
    );
    assert_eq!(s.positive, 4);
    assert!(s.effective_accuracy.numerator * 2 < s.effective_accuracy.denominator);
    assert_eq!(s.input.personal.failure, 50);
}
#[test]
fn bounded_fifo_eviction_never_looks_up_a_forgotten_parent() {
    let mut memory = SocialMemory::neutral(5);
    let mut evictions = 0;
    for episode in 0..8 {
        for slot in 0..3 {
            let mut m = message(slot, 0, Rating::Favorable);
            m.reference.episode = episode;
            let e = ingest(&memory, m, false).unwrap();
            evictions += e.evicted.len();
            memory = e.after;
        }
    }
    assert_eq!(memory.received.len(), 12);
    assert_eq!(evictions, 12);
    let mut correction = message(4, 0, Rating::Unfavorable);
    correction.reference.episode = 7;
    correction.revision = Some(Reference {
        episode: 0,
        slot: 0,
    });
    let e = ingest(&memory, correction, false).unwrap();
    assert_eq!(standing(&e.after, &e.message), Standing::Unresolved);
}
#[test]
fn hidden_intent_never_changes_equal_local_interpretations() {
    let m = message(0, 0, Rating::Favorable);
    let a = social(
        vec![m.clone()],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    let b = social(
        vec![m],
        Counts::default(),
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!(a, b);
    let mut c = config(Diagnostic::AccurateFirsthand, SocialKind::DirectOnly);
    let r = run(c.clone()).unwrap();
    c.base.processes[0].accuracy_before = 0;
    let other = run(c).unwrap();
    assert_eq!(r.episodes[0].investigation, other.episodes[0].investigation);
}
#[test]
fn received_reports_change_future_choices_and_after_reports_do_not_back_edit() {
    let r = run(config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let d = r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_ne!(d.choice.action, d.choice.personal_proposal.action);
    let after = run(config(
        Diagnostic::AfterInvestigation,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let d = after.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_eq!(d.choice.action, d.choice.personal_proposal.action);
    assert!(
        after.episodes[3]
            .investigation
            .iter()
            .any(|d| d.input.local.id == 5 && !d.input.social.received.is_empty())
    );
}
#[test]
fn resources_and_old_finding_weights_are_preserved() {
    let r = run(config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    for l in &r.ledgers {
        assert_eq!(l.initial, l.remaining + l.spent);
    }
    for e in &r.episodes {
        for d in &e.deliveries {
            assert_eq!(d.balance_before - d.balance_after, 1);
        }
    }
    assert!(r.episodes.iter().all(|e| e.incident.decisions.len() == 18
        && e.investigation.len() == 6
        && e.social_decisions.len() == 12));
}
#[test]
fn false_feedback_can_create_a_sincere_false_method_assessment() {
    let r = run(config(
        Diagnostic::CorruptedHistory,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let s = r.episodes[2]
        .social_decisions
        .iter()
        .find(|d| d.input.incident.incident.id == 0)
        .unwrap();
    assert_eq!(s.choice.intent, MethodIntent::Sincere);
    assert_eq!(
        s.message.as_ref().unwrap().payload.rating,
        Rating::Favorable
    );
    assert!(
        r.episodes[0]
            .deliveries
            .iter()
            .any(|d| d.finding.acquisition.observer == 0 && d.correct == Some(false))
    );
    assert!(
        r.episodes[0]
            .method_feedback
            .iter()
            .any(|f| f.observer == 0 && f.correct == Some(false))
    );
}
#[test]
fn executed_credibility_stays_separate_and_can_weight_bad_advice() {
    let r = run(config(
        Diagnostic::CredibleFalse,
        SocialKind::CredibilityWeightedSocial,
    ))
    .unwrap();
    assert!(r.episodes[2].memory_before[5].local.credits[0].score > 0);
    let d = r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert!(
        d.choice
            .social
            .iter()
            .any(|s| s.interpretations.iter().any(|i| i.weight > 1))
    );
    assert!(r.episodes[2].method_before[5].summaries.is_empty());
}
#[test]
fn legal_selected_and_injected_same_messages_have_distinct_origins() {
    let c = config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    );
    let selected = run(c.clone()).unwrap();
    let d = selected.episodes[2]
        .social_decisions
        .iter()
        .find(|d| d.message.is_some())
        .unwrap();
    let mut injected = c;
    injected.injections.push(Injection {
        episode: 2,
        slot: d.input.reference.slot,
        choice: d.choice.clone(),
    });
    let other = run(injected).unwrap();
    let i = other.episodes[2]
        .social_decisions
        .iter()
        .find(|x| x.input.reference == d.input.reference)
        .unwrap();
    assert_eq!(d.message, i.message);
    assert_ne!(d.origin, i.origin);
    assert_eq!(selected.final_social_memory, other.final_social_memory);
    assert_eq!(selected.ledgers, other.ledgers);
}
#[test]
fn malformed_actions_are_rejected_without_extra_speech() {
    let mut c = config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    );
    let r = run(c.clone()).unwrap();
    let d = r.episodes[2]
        .social_decisions
        .iter()
        .find(|d| d.message.is_some())
        .unwrap();
    for invalid in 0..4 {
        let mut choice = d.choice.clone();
        if let MethodAction::Report {
            payload,
            revision,
            recipients,
            ..
        } = &mut choice.action
        {
            match invalid {
                0 => {
                    *revision = Some(Reference {
                        episode: 0,
                        slot: 9,
                    })
                }
                1 => payload.family = MethodId::new("never-acquired"),
                2 => recipients.push(recipients[0]),
                _ => {
                    payload.parent = Some(Reference {
                        episode: 7,
                        slot: 0,
                    })
                }
            }
        }
        c.injections = vec![Injection {
            episode: 2,
            slot: d.input.reference.slot,
            choice,
        }];
        let result = run(c.clone()).unwrap();
        let slot = &result.episodes[2].social_decisions[usize::from(d.input.reference.slot)];
        assert!(slot.rejection.is_some());
        assert!(slot.message.is_none());
    }
}
#[test]
fn relay_depth_is_resolver_enforced_without_publishing_ancestry() {
    let r = run(config(
        Diagnostic::SharedOriginRelay,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let d = r.episodes[2]
        .social_decisions
        .iter()
        .find(|d| matches!(d.choice.basis, PrivateBasis::Received(_)))
        .unwrap();
    let parent = match d.choice.basis {
        PrivateBasis::Received(r) => r,
        _ => unreachable!(),
    };
    let deep = BTreeMap::from([(parent, (vec![parent], 3))]);
    let resolved = resolve_report(d.input.clone(), d.choice.clone(), Origin::Selected, &deep);
    assert!(resolved.rejection.is_some());
    assert!(resolved.message.is_none());
}
#[test]
fn correction_reaches_only_declared_recipients() {
    let all = run(config(
        Diagnostic::CorrectionAll,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    let none = run(config(
        Diagnostic::CorrectionNone,
        SocialKind::ProvenanceAwareSocial,
    ))
    .unwrap();
    assert!(
        all.episodes
            .iter()
            .skip(3)
            .flat_map(|e| &e.social_decisions)
            .any(|s| s.message.as_ref().is_some_and(|m| m.revision.is_some()))
    );
    assert!(
        none.episodes
            .iter()
            .skip(3)
            .flat_map(|e| &e.social_decisions)
            .filter(|s| s.input.incident.incident.id == 0)
            .all(|s| s.deliveries.iter().all(|d| d.after.observer == 0))
    );
}
#[test]
fn all_diagnostics_and_generated_worlds_validate_and_replay() {
    for family in DIAGNOSTICS {
        let c = diagnostic(family, 0, SocialKind::ProvenanceAwareSocial);
        c.validate().unwrap();
        let a = run(c.clone()).unwrap();
        let b = run(c).unwrap();
        assert_eq!(a, b);
    }
    for treatment in TREATMENTS {
        generated(0, treatment).validate().unwrap();
    }
}
#[test]
fn captured_matched_inputs_and_canonical_sections_are_complete() {
    let r = row(
        "diagnostic",
        "AccurateFirsthand",
        0,
        config(
            Diagnostic::AccurateFirsthand,
            SocialKind::ProvenanceAwareSocial,
        ),
    )
    .unwrap();
    assert_eq!(r.digests.len(), 7);
    assert_eq!(r.matched.len(), 1);
    assert_eq!(r.matched[0].variants.len(), 4);
    assert!(
        r.forecasts
            .iter()
            .any(|f| f.personal.graded() == 0 && f.positive > 0)
    );
    assert!(canonical_digest(&0.5).is_err());
    assert_eq!(paired_interval(&[1, 1]).mean, Some(1));
    assert_eq!(paired_interval(&[]).mean, None);
}
#[test]
fn forecasts_exclude_missing_accuracy_and_keep_scopes_separate() {
    let mut c = config(Diagnostic::AccurateFirsthand, SocialKind::DirectOnly);
    for i in &mut c.base.incidents {
        i.artifacts[0].candidate = None;
    }
    let r = row("test", "missing", 0, c).unwrap();
    let scores = forecast_scores(&r, false, "all", false);
    assert!(scores.missingness.samples > scores.accuracy.samples);
    assert!(
        forecast_scores(&r, false, "selected", false)
            .accuracy
            .samples
            <= scores.accuracy.samples
    );
}
#[test]
fn historical_files_are_not_required_as_policy_memory() {
    let c = config(Diagnostic::NoCommunication, SocialKind::DirectOnly);
    let r = run(c).unwrap();
    assert!(r.final_social_memory.iter().all(|m| m.received.is_empty()));
    let mut p = e10::Policy::new(e10::PolicyKind::ExperienceAdjusted, 0, 5);
    let input = &r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap()
        .input;
    assert_eq!(
        e10::InvestigationPolicy::choose(&mut p, &input.personal).action,
        r.episodes[2]
            .investigation
            .iter()
            .find(|d| d.input.local.id == 5)
            .unwrap()
            .choice
            .action
    );
    assert!(
        e7::compare_platforms(
            &e7::PlatformReplay {
                execution: e7::execution().unwrap(),
                manifest: manifest(&Evaluation {
                    version: "test".into(),
                    independent_worlds: 0,
                    incidents: 0,
                    replay_matches: 0,
                    rows: vec![],
                    comparisons: vec![],
                    prediction_groups: BTreeMap::new()
                })
                .unwrap()
            },
            &e7::PlatformReplay {
                execution: e7::execution().unwrap(),
                manifest: manifest(&Evaluation {
                    version: "test".into(),
                    independent_worlds: 0,
                    incidents: 0,
                    replay_matches: 0,
                    rows: vec![],
                    comparisons: vec![],
                    prediction_groups: BTreeMap::new()
                })
                .unwrap()
            }
        )
        .is_err()
    );
}
#[test]
fn executed_social_history_changes_equal_present_decisions() {
    let [a, b] = history_pair(0);
    let ar = run(a).unwrap();
    let br = run(b).unwrap();
    let x = ar.episodes[7]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    let y = br.episodes[7]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_eq!(x.input.personal, y.input.personal);
    assert_ne!(x.input.social, y.input.social);
    assert_ne!(x.choice.action, y.choice.action);
}
#[test]
fn method_communication_limits_do_not_grant_extra_incident_speech() {
    let mut c = config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    );
    c.base.budgets = [0; 6];
    let r = run(c).unwrap();
    assert!(r.ledgers.iter().all(|l| l.spent == 0 && l.remaining == 0));
    for e in &r.episodes {
        let methods = e
            .social_decisions
            .iter()
            .filter(|s| s.message.is_some())
            .count();
        let incident = e.incident.testimony.len();
        assert!(methods + incident <= 12);
        assert_eq!(e.incident.decisions.len(), 18);
    }
    let mut c = config(Diagnostic::AccurateFirsthand, SocialKind::DirectOnly);
    c.report_round = Some(2);
    assert!(c.validate().is_err());
    c.report_round = None;
    c.injections = vec![Injection {
        episode: 0,
        slot: 12,
        choice: SharingChoice {
            action: MethodAction::Silence,
            intent: MethodIntent::NoMethod,
            basis: PrivateBasis::Unsupported,
            eligible: false,
            reason: "invalid".into(),
        },
    }];
    assert!(c.validate().is_err());
}
#[test]
fn development_population_has_complete_world_pairing_and_separate_controls() {
    let e = evaluate_seeds(0..1, 0..1).unwrap();
    assert_eq!(e.rows.len(), 104);
    assert_eq!(e.incidents, 832);
    assert_eq!(e.independent_worlds, 1);
    assert_eq!(e.replay_matches, e.rows.len());
    assert_eq!(e.comparisons.len(), 96);
    assert!(e.rows.iter().all(|r| r.digests.len() == 7));
    assert!(!e.prediction_groups.is_empty());
    let origin: Vec<_> = e
        .rows
        .iter()
        .filter(|r| r.population == "message_origin")
        .collect();
    assert_eq!(origin.len(), 2);
    assert_eq!(origin[0].ballots, origin[1].ballots);
    assert_eq!(
        origin[0]
            .speech
            .iter()
            .filter_map(|s| s.wire.as_ref())
            .collect::<Vec<_>>(),
        origin[1]
            .speech
            .iter()
            .filter_map(|s| s.wire.as_ref())
            .collect::<Vec<_>>()
    );
}
#[test]
fn equal_wire_with_changed_hidden_selected_intent_preserves_listener_decisions() {
    let c = config(
        Diagnostic::AccurateFirsthand,
        SocialKind::ProvenanceAwareSocial,
    );
    let original = run(c.clone()).unwrap();
    let source = original.episodes[2]
        .social_decisions
        .iter()
        .find(|s| s.message.is_some())
        .unwrap();
    let mut altered = c;
    let mut choice = source.choice.clone();
    choice.intent = MethodIntent::Contrary;
    altered.injections.push(Injection {
        episode: 2,
        slot: source.input.reference.slot,
        choice,
    });
    let changed = run(altered).unwrap();
    let a = original.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    let b = changed.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(original.final_social_memory, changed.final_social_memory);
}
#[test]
fn role_and_order_permutation_does_not_install_reliability_in_identity() {
    let a = diagnostic(
        Diagnostic::AccurateFirsthand,
        0,
        SocialKind::ProvenanceAwareSocial,
    );
    let b = diagnostic(
        Diagnostic::IdentityOrder,
        0,
        SocialKind::ProvenanceAwareSocial,
    );
    assert_eq!(a.base.processes, b.base.processes);
    assert_eq!(a.incidents[2].artifacts, b.incidents[2].artifacts);
    assert_ne!(a.incidents[2].game.order, b.incidents[2].game.order);
    assert_ne!(a.sharing, b.sharing);
    let r = run(b).unwrap();
    assert!(
        r.episodes[2]
            .social_decisions
            .iter()
            .any(|s| s.message.as_ref().is_some_and(|m| m.speaker == 3))
    );
}
#[test]
fn duplicate_receipts_and_returned_own_source_echoes_add_no_evidence() {
    let original = message(0, 0, Rating::Favorable);
    let a = ingest(&SocialMemory::neutral(5), original.clone(), false).unwrap();
    let b = ingest(&a.after, original, false).unwrap();
    assert!(b.duplicate);
    assert_eq!(a.after, b.after);
    let mut echo = message(1, 1, Rating::Favorable);
    echo.payload.claimed_origin = Some(5);
    assert_eq!(
        social(
            vec![echo],
            Counts::default(),
            SocialKind::ProvenanceAwareSocial
        )
        .positive,
        0
    );
}
