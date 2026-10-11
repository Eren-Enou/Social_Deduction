use social_deduction::{
    Phase, experiment005 as e5, experiment006 as e6, experiment007 as e7, experiment008::*,
};
fn fixture(d: Diagnostic, p: PolicyKind) -> Report {
    run(diagnostic_config(d, 0, p)).unwrap()
}
fn belief(candidate: Option<u8>) -> e5::Belief {
    let mut scores = [0; 6];
    if let Some(c) = candidate {
        scores[usize::from(c)] = 2;
    }
    e5::Belief::from_scores(scores)
}
fn view() -> InvestigationView {
    InvestigationView {
        id: 5,
        episode: 0,
        episodes_remaining: 2,
        belief: belief(None),
        unweighted: belief(None),
        received: vec![],
        credits: [e6::Credit::default(); 6],
        catalog: vec![Descriptor {
            id: 0,
            description: "sealed scene trace".into(),
            visible_parent: None,
        }],
        balance: 2,
        stage: Stage::Investigation,
        used: false,
        action_cost: 1,
    }
}
fn evidence() -> EvidenceView {
    let original = e5::IndividualState {
        observation: None,
        briefing: None,
        belief: None,
        incentives: e5::Incentives::ACCURACY,
    };
    EvidenceView {
        incident: e5::ListenerView {
            id: 5,
            original,
            observations: vec![e5::Observation {
                acquisition: 5,
                candidate: None,
            }],
            prior: belief(None),
            received: vec![],
        },
        credits: [e6::Credit::default(); 6],
        findings: vec![],
        reports: vec![],
    }
}
fn finding(
    id: u8,
    candidate: Option<u8>,
    parent: Option<u8>,
    observer: u8,
    local: usize,
) -> Finding {
    Finding {
        source: Descriptor {
            id,
            description: "trace".into(),
            visible_parent: parent,
        },
        candidate,
        acquisition: Acquisition {
            episode: 0,
            observer,
            local,
        },
    }
}
fn assess(v: &EvidenceView) -> Assessment {
    EvidenceAware {
        testimony: e6::ListenerKind::Credibility,
    }
    .assess(v)
}
fn message(id: usize, speaker: u8, candidate: u8) -> e5::Message {
    e5::Message {
        id,
        speaker,
        body: e5::Body::Assert(candidate),
        attribution: e5::Attribution::Firsthand,
    }
}
#[test]
fn latent_worlds_are_policy_independent_and_reproducible() {
    for seed in 0..16 {
        let w = Generator.generate(seed);
        assert_eq!(w, Generator.generate(seed));
        for p in POLICIES {
            let c = generated_config(&w, p, false);
            assert_eq!(c.incidents, w.base.incidents);
            c.validate().unwrap();
        }
    }
}
#[test]
fn hidden_truth_content_and_roots_cannot_change_identical_policy_inputs() {
    let mut a = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::SeededRandom);
    a.incidents.truncate(1);
    a.incidents[0].communication = [CommunicationKind::Silent; 6];
    let mut b = a.clone();
    b.incidents[0].artifacts[0].candidate = Some((a.incidents[0].game.actor() + 1) % 6);
    let x = run(a).unwrap();
    let y = run(b).unwrap();
    assert_eq!(
        x.episodes[0].investigation,
        y.episodes[0]
            .investigation
            .iter()
            .map(|d| {
                let mut d = d.clone();
                d.delivery = x.episodes[0]
                    .investigation
                    .iter()
                    .find(|x| x.input.id == d.input.id)
                    .unwrap()
                    .delivery;
                d
            })
            .collect::<Vec<_>>()
    );
    for p in POLICIES {
        assert_eq!(
            Policy::new(p, 44, 5).choose(&view()),
            Policy::new(p, 44, 5).choose(&view())
        );
    }
    let input = serde_json::to_value(&x.episodes[0].investigation[0].input).unwrap();
    assert!(input.get("actual_root").is_none());
    assert!(input.get("correct").is_none());
    assert!(input.get("artifacts").is_none());
    assert!(input["catalog"][0].get("candidate").is_none());
}
#[test]
fn no_investigation_content_is_exposed_before_access_or_to_noninvestigators() {
    let r = fixture(Diagnostic::Accurate, PolicyKind::NeverInvestigate);
    assert!(r.episodes.iter().all(|e| {
        e.deliveries.is_empty()
            && e.cognition
                .iter()
                .all(|c| c.input.findings.is_empty() && c.input.reports.is_empty())
    }));
    let a = fixture(
        Diagnostic::DifferentObservers,
        PolicyKind::AlwaysInvestigate,
    );
    for e in &a.episodes {
        for c in e.cognition.iter().take(6) {
            assert!(c.input.findings.is_empty());
            assert!(c.input.reports.is_empty());
        }
        for c in &e.cognition {
            if ![2, 4].contains(&c.input.incident.id) {
                assert!(c.input.findings.is_empty());
            }
        }
    }
}
#[test]
fn effort_conservation_exhaustion_decline_and_missing_cost_are_explicit() {
    for family in [
        Diagnostic::Accurate,
        Diagnostic::Missing,
        Diagnostic::Exhaustion,
    ] {
        let r = fixture(family, PolicyKind::AlwaysInvestigate);
        for l in &r.ledgers {
            assert_eq!(l.initial, l.remaining + l.spent);
            assert_eq!(l.remaining, 0);
        }
        for d in r.episodes.iter().flat_map(|e| &e.deliveries) {
            assert_eq!(d.balance_before, d.balance_after + 1);
        }
        if family == Diagnostic::Missing {
            assert!(
                r.episodes
                    .iter()
                    .flat_map(|e| &e.deliveries)
                    .all(|d| d.finding.candidate.is_none())
            );
        }
    }
    let r = fixture(Diagnostic::Accurate, PolicyKind::NeverInvestigate);
    assert!(r.ledgers.iter().all(|l| l.spent == 0 && l.remaining == 2));
    let mut v = view();
    v.balance = 0;
    assert!(validate_investigation(&v, InvestigationAction::Inspect(0)).is_err());
    assert!(validate_investigation(&v, InvestigationAction::Decline).is_ok());
}
#[test]
fn uncertainty_and_scarcity_causally_change_selective_actions() {
    let mut p = Policy::new(PolicyKind::CostSensitive, 0, 5);
    let mut v = view();
    assert_eq!(p.choose(&v).action, InvestigationAction::Inspect(0));
    v.balance = 1;
    v.belief = e5::Belief::from_scores([3, 1, 0, 0, 0, 0]);
    assert_eq!(p.choose(&v).action, InvestigationAction::Decline);
    v.balance = 2;
    assert_eq!(p.choose(&v).action, InvestigationAction::Inspect(0));
    v.belief = belief(Some(0));
    assert_eq!(p.choose(&v).action, InvestigationAction::Decline);
    let always = fixture(Diagnostic::Conservation, PolicyKind::AlwaysInvestigate);
    let selective = fixture(Diagnostic::Conservation, PolicyKind::ConflictDriven);
    assert!(
        always.episodes[0]
            .investigation
            .iter()
            .all(|d| matches!(d.choice.action, InvestigationAction::Inspect(_)))
    );
    assert!(
        selective.episodes[0]
            .investigation
            .iter()
            .all(|d| d.choice.action == InvestigationAction::Decline)
    );
    assert!(always.episodes[1].deliveries.is_empty());
    assert!(!selective.episodes[1].deliveries.is_empty());
}
#[test]
fn local_credibility_can_trigger_inspection_without_truth_access() {
    let mut v = view();
    v.belief = belief(Some(0));
    v.unweighted = v.belief.clone();
    v.received.push(message(0, 0, 0));
    let mut p = Policy::new(PolicyKind::CredibilityAware, 0, 5);
    assert_eq!(p.choose(&v).action, InvestigationAction::Decline);
    v.credits[0].score = -1;
    let c = p.choose(&v);
    assert!(c.credibility_warning);
    assert_eq!(c.action, InvestigationAction::Inspect(0));
    assert_eq!(
        Policy::new(PolicyKind::CostSensitive, 0, 5)
            .choose(&v)
            .action,
        InvestigationAction::Decline
    );
}
#[test]
fn resolver_rejects_duplicate_phase_ids_access_and_exhausted_actions() {
    let mut v = view();
    assert!(validate_investigation(&v, InvestigationAction::Inspect(99)).is_err());
    v.catalog.clear();
    assert!(validate_investigation(&v, InvestigationAction::Inspect(0)).is_err());
    v = view();
    v.used = true;
    assert!(validate_investigation(&v, InvestigationAction::Decline).is_err());
    v = view();
    for stage in [
        Stage::FirstDiscussion,
        Stage::SecondDiscussion,
        Stage::Judgment,
        Stage::Complete,
    ] {
        v.stage = stage;
        assert!(validate_investigation(&v, InvestigationAction::Inspect(0)).is_err());
    }
    v = view();
    v.id = 9;
    assert!(validate_investigation(&v, InvestigationAction::Decline).is_err());
}
struct Illegal;
impl InvestigationPolicy for Illegal {
    fn choose(&mut self, _: &InvestigationView) -> InvestigationChoice {
        InvestigationChoice {
            action: InvestigationAction::Inspect(99),
            usefulness: 0,
            cost: 0,
            credibility_warning: false,
            mechanism: "adversarial".into(),
        }
    }
}
#[test]
fn rejected_custom_policy_gets_no_delivery_or_resource_and_cannot_retry() {
    let c = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::AlwaysInvestigate);
    let mut p = Components::from_config(&c);
    p.investigators = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn InvestigationPolicy>)
        .collect();
    let r = run_with(c, &mut p).unwrap();
    assert_eq!(r.implementation, "caller-supplied-unversioned");
    assert!(r.ledgers.iter().all(|l| l.spent == 0));
    for e in r.episodes {
        assert_eq!(e.investigation.len(), 6);
        assert!(e.investigation.iter().all(|d| d.rejection.is_some()));
        assert!(e.deliveries.is_empty());
    }
}
#[test]
fn accurate_and_false_findings_change_beliefs_without_rewriting_truth() {
    let mut v = evidence();
    v.findings.push(finding(0, Some(2), None, 5, 0));
    assert_eq!(assess(&v).after.candidate, Some(2));
    v.findings[0].candidate = Some(3);
    assert_eq!(assess(&v).after.candidate, Some(3));
    v.incident.original.observation = Some(2);
    v.incident.observations[0].candidate = Some(2);
    let a = assess(&v);
    assert_eq!(a.after.candidate, None);
    assert_eq!(a.after.scores[2], 2);
    assert_eq!(a.after.scores[3], 2);
    assert_eq!(a.unweighted, a.after);
    let r = fixture(Diagnostic::Misleading, PolicyKind::AlwaysInvestigate);
    assert!(
        r.episodes
            .iter()
            .flat_map(|e| &e.deliveries)
            .all(|d| d.correct == Some(false))
    );
    assert!(metrics(&r).unwrap().acquisition_worsened > 0);
}
#[test]
fn repeated_artifact_acquisitions_reports_and_relays_count_one_source() {
    let mut v = evidence();
    let f = finding(0, Some(2), None, 0, 0);
    for id in 0..3 {
        v.incident.received.push(message(id, id as u8, 2));
        v.reports.push(ReportedFinding {
            message: id,
            finding: f.clone(),
            basis: if id == 0 {
                ReportBasis::Acquired(f.acquisition)
            } else {
                ReportBasis::Received(0)
            },
        });
    }
    v.findings.push(finding(0, Some(2), None, 5, 0));
    let a = assess(&v);
    assert_eq!(a.after.scores[2], 2);
    assert_eq!(a.contributions.len(), 1);
    assert_eq!(a.contributions[0].observations, 4);
    assert_eq!(a.contributions[0].group, Perceived::Artifact(0));
}
#[test]
fn known_dependence_uses_maximum_weight_hidden_dependence_is_separate() {
    let mut v = evidence();
    v.incident.received.push(message(0, 0, 2));
    v.findings.push(finding(1, Some(2), Some(0), 5, 0));
    let known = assess(&v);
    assert_eq!(known.after.scores[2], 2);
    assert_eq!(known.contributions.len(), 1);
    v.findings[0].source.visible_parent = None;
    let hidden = assess(&v);
    assert_eq!(hidden.after.scores[2], 3);
    assert_eq!(hidden.contributions.len(), 2);
    let r = fixture(Diagnostic::HiddenDependent, PolicyKind::AlwaysInvestigate);
    for d in r.episodes.iter().flat_map(|e| &e.deliveries) {
        assert_eq!(d.actual_root, Root::Observation(0));
        assert_eq!(d.finding.source.visible_parent, None);
        assert!(
            !serde_json::to_string(&d.finding)
                .unwrap()
                .contains("actual_root")
        );
    }
}
#[test]
fn conflicting_known_sources_and_retractions_preserve_uncertainty() {
    let mut v = evidence();
    v.incident.received.push(message(0, 0, 2));
    v.findings.push(finding(1, Some(3), Some(0), 5, 0));
    let a = assess(&v);
    assert_eq!(a.after.candidate, None);
    assert!(a.contributions[0].contested);
    assert_eq!(a.contributions[0].weight, 0);
    let f = finding(0, Some(2), None, 0, 0);
    v = evidence();
    v.incident.received.push(message(0, 0, 2));
    v.reports.push(ReportedFinding {
        message: 0,
        finding: f,
        basis: ReportBasis::Acquired(Acquisition {
            episode: 0,
            observer: 0,
            local: 0,
        }),
    });
    v.incident.received.push(e5::Message {
        id: 1,
        speaker: 0,
        body: e5::Body::Revise {
            target: 0,
            replacement: None,
            forwarded: None,
        },
        attribution: e5::Attribution::Unspecified,
    });
    assert_eq!(assess(&v).after.candidate, None);
    assert_eq!(v.incident.received[0].body, e5::Body::Assert(2));
}
#[test]
fn explicit_finding_revision_changes_branch_identity_and_ignores_speaker_credit() {
    let mut v = evidence();
    v.incident.received.push(message(0, 0, 2));
    v.incident.received.push(e5::Message {
        id: 1,
        speaker: 0,
        body: e5::Body::Revise {
            target: 0,
            replacement: Some(3),
            forwarded: None,
        },
        attribution: e5::Attribution::Firsthand,
    });
    let f = finding(0, Some(3), None, 0, 0);
    v.reports.push(ReportedFinding {
        message: 1,
        finding: f.clone(),
        basis: ReportBasis::Acquired(f.acquisition),
    });
    v.credits[0].score = -2;
    let a = assess(&v);
    assert_eq!(a.after.candidate, Some(3));
    assert_eq!(a.after.scores[3], 1);
    assert_eq!(a.contributions[0].group, Perceived::Artifact(0));
}
#[test]
fn private_results_do_not_change_undelivered_observer_inputs_or_actions() {
    let mut c = diagnostic_config(Diagnostic::PrivateFinding, 0, PolicyKind::AlwaysInvestigate);
    c.incidents.truncate(1);
    c.incidents[0].artifacts[0].inspectors = vec![2];
    let mut different = c.clone();
    different.incidents[0].artifacts[0].candidate = Some((c.incidents[0].game.actor() + 1) % 6);
    let a = run(c).unwrap();
    let b = run(different).unwrap();
    assert!(a.episodes[0].incident.testimony.is_empty());
    for id in [0, 1, 3, 4, 5] {
        let x: Vec<_> = a.episodes[0]
            .communication
            .iter()
            .filter(|c| c.input.incident.id == id)
            .collect();
        let y: Vec<_> = b.episodes[0]
            .communication
            .iter()
            .filter(|c| c.input.incident.id == id)
            .collect();
        assert_eq!(x, y);
    }
    assert_ne!(
        a.episodes[0].deliveries[0].after_assessment.after,
        b.episodes[0].deliveries[0].after_assessment.after
    );
}
#[test]
fn selected_inspection_and_injection_same_information_different_cost_origin() {
    let mut a = diagnostic_config(Diagnostic::PrivateFinding, 0, PolicyKind::AlwaysInvestigate);
    a.incidents.truncate(1);
    let mut b = a.clone();
    b.policy = [PolicyKind::NeverInvestigate; 6];
    b.incidents[0].injected = (0..6).map(|id| (id, 0)).collect();
    let x = run(a).unwrap();
    let y = run(b).unwrap();
    for (d, e) in x.episodes[0]
        .deliveries
        .iter()
        .zip(&y.episodes[0].deliveries)
    {
        assert_eq!(d.finding, e.finding);
        assert_eq!(d.after_assessment, e.after_assessment);
        assert!(matches!(d.origin, DeliveryOrigin::Selected(_)));
        assert!(matches!(e.origin, DeliveryOrigin::Intervention(_)));
        assert_eq!(d.balance_before, d.balance_after + 1);
        assert_eq!(e.balance_before, e.balance_after);
    }
    assert!(y.ledgers.iter().all(|l| l.spent == 0));
}
#[test]
fn budgets_memories_and_acquisitions_are_local_across_episodes() {
    let r = fixture(
        Diagnostic::DifferentObservers,
        PolicyKind::AlwaysInvestigate,
    );
    for id in 0..6 {
        assert_eq!(r.final_memory[id].local.observer, id as u8);
        assert_eq!(
            r.ledgers[id].spent,
            if [2, 4].contains(&id) { 2 } else { 0 }
        );
    }
    for e in &r.episodes {
        for c in &e.cognition {
            assert!(
                c.input
                    .findings
                    .iter()
                    .all(|f| f.acquisition.episode == e.id
                        && f.acquisition.observer == c.input.incident.id)
            );
            assert_eq!(
                c.input.credits,
                e.memory_before[usize::from(c.input.incident.id)]
                    .local
                    .credits
            );
        }
    }
}
#[test]
fn schedule_is_bounded_and_ballots_legal_with_majority_and_termination() {
    for p in POLICIES {
        let r = fixture(Diagnostic::Conflict, p);
        for e in &r.episodes {
            assert_eq!(e.investigation.len(), 6);
            assert_eq!(e.incident.decisions.len(), 18);
            for d in &e.incident.decisions {
                if let Some(reason) = &d.rejection {
                    assert_eq!(reason, "evidence depth exceeds three");
                    assert_eq!(d.input.phase, Phase::Discussion);
                    assert!(!e.incident.testimony.iter().any(|t| {
                        t.origin
                            == e5::Origin::Selected {
                                decision: e
                                    .incident
                                    .decisions
                                    .iter()
                                    .position(|x| std::ptr::eq(x, d))
                                    .unwrap(),
                            }
                    }));
                }
            }
            assert_eq!(
                e.incident.verdict,
                social_deduction::verdict(e.incident.tally)
            );
            assert!(matches!(
                e.incident.objective_history.last(),
                Some(e5::Event::Outcome { .. })
            ));
            for d in &e.incident.decisions {
                e5::validate_action(&d.input, &d.choice.action).unwrap();
            }
        }
        for id in 0..3 {
            let stages: Vec<_> = r
                .objective_history
                .iter()
                .filter_map(|e| {
                    if let Event::Stage { episode, stage } = e {
                        (*episode == id).then_some(*stage)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(
                stages,
                vec![
                    Stage::FirstDiscussion,
                    Stage::Investigation,
                    Stage::SecondDiscussion,
                    Stage::Judgment,
                    Stage::Complete
                ]
            );
        }
    }
    assert_eq!(social_deduction::verdict([3, 3, 0, 0, 0, 0]), None);
    assert_eq!(social_deduction::verdict([4, 2, 0, 0, 0, 0]), Some(0));
}
#[test]
fn malformed_configs_and_controller_deliveries_are_rejected() {
    let original = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::AlwaysInvestigate);
    let mut c = original.clone();
    c.incidents[0].access_order = [0; 6];
    assert!(c.validate().is_err());
    c = original.clone();
    let duplicate = c.incidents[0].artifacts[0].clone();
    c.incidents[0].artifacts.push(duplicate);
    assert!(c.validate().is_err());
    c = original.clone();
    c.incidents[0].artifacts[0].actual_root = Root::Observation(5);
    assert!(c.validate().is_err());
    c = original.clone();
    c.incidents[0].injected = vec![(9, 0)];
    assert!(c.validate().is_err());
    c = original.clone();
    c.incidents[0].injected = vec![(0, 0), (0, 0)];
    assert!(c.validate().is_err());
    c = original.clone();
    c.incidents[0].artifacts[0].inspectors = vec![0, 0];
    assert!(c.validate().is_err());
    c = original.clone();
    c.budgets[0] = 4;
    assert!(c.validate().is_err());
    c = original;
    c.action_cost = 255;
    assert!(c.validate().is_err());
}
#[test]
fn finding_reports_require_acquired_or_exact_received_payload_and_valid_recipients() {
    let r = fixture(Diagnostic::Accurate, PolicyKind::AlwaysInvestigate);
    let d = r.episodes[0]
        .communication
        .iter()
        .find(|d| d.choice.report.is_some())
        .unwrap();
    validate_communication(&d.input, &d.choice).unwrap();
    let mut bad = d.choice.clone();
    bad.report.as_mut().unwrap().0.acquisition.observer = 9;
    assert!(validate_communication(&d.input, &bad).is_err());
    bad = d.choice.clone();
    if let e5::Action::Speak { recipients, .. } = &mut bad.choice.action {
        *recipients = vec![9];
    }
    assert!(validate_communication(&d.input, &bad).is_err());
    bad = d.choice.clone();
    if let e5::Action::Speak { recipients, .. } = &mut bad.choice.action {
        *recipients = vec![0, 0];
    }
    assert!(validate_communication(&d.input, &bad).is_err());
    bad = d.choice.clone();
    bad.report.as_mut().unwrap().1 = ReportBasis::Received(999);
    assert!(validate_communication(&d.input, &bad).is_err());
    bad = d.choice.clone();
    bad.report.as_mut().unwrap().0.source.id = 99;
    assert!(validate_communication(&d.input, &bad).is_err());
    let mut view = d.input.clone();
    view.incident.phase = Phase::Judgment;
    assert!(validate_communication(&view, &d.choice).is_err());
}
#[test]
fn legitimate_relay_preserves_environment_root_and_does_not_acquire() {
    let mut c = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::AlwaysInvestigate);
    c.incidents.truncate(1);
    c.incidents[0].game.observations = [None; 6];
    c.incidents[0].artifacts[0].inspectors = vec![0];
    c.incidents[0].communication = [CommunicationKind::RelayFinding; 6];
    c.incidents[0].communication[0] = CommunicationKind::ShareFinding;
    let r = run(c).unwrap();
    assert_eq!(r.episodes[0].deliveries.len(), 1);
    assert!(r.episodes[0].reports.len() > 1);
    for a in r.episodes[0]
        .ancestry
        .iter()
        .filter(|a| a.report_basis.is_some())
    {
        assert_eq!(a.actual_roots, vec![Root::Artifact(0)]);
    }
    for report in r.episodes[0].reports.iter().skip(1) {
        assert!(matches!(report.basis, ReportBasis::Received(_)));
        assert_eq!(report.finding.acquisition.observer, 0);
    }
    assert!(r.ledgers[1..].iter().all(|l| l.spent == 0));
}
#[test]
fn environmental_reports_do_not_manufacture_witness_credibility() {
    let mut c = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::AlwaysInvestigate);
    c.incidents.truncate(1);
    c.incidents[0].game.observations = [None; 6];
    let r = run(c).unwrap();
    assert!(!r.episodes[0].reports.is_empty());
    for m in r.final_memory {
        assert!(m.local.records.is_empty());
        assert_eq!(m.local.credits, [e6::Credit::default(); 6]);
    }
}
#[test]
fn historical_records_remain_immutable_and_complete_replay_matches() {
    for d in [
        Diagnostic::Accurate,
        Diagnostic::Misleading,
        Diagnostic::TrustedWrong,
        Diagnostic::KnownDependent,
        Diagnostic::Permuted,
    ] {
        let c = diagnostic_config(d, 3, PolicyKind::SeededRandom);
        let a = run(c.clone()).unwrap();
        let b = run(c).unwrap();
        assert_eq!(a, b);
        assert_eq!(report_digests(&a).unwrap(), report_digests(&b).unwrap());
        for e in &a.episodes {
            for (i, t) in e.incident.testimony.iter().enumerate() {
                assert_eq!(t.message.id, i);
                assert_eq!(e.ancestry[i].message, i);
                assert!(t.lineage.evidence_parents.iter().all(|p| *p < i));
            }
        }
    }
}
#[test]
fn canonical_sections_cover_all_new_mechanisms_and_float_rejection() {
    let a = fixture(Diagnostic::Accurate, PolicyKind::AlwaysInvestigate);
    let mut b = a.clone();
    b.ledgers[0].remaining += 1;
    assert_ne!(report_digests(&a).unwrap(), report_digests(&b).unwrap());
    b = a.clone();
    b.episodes[0].deliveries[0].actual_root = Root::Observation(0);
    assert_ne!(canonical_digest(&a).unwrap(), canonical_digest(&b).unwrap());
    assert!(canonical_digest(&1.5).is_err());
    assert_ne!(
        canonical_digest(&a).unwrap(),
        e7::canonical_digest(&a).unwrap()
    );
}
#[test]
fn portability_comparator_does_not_accept_two_windows_or_changed_evidence() {
    let manifest = e7::ReplayManifest {
        version: "replay-manifest-008-v1".into(),
        series: 0,
        incidents: 0,
        evaluation_digest: "a".into(),
        metric_digest: "b".into(),
        series_sections: vec![],
    };
    let a = e7::PlatformReplay {
        execution: e7::Execution {
            os: "windows".into(),
            architecture: "x86_64".into(),
            rustc: "pinned".into(),
            cargo: "pinned".into(),
            revision: "same".into(),
            lock_digest: "same".into(),
        },
        manifest,
    };
    let mut b = a.clone();
    assert!(e7::compare_platforms(&a, &b).is_err());
    b.execution.os = "linux".into();
    e7::compare_platforms(&a, &b).unwrap();
    b.manifest.evaluation_digest = "tampered".into();
    assert!(e7::compare_platforms(&a, &b).is_err());
}
#[test]
fn full_preregistered_grid_counts_pairing_and_replay_contract() {
    let e = evaluate().unwrap();
    assert_eq!(e.rows.len(), 608);
    assert_eq!(e.incidents, 1824);
    assert_eq!(e.replay_matches, 608);
    assert_eq!(e.independent_worlds, 32);
    assert!(e.rows.iter().all(|r| r.metrics.rejected == 0));
    assert!(e.comparisons.iter().all(|c| c.worlds == 32
        && c.improved + c.worsened + c.unchanged == 32
        && c.coverage_interval.defined_resamples == 512));
    for seed in 2000..2032 {
        let rows: Vec<_> = e
            .rows
            .iter()
            .filter(|r| r.population == "heldout" && r.seed == seed)
            .collect();
        assert_eq!(rows.len(), 7);
        let roots: Vec<_> = rows[0].trace.iter().map(|t| t.actor).collect();
        assert!(
            rows.iter()
                .all(|r| r.trace.iter().map(|t| t.actor).collect::<Vec<_>>() == roots)
        );
    }
    assert_eq!(manifest(&e).unwrap().series_sections.len(), 608);
}
#[test]
fn hidden_actor_and_causal_root_changes_leave_investigation_inputs_identical() {
    let mut c = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::ConflictDriven);
    c.incidents.truncate(1);
    c.incidents[0].communication = [CommunicationKind::Silent; 6];
    let mut different = c.clone();
    different.incidents[0].artifacts[0].actual_root = Root::Observation(0);
    let actor = c.incidents[0].game.actor();
    while different.incidents[0].game.actor() == actor {
        different.incidents[0].game.seed += 1;
    }
    let a = run(c).unwrap();
    let b = run(different).unwrap();
    assert_ne!(
        a.episodes[0].incident.config.actor(),
        b.episodes[0].incident.config.actor()
    );
    assert_eq!(a.episodes[0].investigation, b.episodes[0].investigation);
    assert_ne!(
        a.episodes[0].deliveries[0].actual_root,
        b.episodes[0].deliveries[0].actual_root
    );
}
#[test]
fn executed_local_histories_change_acquisition_in_matched_present_worlds() {
    let mut c = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::CredibilityAware);
    c.incidents.truncate(2);
    for e in &mut c.incidents {
        e.game.observations = [None; 6];
        e.game.observations[0] = Some(e.game.actor());
        e.communication = [CommunicationKind::Sincere; 6];
    }
    c.incidents[0].artifacts.clear();
    let mut different = c.clone();
    different.incidents[0].game.observations[0] = Some((c.incidents[0].game.actor() + 1) % 6);
    assert_eq!(c.incidents[1], different.incidents[1]);
    let a = run(c).unwrap();
    let b = run(different).unwrap();
    assert!(a.episodes[1].memory_before[5].local.credits[0].score > 0);
    assert!(b.episodes[1].memory_before[5].local.credits[0].score < 0);
    let x = a.episodes[1]
        .investigation
        .iter()
        .find(|d| d.input.id == 5)
        .unwrap();
    let y = b.episodes[1]
        .investigation
        .iter()
        .find(|d| d.input.id == 5)
        .unwrap();
    assert_eq!(x.input.balance, y.input.balance);
    assert_eq!(x.input.catalog, y.input.catalog);
    assert_eq!(x.choice.action, InvestigationAction::Decline);
    assert_eq!(y.choice.action, InvestigationAction::Inspect(0));
}
#[test]
fn acquiring_evidence_changes_executed_messages_and_votes_in_fixed_world() {
    let c = diagnostic_config(Diagnostic::Accurate, 0, PolicyKind::NeverInvestigate);
    let mut d = c.clone();
    d.policy = [PolicyKind::AlwaysInvestigate; 6];
    assert_eq!(c.incidents, d.incidents);
    let a = run(c).unwrap();
    let b = run(d).unwrap();
    assert!(metrics(&b).unwrap().candidate_changes > 0);
    assert_ne!(
        a.episodes[0].incident.testimony,
        b.episodes[0].incident.testimony
    );
    assert_ne!(a.episodes[0].incident.tally, b.episodes[0].incident.tally);
}
#[test]
fn legitimate_participation_briefing_keeps_original_judgment_and_anchor_contract() {
    let mut c = diagnostic_config(Diagnostic::Baseline, 0, PolicyKind::NeverInvestigate);
    c.incidents.truncate(1);
    c.incidents[0].game.participation_briefing = true;
    c.incidents[0].communication = [CommunicationKind::Silent; 6];
    c.incidents[0].game.judgments = [e5::JudgmentKind::Original; 6];
    let actor = c.incidents[0].game.actor();
    c.incidents[0].game.observations[usize::from(actor)] = Some((actor + 1) % 6);
    let r = run(c).unwrap();
    let d = r.episodes[0]
        .incident
        .decisions
        .iter()
        .find(|d| d.input.phase == Phase::Judgment && d.input.id == actor)
        .unwrap();
    assert_eq!(d.input.original.belief, Some(actor));
    assert_eq!(d.choice.action, e5::Action::Vote(Some(actor)));
    assert_eq!(d.input.current.scores[usize::from(actor)], 6);
}
