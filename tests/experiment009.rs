use social_deduction::{
    Phase, experiment005 as e5, experiment006 as e6, experiment007 as e7, experiment009::*,
};
fn evidence() -> EvidenceView {
    EvidenceView {
        incident: e5::ListenerView {
            id: 5,
            original: e5::IndividualState {
                observation: None,
                briefing: None,
                belief: None,
                incentives: e5::Incentives::ACCURACY,
            },
            observations: vec![e5::Observation {
                acquisition: 5,
                candidate: None,
            }],
            prior: e5::Belief::from_scores([0; 6]),
            received: vec![],
        },
        credits: [e6::Credit::default(); 6],
        findings: vec![],
        reports: vec![],
    }
}
fn view() -> InvestigationView {
    InvestigationView {
        id: 5,
        episode: 0,
        episodes_remaining: 2,
        evidence: evidence(),
        belief: e5::Belief::from_scores([0; 6]),
        testimony: e6::ListenerKind::Credibility,
        catalog: vec![
            source(0, Some(1), 1).public(),
            source(1, Some(2), 2).public(),
        ],
        balance: 3,
        stage: Stage::Investigation,
        used: false,
    }
}
fn choose(p: PolicyKind, v: &InvestigationView) -> InvestigationChoice {
    Policy::new(p, 0, 5).choose(v)
}
fn first(c: Config) -> Report {
    run(c).unwrap()
}
fn single(p: PolicyKind) -> Config {
    let mut c = diagnostic_config(Diagnostic::QualityVsCost, 0, p);
    c.incidents.truncate(1);
    c
}
fn assess(v: &EvidenceView) -> Assessment {
    EvidenceAware {
        testimony: e6::ListenerKind::Credibility,
    }
    .assess(v)
}
#[test]
fn generation_precedes_treatment_and_preserves_latent_worlds() {
    for seed in 0..16 {
        let w = Generator.generate(seed);
        assert_eq!(w, Generator.generate(seed));
        for p in POLICIES {
            let c = generated_config(&w, p, false);
            assert_eq!(c.incidents, w.base.incidents);
            assert_eq!(c.budgets, w.base.budgets);
            c.validate().unwrap();
        }
    }
}
#[test]
fn hidden_content_truth_and_ancestry_do_not_change_choices() {
    for p in POLICIES {
        let mut c = single(p);
        c.incidents[0].communication = [CommunicationKind::Silent; 6];
        let mut changed = c.clone();
        changed.incidents[0].artifacts[0].candidate = Some(5);
        changed.incidents[0].artifacts[1].actual_root = Root::Observation(0);
        changed.incidents[0].artifacts[1].candidate = changed.incidents[0].game.observations[0];
        let a = run(c.clone()).unwrap();
        let b = run(changed).unwrap();
        assert_eq!(a.episodes[0].investigation, b.episodes[0].investigation);
        c.incidents[0].game.seed += 1;
        let other = run(c).unwrap();
        assert_eq!(a.episodes[0].investigation, other.episodes[0].investigation);
        let json = serde_json::to_value(&a.episodes[0].investigation[0].input).unwrap();
        assert!(json.get("actual_root").is_none());
        assert!(json.get("actor").is_none());
        assert!(json["catalog"][0].get("candidate").is_none());
        assert!(json["catalog"][0].get("actual_root").is_none());
    }
}
#[test]
fn no_uninspected_content_or_cross_recipient_delivery() {
    let a = first(single(PolicyKind::NeverInvestigate));
    assert!(a.episodes[0].deliveries.is_empty());
    assert!(
        a.episodes[0]
            .cognition
            .iter()
            .all(|x| x.input.findings.is_empty())
    );
    let mut c = single(PolicyKind::FirstAccessible);
    for s in &mut c.incidents[0].artifacts {
        s.inspectors = vec![5];
    }
    c.incidents[0].communication = [CommunicationKind::Silent; 6];
    let r = first(c);
    assert!(
        r.episodes[0]
            .cognition
            .iter()
            .filter(|x| x.input.incident.id != 5)
            .all(|x| x.input.findings.is_empty() && x.input.reports.is_empty())
    );
    assert!(
        r.episodes[0]
            .communication
            .iter()
            .take(6)
            .all(|x| x.input.findings.is_empty())
    );
}
#[test]
fn legal_alternatives_and_voluntary_decline() {
    let v = view();
    assert_eq!(v.catalog.len(), 2);
    for a in [
        InvestigationAction::Decline,
        InvestigationAction::Inspect(0),
        InvestigationAction::Inspect(1),
    ] {
        validate_investigation(&v, a).unwrap();
    }
    assert_eq!(
        choose(PolicyKind::NeverInvestigate, &v).action,
        InvestigationAction::Decline
    );
}
#[test]
fn visible_quality_and_cost_change_target() {
    let mut v = view();
    v.episodes_remaining = 0;
    assert_eq!(
        choose(PolicyKind::InformationValueHeuristic, &v).action,
        InvestigationAction::Inspect(1)
    );
    v.catalog[1].declared_accuracy = 10;
    assert_eq!(
        choose(PolicyKind::InformationValueHeuristic, &v).action,
        InvestigationAction::Inspect(0)
    );
    v.catalog[1].declared_accuracy = 90;
    v.catalog[1].cost = 3;
    v.balance = 2;
    assert_eq!(
        choose(PolicyKind::InformationValueHeuristic, &v).action,
        InvestigationAction::Inspect(0)
    );
    v.balance = 3;
    v.catalog[0].declared_accuracy = 90;
    v.catalog[0].declared_missing = 0;
    v.catalog[0].cost = 3;
    v.catalog[1].cost = 1;
    assert_eq!(
        choose(PolicyKind::InformationValueHeuristic, &v).action,
        InvestigationAction::Inspect(1)
    );
}
#[test]
fn relevance_uses_only_described_competing_candidates() {
    let mut v = view();
    v.belief = e5::Belief::from_scores([2, 2, 0, 0, 0, 0]);
    v.evidence.incident.original.observation = Some(0);
    v.evidence.incident.observations[0].candidate = Some(0);
    for s in &mut v.catalog {
        s.declared_accuracy = 90;
        s.declared_missing = 0;
    }
    v.catalog[0].coverage = vec![4, 5];
    v.catalog[1].coverage = vec![0, 1];
    assert_eq!(
        choose(PolicyKind::ConflictDrivenTarget, &v).action,
        InvestigationAction::Inspect(1)
    );
    v.catalog[0].coverage = vec![0, 1];
    v.catalog[1].coverage = vec![4, 5];
    assert_eq!(
        choose(PolicyKind::ConflictDrivenTarget, &v).action,
        InvestigationAction::Inspect(0)
    );
}
#[test]
fn order_and_source_ids_do_not_affect_semantic_ties() {
    let mut v = view();
    v.catalog[1].cost = 1;
    v.catalog[1].category = v.catalog[0].category;
    v.catalog[1].declared_accuracy = v.catalog[0].declared_accuracy;
    v.catalog[1].declared_missing = v.catalog[0].declared_missing;
    for p in [
        PolicyKind::CheapestAvailable,
        PolicyKind::ConflictDrivenTarget,
        PolicyKind::InformationValueHeuristic,
        PolicyKind::BudgetAwareTarget,
    ] {
        let a = choose(p, &v);
        let mut reversed = v.clone();
        reversed.catalog.reverse();
        assert_eq!(a.action, choose(p, &reversed).action);
        for s in &mut reversed.catalog {
            s.descriptor.id += 20;
        }
        assert_eq!(
            choose(p, &reversed).action,
            InvestigationAction::Inspect(20)
        );
    }
    v.catalog.reverse();
    assert_eq!(
        choose(PolicyKind::FirstAccessible, &v).action,
        InvestigationAction::Inspect(1)
    );
}
#[test]
fn random_target_has_own_repeatable_stream_and_always_chooses_affordable() {
    let mut v = view();
    let mut a = Policy::new(PolicyKind::SeededRandomTarget, 3, 5);
    let mut b = Policy::new(PolicyKind::SeededRandomTarget, 3, 5);
    let mut targets = std::collections::BTreeSet::new();
    for _ in 0..32 {
        let x = a.choose(&v);
        assert_eq!(x, b.choose(&v));
        validate_investigation(&v, x.action).unwrap();
        if let InvestigationAction::Inspect(id) = x.action {
            targets.insert(id);
        } else {
            panic!("random target is not random whether");
        }
    }
    assert_eq!(targets.len(), 2);
    v.balance = 1;
    for _ in 0..8 {
        assert_eq!(a.choose(&v).action, InvestigationAction::Inspect(0));
    }
}
#[test]
fn known_overlap_changes_expected_value_hidden_overlap_is_absent() {
    let mut v = view();
    v.evidence.incident.received = vec![e5::Message {
        id: 0,
        speaker: 0,
        body: e5::Body::Assert(1),
        attribution: e5::Attribution::Firsthand,
    }];
    v.belief = assess(&v.evidence).after;
    v.catalog[1].coverage = vec![1, 2];
    let unknown = estimate(&v, &v.catalog[1]);
    assert!(!unknown.known_overlap);
    assert!(unknown.usefulness > 0);
    v.catalog[1].descriptor.visible_parent = Some(0);
    let known = estimate(&v, &v.catalog[1]);
    assert!(known.known_overlap);
    assert_eq!(known.outcomes.len(), 1);
    assert_eq!(known.outcomes[0].candidate, 1);
    assert_eq!(known.usefulness, 0);
}
#[test]
fn strong_belief_conserves_and_remaining_opportunities_change_gate() {
    let mut v = view();
    v.evidence.incident.original.observation = Some(0);
    v.evidence.incident.observations[0].candidate = Some(0);
    v.evidence.incident.original.briefing = Some(e5::Briefing::Participated);
    v.belief = assess(&v.evidence).after;
    assert_eq!(
        choose(PolicyKind::InformationValueHeuristic, &v).action,
        InvestigationAction::Decline
    );
    let mut v = view();
    v.balance = 1;
    v.catalog.truncate(1);
    v.catalog[0].declared_accuracy = 25;
    v.catalog[0].declared_missing = 0;
    v.episodes_remaining = 2;
    assert_eq!(
        choose(PolicyKind::BudgetAwareTarget, &v).action,
        InvestigationAction::Decline
    );
    v.episodes_remaining = 0;
    assert_eq!(
        choose(PolicyKind::BudgetAwareTarget, &v).action,
        InvestigationAction::Inspect(0)
    );
}
#[test]
fn resolver_enforces_one_two_three_actual_units() {
    for cost in 1..=3 {
        let mut c = single(PolicyKind::FirstAccessible);
        c.budgets = [cost; 6];
        c.incidents[0].artifacts.truncate(1);
        c.incidents[0].artifacts[0].cost = cost;
        let r = first(c);
        assert!(
            r.ledgers
                .iter()
                .all(|l| l.initial == cost && l.spent == cost && l.remaining == 0)
        );
        assert!(
            r.episodes[0]
                .deliveries
                .iter()
                .all(|d| d.balance_before - d.balance_after == cost)
        );
    }
}
#[test]
fn missing_and_misleading_still_pay_expensive_price() {
    for candidate in [None, Some(5)] {
        let mut c = single(PolicyKind::FirstAccessible);
        c.incidents[0].artifacts.truncate(1);
        c.incidents[0].artifacts[0].cost = 3;
        c.incidents[0].artifacts[0].candidate = candidate;
        let r = first(c);
        assert!(r.ledgers.iter().all(|l| l.spent == 3));
        assert!(
            r.episodes[0]
                .deliveries
                .iter()
                .all(|d| d.finding.candidate == candidate)
        );
    }
}
struct Illegal;
impl InvestigationPolicy for Illegal {
    fn choose(&mut self, _: &InvestigationView) -> InvestigationChoice {
        InvestigationChoice {
            action: InvestigationAction::Inspect(1),
            considered: vec![],
            mechanism: "deliberately unaffordable".into(),
        }
    }
}
#[test]
fn unaffordable_rejection_has_no_fallback_or_retry() {
    let mut c = single(PolicyKind::FirstAccessible);
    c.budgets = [1; 6];
    let mut p = Components::from_config(&c);
    p.investigators = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn InvestigationPolicy>)
        .collect();
    let r = run_with(c, &mut p).unwrap();
    assert!(r.ledgers.iter().all(|l| l.spent == 0));
    assert!(r.episodes[0].deliveries.is_empty());
    assert_eq!(r.episodes[0].investigation.len(), 6);
    assert!(
        r.episodes[0]
            .investigation
            .iter()
            .all(|d| d.rejection.is_some())
    );
    assert_eq!(r.episodes[0].incident.decisions.len(), 18);
}
#[test]
fn legal_stage_identity_access_exhaustion_and_duplicates() {
    let mut v = view();
    for balance in [0, 1] {
        v.balance = balance;
        assert!(validate_investigation(&v, InvestigationAction::Inspect(1)).is_err());
        validate_investigation(&v, InvestigationAction::Decline).unwrap();
    }
    v.balance = 3;
    for stage in [
        Stage::FirstDiscussion,
        Stage::SecondDiscussion,
        Stage::Judgment,
        Stage::Complete,
    ] {
        v.stage = stage;
        assert!(validate_investigation(&v, InvestigationAction::Decline).is_err());
    }
    v.stage = Stage::Investigation;
    v.used = true;
    assert!(validate_investigation(&v, InvestigationAction::Inspect(0)).is_err());
    v.used = false;
    assert!(validate_investigation(&v, InvestigationAction::Inspect(99)).is_err());
    v.id = 6;
    assert!(validate_investigation(&v, InvestigationAction::Decline).is_err());
}
#[test]
fn bounded_malformed_catalog_and_cost_overflow_rejected() {
    let c = single(PolicyKind::FirstAccessible);
    for cost in [0, 4, 255] {
        let mut bad = c.clone();
        bad.incidents[0].artifacts[0].cost = cost;
        assert!(run(bad).is_err());
    }
    let mut bad = c.clone();
    bad.budgets = [255; 6];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.incidents[0].artifacts[1].descriptor.id = 0;
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.incidents[0].artifacts[1].method_key = "method-0".into();
    assert!(run(bad).is_err());
    for text in ["".to_string(), " ".to_string(), "x".repeat(129)] {
        let mut bad = c.clone();
        bad.incidents[0].artifacts[0].descriptor.description = text;
        assert!(run(bad).is_err());
    }
    for coverage in [vec![], vec![0, 0], vec![6]] {
        let mut bad = c.clone();
        bad.incidents[0].artifacts[0].coverage = coverage;
        assert!(run(bad).is_err());
    }
    let mut bad = c.clone();
    bad.incidents[0].artifacts[0].declared_accuracy = 101;
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.incidents[0].artifacts[0].descriptor.visible_parent = Some(6);
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.incidents[0].forced = vec![(5, InvestigationAction::Inspect(99))];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.incidents[0].injected = vec![(5, 0), (5, 0)];
    assert!(run(bad).is_err());
    let mut bad = c;
    bad.incidents[0].artifacts[0].inspectors = vec![5, 5];
    assert!(run(bad).is_err());
}
#[test]
fn no_affordable_catalog_declines_without_fabricating_source() {
    let mut v = view();
    v.balance = 0;
    for p in POLICIES {
        assert_eq!(choose(p, &v).action, InvestigationAction::Decline);
    }
    v.catalog.clear();
    v.balance = 9;
    for p in POLICIES {
        assert_eq!(choose(p, &v).action, InvestigationAction::Decline);
    }
}
#[test]
fn conserved_resources_preserve_later_legal_options() {
    let a = first(diagnostic_config(
        Diagnostic::ResourceScarcity,
        0,
        PolicyKind::FirstAccessible,
    ));
    let b = first(diagnostic_config(
        Diagnostic::ResourceScarcity,
        0,
        PolicyKind::BudgetAwareTarget,
    ));
    assert!(
        b.episodes[0]
            .investigation
            .iter()
            .all(|d| d.choice.action == InvestigationAction::Decline)
    );
    assert!(a.ledgers.iter().all(|l| l.initial == l.spent + l.remaining));
    assert!(b.ledgers.iter().all(|l| l.initial == l.spent + l.remaining));
    assert!(
        b.episodes[1]
            .investigation
            .iter()
            .all(|d| d.input.balance == 2)
    );
    assert!(
        a.episodes[1]
            .investigation
            .iter()
            .all(|d| d.input.balance == 1)
    );
    assert!(b.episodes[1].investigation.iter().any(|d| {
        d.input
            .catalog
            .iter()
            .filter(|s| s.cost <= d.input.balance)
            .count()
            > a.episodes[1]
                .investigation
                .iter()
                .find(|x| x.input.id == d.input.id)
                .unwrap()
                .input
                .catalog
                .iter()
                .filter(|s| s.cost <= 1)
                .count()
    }));
}
#[test]
fn duplicate_artifact_and_shared_root_do_not_manufacture_support() {
    let mut v = evidence();
    let f = Finding {
        source: source(0, None, 1).descriptor,
        candidate: Some(1),
        acquisition: Acquisition {
            episode: 0,
            observer: 5,
            local: 0,
        },
    };
    v.findings = vec![f.clone()];
    let a = assess(&v);
    let mut copy = f;
    copy.acquisition.observer = 4;
    copy.acquisition.local = 1;
    v.findings.push(copy);
    assert_eq!(a.after, assess(&v).after);
    assert_eq!(a.after.scores[1], 2);
    v.incident.received = vec![e5::Message {
        id: 0,
        speaker: 0,
        body: e5::Body::Assert(1),
        attribution: e5::Attribution::Firsthand,
    }];
    for f in &mut v.findings {
        f.source.visible_parent = Some(0);
    }
    assert_eq!(assess(&v).after.scores[1], 2);
    for f in &mut v.findings {
        f.source.visible_parent = None;
    }
    assert_eq!(assess(&v).after.scores[1], 3);
}
#[test]
fn selected_forced_and_injected_have_matching_information_distinct_origins() {
    for c in controls().unwrap() {
        let [a, b, i]: [_; 3] = c.deliveries.clone().try_into().unwrap();
        assert_eq!(a[0].finding, b[0].finding);
        assert_eq!(a[0].finding, i[0].finding);
        assert_eq!(a[0].after_assessment, i[0].after_assessment);
        assert_eq!(a[0].balance_before - a[0].balance_after, 1);
        assert_eq!(b[0].balance_before - b[0].balance_after, 1);
        assert_eq!(i[0].balance_before, i[0].balance_after);
        assert!(matches!(a[0].origin, DeliveryOrigin::Selected(_)));
        assert!(matches!(b[0].origin, DeliveryOrigin::Forced(_)));
        assert!(matches!(i[0].origin, DeliveryOrigin::Intervention(_)));
        assert_eq!(c.metrics[0].selected, 1);
        assert_eq!(c.metrics[1].selected, 0);
    }
}
#[test]
fn identity_permutation_preserves_causal_candidates_and_source_references() {
    for p in [
        PolicyKind::FirstAccessible,
        PolicyKind::InformationValueHeuristic,
        PolicyKind::BudgetAwareTarget,
    ] {
        let a = first(diagnostic_config(Diagnostic::QualityVsCost, 0, p));
        let b = first(diagnostic_config(
            Diagnostic::SourceIdentityPermutation,
            0,
            p,
        ));
        for (x, y) in a.episodes.iter().zip(&b.episodes) {
            assert_eq!(
                (x.incident.config.actor() + 3) % 6,
                y.incident.config.actor()
            );
            for d in &x.investigation {
                let mapped = y
                    .investigation
                    .iter()
                    .find(|v| v.input.id == (d.input.id + 3) % 6)
                    .unwrap();
                assert_eq!(d.choice.action, mapped.choice.action.map_source_back());
            }
        }
    }
}
trait MapSource {
    fn map_source_back(self) -> Self;
}
impl MapSource for InvestigationAction {
    fn map_source_back(self) -> Self {
        match self {
            Self::Inspect(id) => Self::Inspect(id - 10),
            x => x,
        }
    }
}
#[test]
fn immutable_earlier_snapshots_namespaces_legal_ballots_and_termination() {
    let c = diagnostic_config(
        Diagnostic::MisleadingExpensive,
        0,
        PolicyKind::InformationValueHeuristic,
    );
    let r = first(c.clone());
    assert_eq!(r, first(c));
    assert_eq!(report_digests(&r).unwrap().len(), 6);
    for (idx, e) in r.episodes.iter().enumerate() {
        assert_eq!(e.incident.decisions.len(), 18);
        assert_eq!(e.investigation.len(), 6);
        assert!(
            e.incident
                .decisions
                .iter()
                .filter(|d| d.input.phase == Phase::Judgment)
                .all(|d| d.rejection.is_none())
        );
        assert!(
            e.incident
                .objective_history
                .iter()
                .any(|ev| matches!(ev, e5::Event::Transition(Phase::Terminated)))
        );
        for d in &e.deliveries {
            assert_eq!(d.finding.acquisition.episode, idx as u8);
            assert_eq!(d.before.incident.received, d.after.incident.received);
            assert_eq!(d.before.incident.original, d.after.incident.original);
        }
        for decision in e.incident.decisions.iter().take(6) {
            assert_eq!(decision.input.observations.len(), 1);
        }
    }
}
#[test]
fn observer_counterfactuals_do_not_enter_policy_views() {
    let c = counterfactuals().unwrap();
    assert_eq!(c.len(), 64);
    for row in &c {
        assert!(row.alternatives.iter().all(|a| a.policy.is_none()));
        assert_eq!(row.selections.len(), 2);
        assert!(
            row.alternatives
                .iter()
                .flat_map(|a| &a.regret_millionths)
                .all(|x| *x >= 0)
        );
        assert!(
            row.selections
                .iter()
                .flat_map(|a| &a.regret_millionths)
                .all(|x| *x >= 0)
        );
    }
}
#[test]
fn canonical_integer_encoding_and_section_changes() {
    let a = first(single(PolicyKind::FirstAccessible));
    let mut b = a.clone();
    b.episodes[0].investigation[0].input.catalog[0].cost = 3;
    assert_ne!(
        report_digests(&a).unwrap()["investigation_deliveries"],
        report_digests(&b).unwrap()["investigation_deliveries"]
    );
    assert_eq!(
        e7::canonical_bytes(&serde_json::json!({"b":2,"a":1})).unwrap(),
        br#"{"a":1,"b":2}"#
    );
    assert!(canonical_digest(&1.5).is_err());
}
#[test]
fn full_preregistered_grid_replays_and_pairs_world_units() {
    let e = evaluate().unwrap();
    assert_eq!(e.independent_worlds, 64);
    assert_eq!(e.rows.len(), 960);
    assert_eq!(e.replay_matches, 960);
    assert_eq!(e.incidents, 2880);
    assert_eq!(e.controls.len(), 4);
    assert_eq!(e.counterfactuals.len(), 64);
    assert!(e.comparisons.iter().all(|c| c.worlds == 64
        && c.improved + c.worsened + c.unchanged == 64
        && c.accuracy_interval.defined_resamples > 0));
    assert_eq!(
        manifest(&e).unwrap().evaluation_digest,
        canonical_digest(&e).unwrap()
    );
    for row in &e.rows {
        if row.population == "heldout" {
            assert!(
                e.rows
                    .iter()
                    .filter(|other| other.population == "heldout" && other.seed == row.seed)
                    .all(|other| other.environment_digest == row.environment_digest)
            );
        }
        assert_eq!(row.metrics.rejected, 0);
        assert_eq!(row.metrics.opportunities, 18);
        assert_eq!(
            row.metrics.selected_costs.iter().sum::<u64>(),
            row.metrics.selected
        );
        assert_eq!(
            row.metrics.correct_ballots + row.metrics.wrong_ballots + row.metrics.abstentions,
            18
        );
    }
}
#[test]
fn public_separate_acquisition_is_legitimate_optional_metadata() {
    let s = source(0, Some(1), 1);
    assert_eq!(s.public().provenance, PublicProvenance::SeparateAcquisition);
    let mut c = single(PolicyKind::FirstAccessible);
    c.incidents[0].artifacts[0].actual_root = Root::Observation(0);
    c.incidents[0].artifacts[0].candidate = c.incidents[0].game.observations[0];
    assert!(c.validate().is_err());
    c.incidents[0].artifacts[0].separate_acquisition_disclosed = false;
    c.validate().unwrap();
    assert_eq!(
        c.incidents[0].artifacts[0].public().provenance,
        PublicProvenance::Unknown
    );
}
#[test]
fn new_scheduler_preserves_majority_ties_abstention_and_bounded_turns() {
    for agreeing in [0, 3, 4, 6] {
        let mut c = single(PolicyKind::NeverInvestigate);
        let actor = c.incidents[0].game.actor();
        c.incidents[0].communication = [CommunicationKind::Silent; 6];
        c.incidents[0].game.observations = [None; 6];
        for id in 0..agreeing {
            c.incidents[0].game.observations[id] = Some(actor);
        }
        if agreeing == 3 {
            for id in 3..6 {
                c.incidents[0].game.observations[id] = Some((actor + 1) % 6);
            }
        }
        let r = first(c);
        assert_eq!(
            r.episodes[0].incident.verdict,
            if agreeing >= 4 { Some(actor) } else { None }
        );
        assert_eq!(r.episodes[0].incident.decisions.len(), 18);
        assert_eq!(r.episodes[0].investigation.len(), 6);
    }
}
#[test]
fn source_order_fixture_changes_first_accessible_only_when_ties_require_it() {
    for p in [
        PolicyKind::FirstAccessible,
        PolicyKind::CheapestAvailable,
        PolicyKind::ConflictDrivenTarget,
        PolicyKind::InformationValueHeuristic,
        PolicyKind::BudgetAwareTarget,
    ] {
        let mut a = single(p);
        let mut b = a.clone();
        b.incidents[0].artifacts.reverse();
        a.incidents[0].communication = [CommunicationKind::Silent; 6];
        b.incidents[0].communication = [CommunicationKind::Silent; 6];
        let x = first(a);
        let y = first(b);
        for (a, b) in x.episodes[0]
            .investigation
            .iter()
            .zip(&y.episodes[0].investigation)
        {
            assert_eq!(a.input.belief, b.input.belief);
            if p == PolicyKind::FirstAccessible {
                assert_ne!(a.choice.action, b.choice.action);
            } else {
                assert_eq!(a.choice.action, b.choice.action);
            }
        }
    }
}
#[test]
fn changed_source_description_changes_selection_in_executed_matched_incident() {
    let mut a = single(PolicyKind::InformationValueHeuristic);
    a.incidents[0].communication = [CommunicationKind::Silent; 6];
    a.incidents[0].game.observations = [None; 6];
    let mut b = a.clone();
    b.incidents[0].artifacts[1].declared_accuracy = 5;
    let x = first(a);
    let y = first(b);
    for (a, b) in x.episodes[0]
        .investigation
        .iter()
        .zip(&y.episodes[0].investigation)
    {
        assert_eq!(a.assessment_input, b.assessment_input);
        assert_eq!(a.input.belief, b.input.belief);
        assert_eq!(a.input.balance, b.input.balance);
        assert_eq!(a.choice.action, InvestigationAction::Inspect(1));
        assert_eq!(b.choice.action, InvestigationAction::Inspect(0));
    }
}
#[test]
fn cli_scenario_reproduces_library_and_refuses_overwrite() {
    let root = std::env::temp_dir().join(format!("social-deduction009-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let config = root.join("config.json");
    let report = root.join("report.json");
    let bin = env!("CARGO_BIN_EXE_experiment009");
    assert!(
        std::process::Command::new(bin)
            .args([
                "scenario",
                "diagnostic",
                "MissingExpensive",
                "7",
                "FirstAccessible"
            ])
            .arg(&config)
            .status()
            .unwrap()
            .success()
    );
    let c: Config = serde_json::from_slice(&std::fs::read(&config).unwrap()).unwrap();
    assert_eq!(
        c,
        diagnostic_config(Diagnostic::MissingExpensive, 7, PolicyKind::FirstAccessible)
    );
    assert!(
        std::process::Command::new(bin)
            .arg("run")
            .arg(&config)
            .arg(&report)
            .status()
            .unwrap()
            .success()
    );
    let bytes = std::fs::read(&report).unwrap();
    assert_eq!(bytes, serde_json::to_vec(&first(c)).unwrap());
    assert!(
        !std::process::Command::new(bin)
            .arg("run")
            .arg(&config)
            .arg(&report)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(bytes, std::fs::read(&report).unwrap());
    std::fs::remove_file(config).unwrap();
    std::fs::remove_file(report).unwrap();
    std::fs::remove_dir(root).unwrap();
}
