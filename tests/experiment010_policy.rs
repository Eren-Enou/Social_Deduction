use social_deduction::{experiment009 as e9, experiment010::*};
use std::collections::BTreeMap;

fn view() -> InvestigationView {
    let r = e9::run(e9::diagnostic_config(
        e9::Diagnostic::EqualSources,
        0,
        e9::PolicyKind::NeverInvestigate,
    ))
    .unwrap();
    let local = r.episodes[0]
        .investigation
        .iter()
        .find(|i| i.input.id == 5)
        .unwrap()
        .input
        .clone();
    let methods = local
        .catalog
        .iter()
        .enumerate()
        .map(|(id, source)| PublicMethod {
            source: source.clone(),
            family: MethodId::new(format!("family-{id}")),
        })
        .collect();
    InvestigationView {
        local,
        methods,
        experience: BTreeMap::new(),
    }
}
fn choose(kind: PolicyKind, v: &InvestigationView) -> InvestigationChoice {
    Policy::new(kind, 0, v.local.id).choose(v)
}

#[test]
fn frozen009_choices_and_considered_values_ignore_learned_history() {
    use e9::InvestigationPolicy as _;
    let mut v = view();
    for kind in POLICIES.into_iter().filter(|p| !p.adaptive()) {
        let original = e9::Policy::new(kind.frozen(), 0, 5).choose(&v.local);
        v.experience.insert(
            v.methods[0].family.clone(),
            Counts {
                failure: 5,
                nonmissing: 5,
                ..Counts::default()
            },
        );
        let result = choose(kind, &v);
        assert_eq!(result.action, original.action);
        assert_eq!(result.considered, original.considered);
    }
}
#[test]
fn identical_current_inputs_different_local_experience_changes_target() {
    let mut v = view();
    v.experience.insert(
        v.methods[0].family.clone(),
        Counts {
            failure: 3,
            nonmissing: 3,
            ..Counts::default()
        },
    );
    v.experience.insert(
        v.methods[1].family.clone(),
        Counts {
            success: 3,
            nonmissing: 3,
            ..Counts::default()
        },
    );
    let before = v.local.clone();
    let a = choose(PolicyKind::ExperienceAdjusted, &v);
    assert_eq!(
        a.action,
        e9::InvestigationAction::Inspect(v.methods[1].source.descriptor.id)
    );
    v.experience.insert(
        v.methods[0].family.clone(),
        Counts {
            success: 3,
            nonmissing: 3,
            ..Counts::default()
        },
    );
    v.experience.insert(
        v.methods[1].family.clone(),
        Counts {
            failure: 3,
            nonmissing: 3,
            ..Counts::default()
        },
    );
    let b = choose(PolicyKind::ExperienceAdjusted, &v);
    assert_eq!(
        b.action,
        e9::InvestigationAction::Inspect(v.methods[0].source.descriptor.id)
    );
    assert_eq!(v.local, before);
    assert_ne!(a.expectations, b.expectations);
}
#[test]
fn zero_observations_mean_neutral_forecasts_and_sparse_conservative_uses_claims() {
    let v = view();
    let c = choose(PolicyKind::ExperienceAdjusted, &v);
    assert!(
        c.expectations
            .iter()
            .all(|e| e.learned_accuracy.numerator == 1 && e.learned_accuracy.denominator == 2)
    );
    let m = &v.methods[0];
    let sparse = Counts {
        success: 2,
        nonmissing: 2,
        ..Counts::default()
    };
    let e = expectation(PolicyKind::ConservativeLearner, m, sparse);
    assert!(!e.experienced_accuracy_used);
    assert!(!e.experienced_missing_used);
    assert_eq!(e.effective_accuracy, e.advertised_accuracy);
    let sufficient = Counts {
        success: 3,
        nonmissing: 3,
        ..Counts::default()
    };
    let e = expectation(PolicyKind::ConservativeLearner, m, sufficient);
    assert!(e.experienced_accuracy_used);
    assert_eq!(
        e.effective_accuracy,
        Probability {
            numerator: 5,
            denominator: 7
        }
    );
    assert_eq!(e.effective_accuracy_percent, 71);
    let missing_only = expectation(
        PolicyKind::ConservativeLearner,
        m,
        Counts {
            missing: 3,
            ..Counts::default()
        },
    );
    assert!(missing_only.experienced_missing_used);
    assert!(!missing_only.experienced_accuracy_used);
}
#[test]
fn paid_exploration_chooses_underobserved_legal_method_and_can_decline() {
    let mut v = view();
    v.experience.insert(
        v.methods[0].family.clone(),
        Counts {
            success: 4,
            nonmissing: 4,
            ..Counts::default()
        },
    );
    let mut p = Policy::new(PolicyKind::ExploratoryLearner, 0, 5);
    for _ in 0..3 {
        assert!(!p.choose(&v).exploration);
    }
    let c = p.choose(&v);
    assert!(c.exploration);
    assert_eq!(c.opportunity, 4);
    assert_eq!(
        c.action,
        e9::InvestigationAction::Inspect(v.methods[1].source.descriptor.id)
    );
    validate_investigation(&v, c.action).unwrap();
    v.local.balance = 0;
    p.opportunities = 3;
    let c = p.choose(&v);
    assert!(!c.exploration);
    assert_eq!(c.action, e9::InvestigationAction::Decline);
    v.local.stage = e9::Stage::Complete;
    let c = p.choose(&v);
    assert_eq!(c.action, e9::InvestigationAction::Decline);
}
#[test]
fn semantic_ties_and_source_order_permutation_preserve_adaptive_choices() {
    let mut v = view();
    let before = choose(PolicyKind::ExperienceAdjusted, &v);
    v.methods.reverse();
    v.local.catalog.reverse();
    let after = choose(PolicyKind::ExperienceAdjusted, &v);
    assert_eq!(before.action, after.action);
    v.validate().unwrap();
    v.methods[0].family = MethodId::new("malformed id");
    assert!(v.validate().is_err());
}
#[test]
fn declarations_affect_conservative_choices_while_no_hidden_quality_is_present() {
    let mut v = view();
    for m in &mut v.methods {
        m.source.declared_accuracy = 100;
        m.source.declared_missing = 0;
    }
    v.local.catalog = v.methods.iter().map(|m| m.source.clone()).collect();
    let a = choose(PolicyKind::ConservativeLearner, &v);
    v.methods[0].source.declared_accuracy = 0;
    v.local.catalog[0].declared_accuracy = 0;
    let b = choose(PolicyKind::ConservativeLearner, &v);
    assert_ne!(a.action, b.action);
    let bytes = serde_json::to_string(&v).unwrap();
    for forbidden in [
        "actual_root",
        "candidate_accuracy",
        "actual_accuracy",
        "regime_schedule",
        "objective_history",
    ] {
        assert!(!bytes.contains(forbidden));
    }
    assert_eq!(
        choose(PolicyKind::ExperienceAdjusted, &v),
        choose(PolicyKind::ExperienceAdjusted, &v)
    );
}
