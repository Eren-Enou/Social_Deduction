use social_deduction::{experiment007 as e7, experiment009 as e9, experiment010::*};

fn config(policy: PolicyKind, episodes: usize) -> Config {
    let old = e9::diagnostic_config(
        e9::Diagnostic::QualityVsCost,
        0,
        e9::PolicyKind::FirstAccessible,
    );
    let incidents = (0..episodes)
        .map(|e| {
            let c = old.incidents[e % 3].clone();
            Incident {
                methods: c
                    .artifacts
                    .iter()
                    .map(|s| (s.descriptor.id, MethodId::new(s.method_key.clone())))
                    .collect(),
                game: c.game,
                artifacts: c.artifacts,
                forced: c.forced,
                access_order: c.access_order,
                communication: c.communication,
                injected: c.injected,
            }
        })
        .collect::<Vec<_>>();
    let feedback = incidents
        .iter()
        .enumerate()
        .map(|(e, c)| FeedbackPlan {
            after_episode: e as u8,
            about_episode: e as u8,
            recipients: (0..6).collect(),
            token: e7::Token::outcome(0, e as u8, 0, c.game.actor()),
        })
        .collect();
    Config {
        version: "series-010-v1".into(),
        session: 0,
        incidents,
        policy: [policy; 6],
        budgets: [24; 6],
        listener: old.listener,
        feedback,
        processes: (0..2)
            .map(|id| MethodProcess {
                family: MethodId::new(format!("method-{id}")),
                accuracy_before: 3,
                accuracy_after: 1,
                missing_before: 0,
                missing_after: 1,
                shift_episode: Some(4),
            })
            .collect(),
    }
}

#[test]
fn full_eight_incidents_learn_only_direct_paid_acquisitions_and_replay() {
    let c = config(PolicyKind::FirstAccessible, 8);
    let a = run(c.clone()).unwrap();
    let b = run(c).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        serde_json::to_vec(&a).unwrap(),
        serde_json::to_vec(&b).unwrap()
    );
    assert_eq!(a.episodes.len(), 8);
    assert_eq!(
        a.episodes
            .iter()
            .map(|e| e.method_ingestions.len())
            .sum::<usize>(),
        48
    );
    for id in 0..6 {
        let m = &a.final_method_memory[id];
        assert_eq!(m.summaries[&MethodId::new("method-0")].success, 8);
        assert_eq!(m.records.len(), 8);
        assert!(!m.summaries.contains_key(&MethodId::new("method-1")));
        assert_eq!(
            a.ledgers[id].initial,
            a.ledgers[id].remaining + a.ledgers[id].spent
        );
        assert_eq!(a.ledgers[id].spent, 8);
    }
    for e in &a.episodes {
        assert_eq!(e.investigation.len(), 6);
        assert_eq!(e.incident.decisions.len(), 18);
        for d in &e.investigation {
            assert!(d.rejection.is_none());
            assert!(d.input.local.evidence.findings.is_empty());
            assert!(!d.input.experience.contains_key(&MethodId::new("unknown")));
        }
    }
}
#[test]
fn no_investigation_no_direct_experience_and_private_memory_is_local() {
    let mut c = config(PolicyKind::NeverInvestigate, 3);
    c.policy[5] = PolicyKind::FirstAccessible;
    for e in &mut c.incidents {
        e.communication = [CommunicationKind::Silent; 6];
    }
    let r = run(c).unwrap();
    for id in 0..5 {
        assert!(r.final_method_memory[id].summaries.is_empty());
        assert!(r.final_method_memory[id].records.is_empty());
    }
    assert_eq!(
        r.final_method_memory[5].summaries[&MethodId::new("method-0")].success,
        3
    );
}
#[test]
fn unchanged009_frozen_selection_and_belief_execution_match() {
    let mut c = config(PolicyKind::FirstAccessible, 3);
    c.budgets = [3; 6];
    let r = run(c).unwrap();
    let frozen = e9::run(e9::diagnostic_config(
        e9::Diagnostic::QualityVsCost,
        0,
        e9::PolicyKind::FirstAccessible,
    ))
    .unwrap();
    for (new, old) in r.episodes.iter().zip(&frozen.episodes) {
        assert_eq!(new.incident.decisions, old.incident.decisions);
        assert_eq!(new.incident.testimony, old.incident.testimony);
        for (a, b) in new.cognition.iter().zip(&old.cognition) {
            assert_eq!(a.input, b.input);
            assert_eq!(a.assessment, b.assessment);
        }
        assert_eq!(new.memory_after, old.memory_after);
        for (a, b) in new.investigation.iter().zip(&old.investigation) {
            assert_eq!(a.input.local, b.input);
            assert_eq!(a.choice.action, b.choice.action);
            assert_eq!(a.choice.considered, b.choice.considered);
        }
    }
}
#[test]
fn changing_hidden_method_parameters_or_sealed_results_preserves_first_choices() {
    let c = config(PolicyKind::ExperienceAdjusted, 3);
    let a = run(c.clone()).unwrap();
    let mut hidden = c;
    hidden.processes[0].accuracy_before = 0;
    hidden.processes[0].accuracy_after = 4;
    for e in &mut hidden.incidents {
        e.artifacts[0].candidate = Some((e.game.actor() + 1) % 6);
    }
    let b = run(hidden).unwrap();
    for (a, b) in a.episodes[0]
        .investigation
        .iter()
        .zip(&b.episodes[0].investigation)
    {
        assert_eq!(a.input, b.input);
        assert_eq!(a.choice, b.choice);
    }
}
#[test]
fn variable_missing_costs_free_injections_and_forced_training_remain_distinct() {
    let mut c = config(PolicyKind::NeverInvestigate, 1);
    c.incidents[0].forced = vec![(5, InvestigationAction::Inspect(1))];
    c.incidents[0].injected = vec![(5, 1)];
    c.incidents[0].artifacts[1].cost = 3;
    c.incidents[0].artifacts[1].candidate = None;
    let r = run(c).unwrap();
    assert_eq!(r.ledgers[5].spent, 3);
    let e = &r.episodes[0];
    assert_eq!(e.deliveries.len(), 2);
    assert!(matches!(e.deliveries[0].origin, DeliveryOrigin::Forced(_)));
    assert!(matches!(
        e.deliveries[1].origin,
        DeliveryOrigin::Intervention(_)
    ));
    assert_eq!(
        e.deliveries[1].balance_before,
        e.deliveries[1].balance_after
    );
    let n = r.final_method_memory[5].summaries[&MethodId::new("method-1")];
    assert_eq!(n.missing, 1);
    assert_eq!(n.graded(), 0);
    assert!(e.method_ingestions[1].duplicate);
}
#[test]
fn feedback_errors_quarantine_preserve_past_and_eviction_is_audited() {
    let mut c = config(PolicyKind::FirstAccessible, 8);
    for e in &mut c.incidents {
        e.injected = vec![(5, 1)];
    }
    let actor = c.incidents[0].game.actor();
    c.feedback.push(FeedbackPlan {
        after_episode: 1,
        about_episode: 0,
        recipients: vec![5],
        token: e7::Token::outcome(0, 0, 1, (actor + 1) % 6),
    });
    c.feedback.push(FeedbackPlan {
        after_episode: 7,
        about_episode: 0,
        recipients: vec![5],
        token: e7::Token::outcome(0, 0, 2, (actor + 1) % 6),
    });
    let r = run(c).unwrap();
    assert!(matches!(
        r.episodes[0].method_after[5].records[0].status,
        ExperienceStatus::Accepted { correct: true, .. }
    ));
    assert!(
        r.episodes[1].method_feedback.iter().any(|f| f.observer == 5
            && f.effect.transport_effect.acceptance == e7::Acceptance::Quarantined)
    );
    assert!(r.episodes[7].method_feedback.iter().any(
        |f| f.observer == 5 && f.effect.transport_effect.acceptance == e7::Acceptance::Pending
    ));
    assert!(
        r.episodes
            .iter()
            .flat_map(|e| &e.method_ingestions)
            .any(|i| !i.evicted.is_empty())
    );
    assert_eq!(r.final_method_memory[5].records.len(), 8);
}
#[test]
fn malformed_family_mapping_series_limits_and_budget_rejected() {
    let mut c = config(PolicyKind::FirstAccessible, 1);
    c.budgets[0] = 25;
    assert!(run(c.clone()).is_err());
    c.budgets[0] = 24;
    c.incidents[0].methods.clear();
    assert!(run(c.clone()).is_err());
    let mut c = config(PolicyKind::FirstAccessible, 1);
    c.processes[0].family = MethodId::new("bad identity");
    assert!(run(c).is_err());
    assert!(run(config(PolicyKind::FirstAccessible, 9)).is_err());
}
