use social_deduction::{
    experiment007 as e7, experiment008 as e8, experiment009 as e9, experiment010::*,
};

fn acquired(memory: MethodMemory, episode: u8, candidate: Option<u8>) -> MethodIngestion {
    CountUpdater
        .ingest(&MethodIngestView {
            memory,
            method: MethodId::new("instrument-a"),
            finding: e8::Finding {
                source: e8::Descriptor {
                    id: 7,
                    description: "fallible recurring method".into(),
                    visible_parent: None,
                },
                candidate,
                acquisition: e8::Acquisition {
                    episode,
                    observer: 5,
                    local: 0,
                },
            },
            origin: e9::DeliveryOrigin::Selected(0),
        })
        .unwrap()
}
fn verified(memory: MethodMemory, about: u8, completed: u8, actor: u8) -> MethodFeedback {
    CountUpdater
        .verify(&MethodFeedbackView {
            memory,
            received: e7::Token::outcome(10, about, 0, actor),
            delivery: usize::from(completed),
            completed_episode: completed,
            session: 10,
        })
        .unwrap()
}
fn counts(m: &MethodMemory) -> Counts {
    m.summaries[&MethodId::new("instrument-a")]
}

#[test]
fn exact_neutral_prior_and_missingness_without_accuracy() {
    assert_eq!(
        Counts::default().accuracy(1),
        e7::Probability {
            numerator: 1,
            denominator: 2
        }
    );
    let m = acquired(MethodMemory::neutral(5), 0, None).after;
    assert_eq!(counts(&m).graded(), 0);
    assert_eq!(
        counts(&m).missingness(1),
        e7::Probability {
            numerator: 2,
            denominator: 3
        }
    );
    let feedback = verified(m, 0, 0, 2);
    assert_eq!(counts(&feedback.after).graded(), 0);
    assert!(
        feedback
            .transport_input
            .memory
            .local
            .records
            .iter()
            .all(|r| r.source.is_none() && r.final_candidate.is_none())
    );
}
#[test]
fn accuracy_requires_accepted_verification_and_repeats_are_idempotent() {
    let m = acquired(MethodMemory::neutral(5), 0, Some(2)).after;
    assert_eq!(counts(&m).success, 0);
    assert_eq!(counts(&m).nonmissing, 1);
    let f = verified(m, 0, 0, 2);
    assert_eq!(counts(&f.after).success, 1);
    let repeated = verified(f.after.clone(), 0, 0, 2);
    assert_eq!(
        repeated.transport_effect.acceptance,
        e7::Acceptance::Repeated
    );
    assert_eq!(repeated.after.summaries, f.after.summaries);
    let success = counts(&f.after).accuracy(1);
    assert_eq!(
        success,
        e7::Probability {
            numerator: 2,
            denominator: 3
        }
    );
}
#[test]
fn authenticated_false_feedback_can_corrupt_and_retained_conflict_quarantines() {
    // The updater receives the token, never a factual truth or error flag.
    let m = acquired(MethodMemory::neutral(5), 0, Some(2)).after;
    let false_feedback = verified(m, 0, 0, 3);
    assert_eq!(counts(&false_feedback.after).failure, 1);
    let snapshot = false_feedback.after.clone();
    let conflict = verified(snapshot.clone(), 0, 0, 2);
    assert_eq!(
        conflict.transport_effect.acceptance,
        e7::Acceptance::Quarantined
    );
    assert_eq!(counts(&conflict.after).graded(), 0);
    assert_eq!(counts(&conflict.after).observed(), 1);
    assert_eq!(counts(&snapshot).failure, 1);
}
#[test]
fn undelivered_unauthenticated_malformed_and_absent_history_do_not_grade() {
    let m = acquired(MethodMemory::neutral(5), 0, Some(2)).after;
    let mut v = MethodFeedbackView {
        memory: m.clone(),
        received: e7::Token::outcome(10, 0, 0, 2),
        delivery: 0,
        completed_episode: 0,
        session: 10,
    };
    v.received.authenticated_channel = false;
    let f = CountUpdater.verify(&v).unwrap();
    assert_eq!(
        f.transport_effect.acceptance,
        e7::Acceptance::RejectedAuthentication
    );
    assert_eq!(counts(&f.after).graded(), 0);
    v.received.authenticated_channel = true;
    v.received.session = 11;
    let f = CountUpdater.verify(&v).unwrap();
    assert_eq!(
        f.transport_effect.acceptance,
        e7::Acceptance::RejectedAssociation
    );
    v.memory = MethodMemory::neutral(5);
    v.received.session = 10;
    let f = CountUpdater.verify(&v).unwrap();
    assert_eq!(f.transport_effect.acceptance, e7::Acceptance::Pending);
    assert!(f.after.summaries.is_empty());
    let later = acquired(f.after, 0, Some(2)).after;
    assert_eq!(counts(&later).graded(), 0);
}
#[test]
fn delayed_verification_only_grades_legitimately_retained_records() {
    let m = acquired(MethodMemory::neutral(5), 0, Some(2)).after;
    let m = acquired(m, 1, Some(3)).after;
    let f = verified(m, 0, 1, 2);
    assert_eq!(counts(&f.after).success, 1);
    assert_eq!(counts(&f.after).nonmissing, 2);
    assert!(matches!(
        f.after.records[1].status,
        ExperienceStatus::Pending
    ));
}
#[test]
fn duplicate_acquisition_and_visible_shared_origin_do_not_multiply_samples() {
    let result = acquired(MethodMemory::neutral(5), 0, Some(2));
    let result = CountUpdater
        .ingest(&MethodIngestView {
            memory: result.after,
            ..result.input
        })
        .unwrap();
    assert!(result.duplicate);
    assert_eq!(counts(&result.after).observed(), 1);
    let mut v = result.input;
    v.memory = result.after;
    v.finding.acquisition.local = 1;
    let duplicate = CountUpdater.ingest(&v).unwrap();
    assert!(duplicate.duplicate);
    assert_eq!(counts(&duplicate.after).observed(), 1);
    let mut original = v.clone();
    original.memory = MethodMemory::neutral(5);
    original.finding.acquisition.local = 0;
    original.finding.source.visible_parent = Some(0);
    let first = CountUpdater.ingest(&original).unwrap();
    v.memory = first.after;
    v.finding.source.id = 8;
    v.finding.source.visible_parent = Some(0);
    v.method = MethodId::new("instrument-b");
    let shared = CountUpdater.ingest(&v).unwrap();
    assert!(shared.duplicate);
    assert_eq!(shared.after.summaries[&v.method].observed(), 0);
    v.finding.source.visible_parent = None;
    let hidden = CountUpdater.ingest(&v).unwrap();
    assert!(!hidden.duplicate);
    assert_eq!(hidden.after.summaries[&v.method].observed(), 1);
}
#[test]
fn direct_record_evictions_preserve_only_compact_consequences() {
    let mut m = MethodMemory::neutral(5);
    let mut evictions = 0;
    for episode in 0..5 {
        let first = acquired(m, episode, Some(2));
        evictions += first.evicted.len();
        m = first.after;
        let mut v = first.input;
        v.memory = m;
        v.finding.acquisition.local = 1;
        v.finding.source.id = 8;
        let second = CountUpdater.ingest(&v).unwrap();
        evictions += second.evicted.len();
        m = verified(second.after, episode, episode, 2).after;
    }
    assert_eq!(evictions, 2);
    assert_eq!(m.records.len(), 8);
    assert_eq!(counts(&m).success, 10);
    let old = counts(&m);
    let late = verified(m, 0, 4, 3);
    assert_eq!(late.transport_effect.acceptance, e7::Acceptance::Pending);
    assert_eq!(counts(&late.after), old);
    assert!(
        !late
            .after
            .records
            .iter()
            .any(|r| r.finding.acquisition.episode == 0)
    );
}
#[test]
fn indirect_reports_and_other_observers_never_supply_personal_samples() {
    let m = acquired(MethodMemory::neutral(5), 0, Some(2)).after;
    let other = MethodMemory::neutral(4);
    let r = IndirectExperience {
        episode: 0,
        message: 2,
        method: Some(MethodId::new("instrument-a")),
        finding: m.records[0].finding.clone(),
    };
    let r = ingest_indirect(&other, r).unwrap();
    assert!(r.after.summaries.is_empty());
    assert_eq!(counts(&m).graded(), 0);
    assert!(
        CountUpdater
            .ingest(&MethodIngestView {
                memory: other,
                method: MethodId::new("instrument-a"),
                finding: m.records[0].finding.clone(),
                origin: e9::DeliveryOrigin::Selected(0)
            })
            .is_err()
    );
}
#[test]
fn bounded_ids_exact_arithmetic_and_json_memory_replay() {
    for key in [
        "",
        "bad id",
        "é",
        "/",
        "abcdefghijklmnopqrstuvwxyz0123456789",
    ] {
        assert!(MethodId::new(key).validate().is_err());
    }
    let c = Counts {
        success: u16::MAX,
        failure: u16::MAX,
        nonmissing: u16::MAX,
        missing: u16::MAX,
    };
    assert_eq!(
        c.accuracy(1),
        e7::Probability {
            numerator: 65536,
            denominator: 131072
        }
    );
    let m = verified(
        acquired(MethodMemory::neutral(5), 0, Some(2)).after,
        0,
        0,
        2,
    )
    .after;
    let bytes = serde_json::to_vec(&m).unwrap();
    let restored: MethodMemory = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, m);
    assert_eq!(
        e7::canonical_bytes(&m).unwrap(),
        e7::canonical_bytes(&restored).unwrap()
    );
}
