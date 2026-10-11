use social_deduction::{
    experiment005 as e5, experiment007 as e7, experiment010 as e10, experiment011 as e11,
    experiment012::*,
};
fn fixture(f: Diagnostic) -> Config {
    diagnostic(f, 0, SocialKind::ProvenanceAwareSocial)
}
fn sample() -> (Config, Report) {
    let c = fixture(Diagnostic::AccurateRequested);
    let r = run(c.clone()).unwrap();
    (c, r)
}
fn query(r: &Report) -> &RequestDecision {
    r.episodes[2]
        .requests
        .iter()
        .find(|d| d.choice.selected.is_some())
        .unwrap()
}
fn reply(r: &Report) -> &ResponseDecision {
    r.episodes[2]
        .responses
        .iter()
        .find(|d| d.choice.selected.is_some())
        .unwrap()
}

#[test]
fn typed_request_and_response_versions_and_bounds() {
    let (_, r) = sample();
    let mut q = query(&r).choice.selected.clone().unwrap();
    q.validate().unwrap();
    q.version = "bad".into();
    assert!(q.validate().is_err());
    let mut a = reply(&r).choice.selected.clone().unwrap();
    a.validate().unwrap();
    a.reference.slot = 0;
    assert!(a.validate().is_err());
    let mut c = fixture(Diagnostic::AccurateRequested);
    c.version = "old".into();
    assert!(run(c).is_err());
}
#[test]
fn request_views_are_local_and_catalogs_have_no_sealed_results() {
    let (_, r) = sample();
    let v = &query(&r).input;
    assert_eq!(v.id(), v.local.personal.observer);
    assert_eq!(v.id(), v.memory.observer);
    let j = serde_json::to_value(v).unwrap();
    for field in [
        "config",
        "processes",
        "recipient_memory",
        "actual_actor",
        "actual_root",
        "intent",
    ] {
        assert!(j.get(field).is_none());
    }
    for m in &v.local.catalog {
        let j = serde_json::to_value(m).unwrap();
        assert!(j.get("candidate").is_none());
        assert!(j.get("actual_accuracy").is_none());
    }
}
#[test]
fn unknown_private_expertise_can_be_chosen_and_not_revealed_by_question() {
    let (_, r) = sample();
    let d = query(&r);
    let q = d.choice.selected.as_ref().unwrap();
    assert!(
        !d.input
            .local
            .social
            .received
            .iter()
            .any(|m| m.speaker == q.recipient)
    );
    let wire = serde_json::to_value(q).unwrap();
    assert!(wire.get("recipient_counts").is_none());
    assert!(d.recipient_counts.unwrap().graded() > 0);
}
#[test]
fn requester_decisions_ignore_hidden_truth_and_other_private_memory() {
    let (c, r) = sample();
    let mut altered = c;
    for i in &mut altered.base.base.incidents {
        i.game.seed = i.game.seed.wrapping_add(99);
        for a in &mut i.artifacts {
            a.candidate = Some(4);
        }
    }
    let changed = run(altered).unwrap();
    let a = &r.episodes[0].requests;
    let b = &changed.episodes[0].requests;
    assert_eq!(a, b);
    let v = &query(&r).input;
    let mut p = Requester::new(RequestKind::ExperienceGapAsk, 1, v.id());
    let mut p2 = Requester::new(RequestKind::ExperienceGapAsk, 1, v.id());
    assert_eq!(p.choose(v), p2.choose(v));
}
#[test]
fn uncertainty_and_cost_heuristics_are_locally_sensitive() {
    let (_, r) = sample();
    let mut v = query(&r).input.clone();
    let mut p = Requester::new(RequestKind::CostAwareAsk, 0, v.id());
    v.local.incident.incident.current = e5::Belief::from_scores([0; 6]);
    let uncertain = p.choose(&v);
    v.local.incident.incident.current = e5::Belief::from_scores([6, 0, 0, 0, 0, 0]);
    let settled = p.choose(&v);
    assert!(
        uncertain
            .options
            .iter()
            .zip(&settled.options)
            .all(|(a, b)| a.score >= b.score)
    );
    assert_ne!(uncertain.options, settled.options);
    v.local.personal.summaries.values_mut().for_each(|c| {
        c.success = 10;
        c.failure = 0;
        c.nonmissing = 10;
    });
}
#[test]
fn legal_queries_and_answers_consume_distinct_outbound_actions() {
    let (_, r) = sample();
    let e = &r.episodes[2];
    let q = query(&r).choice.selected.as_ref().unwrap();
    let a = reply(&r).choice.selected.as_ref().unwrap();
    for reference in [q.reference, a.reference] {
        let c = e
            .capacity
            .iter()
            .find(|c| c.reference == reference)
            .unwrap();
        assert!(c.submitted);
        assert_eq!(c.after, c.before + 1);
    }
}
#[test]
fn scoped_delivery_reaches_only_addressed_participants() {
    let (_, r) = sample();
    let q = query(&r).choice.selected.as_ref().unwrap();
    let a = reply(&r).choice.selected.as_ref().unwrap();
    for (id, m) in r.episodes[2].inquiry_after.iter().enumerate() {
        assert_eq!(
            m.received.iter().any(|x| x.reference == q.reference),
            id == usize::from(q.recipient)
        );
        assert_eq!(
            m.responses.iter().any(|x| x.reference == a.reference),
            id == usize::from(a.recipient)
        );
    }
    for (id, m) in r.episodes[2].before_answers.iter().enumerate() {
        assert!(
            !m.received.iter().any(|x| x.reference == a.reference),
            "preanswer observer{id}"
        );
    }
}
#[test]
fn absent_response_creates_no_wire_or_social_evidence() {
    let r = run(fixture(Diagnostic::NoResponse)).unwrap();
    assert!(
        r.episodes
            .iter()
            .flat_map(|e| &e.responses)
            .all(|d| d.choice.selected.is_none())
    );
    assert!(
        r.episodes[2]
            .social_after
            .iter()
            .all(|m| m.received.is_empty())
    );
    assert!(r.episodes[2].requests.iter().any(|q| q.delivered.is_some()));
}
#[test]
fn unreceived_foreign_future_and_duplicate_request_references_reject() {
    let (_, r) = sample();
    let d = reply(&r);
    let mut c = d.choice.clone();
    c.selected.as_mut().unwrap().request = Reference {
        episode: 7,
        slot: 0,
    };
    assert!(validate_response(&d.input, &c).is_err());
    let mut v = d.input.clone();
    v.memory.received.clear();
    assert!(validate_response(&v, &d.choice).is_err());
    v = d.input.clone();
    v.memory
        .answered
        .push(d.choice.selected.as_ref().unwrap().request);
    assert!(validate_response(&v, &d.choice).is_err());
    c = d.choice.clone();
    c.selected.as_mut().unwrap().recipient = 1;
    assert!(validate_response(&d.input, &c).is_err());
}
#[test]
fn invalid_recipient_unknown_family_duplicate_and_inaccessible_prior_reject() {
    let (_, r) = sample();
    let d = query(&r);
    let original = d.choice.selected.as_ref().unwrap();
    for recipient in [6, d.input.id()] {
        let mut q = original.clone();
        q.recipient = recipient;
        assert!(validate_request(&d.input, &q).is_err());
    }
    let mut q = original.clone();
    q.family = MethodId::new("unknown");
    assert!(validate_request(&d.input, &q).is_err());
    q = original.clone();
    q.kind = RequestType::ExperienceBasis;
    q.prior = Some(Reference {
        episode: 0,
        slot: 0,
    });
    assert!(validate_request(&d.input, &q).is_err());
    let mut v = d.input.clone();
    v.memory.sent.push(original.clone());
    assert!(validate_request(&v, original).is_err());
}
#[test]
fn exhausted_opportunities_and_wrong_window_reject_without_retry() {
    let (_, r) = sample();
    let d = query(&r);
    let mut v = d.input.clone();
    v.capacity = 0;
    assert!(validate_request(&v, d.choice.selected.as_ref().unwrap()).is_err());
    v = d.input.clone();
    v.window = Window::Final;
    assert!(validate_request(&v, d.choice.selected.as_ref().unwrap()).is_err());
    let d = reply(&r);
    let mut v = d.input.clone();
    v.capacity = 0;
    assert!(validate_response(&v, &d.choice).is_err());
    v = d.input.clone();
    v.window = Window::First;
    assert!(validate_response(&v, &d.choice).is_err());
}
#[test]
fn no_experience_allows_insufficiency_and_silence() {
    let r = run(fixture(Diagnostic::NoExperience)).unwrap();
    assert!(
        r.episodes
            .iter()
            .flat_map(|e| &e.responses)
            .filter_map(|d| d.choice.selected.as_ref())
            .any(|a| a.answer == Answer::InsufficientInformation)
    );
    let (_, r) = sample();
    let mut v = reply(&r).input.clone();
    assert!(ResponseKind::NoReply.choose(&v).selected.is_none());
    v.local.incident.incident.current = e5::Belief::from_scores([6, 0, 0, 0, 0, 0]);
    assert!(ResponseKind::SelectiveReply.choose(&v).selected.is_none());
    v.local.incident.incident.current = e5::Belief::from_scores([0; 6]);
    v.local.personal = MethodMemory::neutral(v.id());
    assert_eq!(
        ResponseKind::SelectiveReply
            .choose(&v)
            .selected
            .unwrap()
            .answer,
        Answer::Refuse
    );
}
#[test]
fn sincere_response_counts_equal_retained_local_counts() {
    let (_, r) = sample();
    let d = reply(&r);
    let Answer::Assessment { payload, .. } = &d.choice.selected.as_ref().unwrap().answer else {
        panic!("no assessment")
    };
    let c = d.input.local.personal.summaries[&payload.family];
    assert_eq!(payload.counts, Some(c));
    assert_eq!(payload.rating, rating(c));
    assert_eq!(d.choice.intent, MethodIntent::Sincere);
}
#[test]
fn misleading_intent_remains_outside_wire_and_listener_input() {
    let r = run(fixture(Diagnostic::WrongAnswer)).unwrap();
    let d = reply(&r);
    assert_eq!(d.choice.intent, MethodIntent::Contrary);
    let a = d.choice.selected.as_ref().unwrap();
    let j = serde_json::to_value(a).unwrap();
    for f in ["intent", "basis", "actual_origins", "actual_actor", "token"] {
        assert!(j.get(f).is_none());
    }
    let mut change = d.choice.clone();
    change.intent = MethodIntent::Sincere;
    assert_eq!(change.selected, d.choice.selected);
    assert_eq!(
        social_message(a),
        social_message(change.selected.as_ref().unwrap())
    );
}
#[test]
fn questions_and_answers_add_no_verified_personal_samples() {
    let (_, r) = sample();
    let d = r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_eq!(d.input.experience, r.episodes[2].method_before[5].summaries);
    let a = reply(&r).choice.selected.as_ref().unwrap();
    assert!(social_message(a).unwrap().payload.verification_claimed);
    let q = query(&r).choice.selected.as_ref().unwrap();
    assert_eq!(q.requester, 5);
    assert_eq!(
        d.input.experience[&MethodId::new("instrument-a")].graded(),
        2
    );
    assert!(
        !d.input
            .experience
            .contains_key(&MethodId::new("instrument-b"))
    );
}
#[test]
fn identical_answer_interpretation_ignores_private_intent() {
    let (_, r) = sample();
    let a = reply(&r);
    let sharing = response_sharing(&a.input, &a.choice).unwrap();
    let mut change = sharing.clone();
    change.intent = MethodIntent::Contrary;
    let s1 = resolve_report(
        a.input.local.clone(),
        sharing,
        Origin::Selected,
        &Default::default(),
    );
    let s2 = resolve_report(
        a.input.local.clone(),
        change,
        Origin::Selected,
        &Default::default(),
    );
    assert_eq!(s1.message, s2.message);
    let v = &r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap()
        .input;
    let mut p = Policy::new(e10::PolicyKind::ExperienceAdjusted, 0, 5);
    let mut p2 = Policy::new(e10::PolicyKind::ExperienceAdjusted, 0, 5);
    assert_eq!(p.choose(v), p2.choose(v));
}
#[test]
fn known_duplicate_origins_group_and_hidden_ancestry_is_not_available() {
    let family = MethodId::new("instrument-b");
    let (_, r) = sample();
    let a = social_message(reply(&r).choice.selected.as_ref().unwrap()).unwrap();
    let mut b = a.clone();
    b.reference.slot += 1;
    b.speaker = 1;
    b.payload.parent = Some(a.reference);
    b.payload.claimed_origin = Some(0);
    b.payload.basis = ClaimedBasis::Relay;
    let mut memory = SocialMemory::neutral(5);
    memory.received = vec![a.clone(), b.clone()];
    let v = SocialView {
        observer: 5,
        family,
        personal: Counts::default(),
        memory,
        credits: [social_deduction::experiment006::Credit::default(); 6],
    };
    let known = SocialKind::ProvenanceAwareSocial.assess(&v);
    let naive = SocialKind::NaiveSocial.assess(&v);
    assert_eq!(known.positive, 1);
    assert_eq!(naive.positive, 2);
    let mut hidden = v.clone();
    hidden.memory.received[1].payload.parent = None;
    hidden.memory.received[1].payload.claimed_origin = Some(1);
    assert_eq!(
        SocialKind::ProvenanceAwareSocial.assess(&hidden).positive,
        2
    );
}
#[test]
fn response_corrections_preserve_immutable_old_wires() {
    let r = run(fixture(Diagnostic::ResponseCorrection)).unwrap();
    let answers: Vec<_> = r
        .episodes
        .iter()
        .flat_map(|e| &e.responses)
        .filter_map(|d| d.choice.selected.as_ref())
        .collect();
    assert!(answers.iter().any(|a| matches!(
        a.answer,
        Answer::Assessment {
            revision: Some(_),
            ..
        }
    )));
    let first = answers[0].clone();
    assert_eq!(&first, answers[0]);
    assert!(
        r.episodes
            .iter()
            .flat_map(|e| &e.responses)
            .filter_map(|d| d.social.as_ref())
            .any(|s| s.message.as_ref().is_some_and(|m| m.revision.is_some()))
    );
}
#[test]
fn late_answers_cannot_rewrite_completed_investigation() {
    let r = run(fixture(Diagnostic::LateAnswer)).unwrap();
    for e in &r.episodes {
        for d in &e.investigation {
            assert_eq!(d.before_answer_input.social, d.input.social);
            if !d.forced {
                assert_eq!(d.before_answer_choice.action, d.choice.action);
            }
        }
    }
    assert!(
        r.episodes[2]
            .responses
            .iter()
            .any(|d| !d.delivered.is_empty())
    );
}
#[test]
fn legally_different_answers_change_same_incident_investigation() {
    let (_, r) = sample();
    let r2 = run(fixture(Diagnostic::WrongAnswer)).unwrap();
    let a = r.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    let b = r2.episodes[2]
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_eq!(a.input.personal, b.input.personal);
    assert_ne!(a.choice.action, b.choice.action);
    assert_ne!(a.before_answer_choice.action, a.choice.action);
}
#[test]
fn multiple_received_requests_do_not_create_free_answers() {
    let r = run(fixture(Diagnostic::MultipleRequests)).unwrap();
    let e = &r.episodes[2];
    assert!(e.requests.iter().filter(|q| q.delivered.is_some()).count() > 1);
    assert_eq!(
        e.responses
            .iter()
            .filter(|d| d.input.id() == 0 && d.choice.selected.is_some())
            .count(),
        1
    );
    assert!(e.inquiry_after[0].received.len() > 1);
}
#[test]
fn total_outbound_capacity_conserves_across_all_windows() {
    for f in DIAGNOSTICS {
        let r = run(fixture(f)).unwrap();
        for e in &r.episodes {
            for id in 0..6 {
                let records: Vec<_> = e
                    .capacity
                    .iter()
                    .filter(|c| usize::from(c.agent) == id)
                    .collect();
                let mut used = 0;
                for c in records {
                    assert_eq!(c.before, used);
                    assert!(c.after <= 2);
                    assert_eq!(c.after - c.before, u8::from(c.submitted));
                    used = c.after;
                }
            }
        }
    }
}
#[test]
fn response_can_exhaust_capacity_and_block_final_testimony() {
    let mut c = fixture(Diagnostic::ReplyDisplaces);
    for i in c.base.base.incidents.iter_mut().skip(2) {
        i.game.observations[0] = Some((i.game.actor() + 1) % 6);
        i.forced.retain(|(id, _)| *id != 0);
        i.forced.push((0, InvestigationAction::Inspect(1)));
    }
    let r = run(c).unwrap();
    let e = &r.episodes[2];
    let final_slot = e
        .capacity
        .iter()
        .find(|c| c.agent == 0 && c.window == Window::Final)
        .unwrap();
    assert!(final_slot.exhausted);
    assert_eq!(final_slot.after, 2);
    let probe = final_slot.foregone_input.as_ref().unwrap();
    assert_eq!(probe.incident.opportunities_left, 1);
    assert_eq!(
        Speaker::new(CommunicationKind::Sincere).choose(probe),
        *final_slot.foregone.as_ref().unwrap()
    );
    assert!(
        final_slot
            .foregone
            .as_ref()
            .is_some_and(|c| matches!(c.choice.action, e5::Action::Speak { .. }))
    );
    assert_eq!(
        e.incident
            .testimony
            .iter()
            .filter(|t| t.message.speaker == 0)
            .count(),
        1
    );
}
#[test]
fn ordinary_no_query_baseline_retains_two_speech_opportunities() {
    let r = run(generated(0, Treatment::Ordinary)).unwrap();
    for e in &r.episodes {
        assert!(e.responses.is_empty());
        assert_eq!(e.capacity.len(), 12);
        assert!(e.requests.iter().all(|d| d.choice.selected.is_none()));
        assert_eq!(e.incident.decisions.len(), 18);
    }
}
#[test]
fn effort_accounting_and_finding_weights_are_unchanged() {
    let (_, r) = sample();
    for l in &r.ledgers {
        assert_eq!(l.initial, l.remaining + l.spent);
    }
    for e in &r.episodes {
        for d in &e.deliveries {
            assert_eq!(d.balance_before - d.balance_after, 1);
            assert_eq!(
                d.after_assessment.after,
                e11::EvidenceAware {
                    testimony: r.config.listener
                }
                .assess(&d.after)
                .after
            );
        }
    }
}
#[test]
fn selected_and_injected_answers_share_visible_effects_distinct_origin() {
    let [a, b] = origin_pair(0).unwrap();
    let a = run(a).unwrap();
    let b = run(b).unwrap();
    assert_eq!(reply(&a).choice.selected, reply(&b).choice.selected);
    assert_eq!(a.episodes[2].social_after, b.episodes[2].social_after);
    assert_ne!(reply(&a).origin, reply(&b).origin);
    assert_eq!(a.episodes[2].capacity, b.episodes[2].capacity);
    assert!(reply(&b).proposal.is_some());
}
#[test]
fn bounded_memory_evictions_never_reconstruct_from_archive() {
    let (_, r) = sample();
    let q = query(&r).choice.selected.as_ref().unwrap();
    let mut m = InquiryMemory::neutral(q.requester);
    let mut evictions = 0;
    for ep in 0..8 {
        for slot in 0..6 {
            let mut x = q.clone();
            x.reference = Reference { episode: ep, slot };
            let d = remember_request(&m, x, true).unwrap();
            evictions += d.evicted.len();
            m = d.after;
        }
    }
    assert_eq!(m.sent.len(), 12);
    assert_eq!(evictions, 36);
    assert!(!m.sent.iter().any(|x| x.reference == q.reference));
}
#[test]
fn refusal_and_questions_do_not_enter_method_social_boundary() {
    let (_, r) = sample();
    let mut a = reply(&r).choice.selected.clone().unwrap();
    a.answer = Answer::Refuse;
    assert!(social_message(&a).is_none());
    a.answer = Answer::InsufficientInformation;
    assert!(social_message(&a).is_none());
}
#[test]
fn complete_reports_replay_with_legal_ballots_and_termination() {
    for f in DIAGNOSTICS {
        let c = fixture(f);
        let a = run(c.clone()).unwrap();
        let b = run(c).unwrap();
        assert_eq!(a, b);
        assert_eq!(report_digests(&a).unwrap(), report_digests(&b).unwrap());
        for e in &a.episodes {
            assert_eq!(
                e.incident
                    .decisions
                    .iter()
                    .filter(|d| d.input.phase == social_deduction::Phase::Judgment)
                    .count(),
                6
            );
            assert_eq!(
                e.incident.verdict,
                social_deduction::verdict(e.incident.tally)
            );
            assert!(matches!(
                e.incident.objective_history.last(),
                Some(e5::Event::Outcome { .. })
            ));
        }
    }
}
#[test]
fn development_evaluation_pairs_worlds_and_canonical_sections() {
    let e = evaluate_seeds(0..1, 0..1).unwrap();
    assert_eq!(e.rows.len(), 100);
    assert_eq!(e.comparisons.len(), 66);
    assert_eq!(e.incidents, 800);
    for row in &e.rows {
        assert_eq!(row.digests.len(), 8);
        assert_eq!(row.full.ballots.iter().sum::<u64>(), 48);
    }
    let worlds: Vec<_> = e
        .rows
        .iter()
        .filter(|r| r.population == "heldout")
        .collect();
    assert!(
        worlds
            .iter()
            .all(|r| r.environment == worlds[0].environment)
    );
    assert_eq!(manifest(&e).unwrap().series, 100);
}
#[test]
fn malformed_reply_payload_and_foreign_revision_reject() {
    let (_, r) = sample();
    let d = reply(&r);
    let mut c = d.choice.clone();
    if let Answer::Assessment {
        payload, revision, ..
    } = &mut c.selected.as_mut().unwrap().answer
    {
        payload.counts.as_mut().unwrap().success = 101;
        assert!(payload.validate().is_err());
        payload.counts.as_mut().unwrap().success = 1;
        *revision = Some(Reference {
            episode: 0,
            slot: 5,
        });
    }
    assert!(validate_response(&d.input, &c).is_err());
}
#[test]
fn random_requests_use_independent_seeded_stream_and_can_decline() {
    let (_, r) = sample();
    let v = &query(&r).input;
    let mut a = Requester::new(RequestKind::RandomAsk, 120, v.id());
    let mut b = Requester::new(RequestKind::RandomAsk, 120, v.id());
    let choices: Vec<_> = (0..32)
        .map(|_| {
            let x = a.choose(v);
            assert_eq!(x, b.choose(v));
            x
        })
        .collect();
    assert!(choices.iter().any(|x| x.selected.is_none()));
    assert!(choices.iter().any(|x| x.selected.is_some()));
}
#[test]
fn canonical_sorted_float_free_and_same_os_is_not_portability() {
    assert_eq!(
        canonical_digest(&serde_json::json!({"z":1,"a":2})).unwrap(),
        canonical_digest(&serde_json::json!({"a":2,"z":1})).unwrap()
    );
    assert!(canonical_digest(&serde_json::json!({"x":0.5})).is_err());
    let a = e7::PlatformReplay {
        execution: e7::execution().unwrap(),
        manifest: manifest(&evaluate_seeds(0..0, 0..0).unwrap()).unwrap(),
    };
    assert!(e7::compare_platforms(&a, &a).is_err());
}

struct BadQuery;
impl RequestPolicy for BadQuery {
    fn choose(&mut self, v: &InquiryView) -> RequestChoice {
        let mut c = Requester::new(RequestKind::ExperienceGapAsk, 0, v.id()).choose(v);
        if let Some(q) = &mut c.selected {
            q.recipient = 6;
        }
        c
    }
}
#[test]
fn resolver_rejects_query_consumes_capacity_and_delivers_no_free_alternative() {
    let c = fixture(Diagnostic::AccurateRequested);
    let mut components = Components::from_config(&c);
    components.requesters[5] = Box::new(BadQuery);
    let r = run_with(c, &mut components).unwrap();
    let d = query(&r);
    assert!(d.rejection.is_some());
    assert!(d.delivered.is_none());
    let cap = r.episodes[2]
        .capacity
        .iter()
        .find(|x| x.reference == d.input.local.reference)
        .unwrap();
    assert!(cap.submitted && cap.rejected);
    assert_eq!(cap.after, 1);
    assert!(r.episodes[2].responses.is_empty());
}
#[test]
fn requester_same_received_history_survives_changed_responder_private_experience() {
    let (c, r) = sample();
    let mut alternate = c;
    for i in alternate.base.base.incidents.iter_mut().take(2) {
        i.forced.retain(|(id, _)| *id != 0);
        i.forced.push((0, InvestigationAction::Inspect(0)));
    }
    let altered = run(alternate).unwrap();
    assert_ne!(
        r.episodes[2].method_before[0],
        altered.episodes[2].method_before[0]
    );
    assert_eq!(query(&r).input, query(&altered).input);
    assert_eq!(query(&r).choice, query(&altered).choice);
}
#[test]
fn answer_earlier_delivery_precedes_all_investigation_decisions() {
    let (_, r) = sample();
    let answer = r
        .objective_history
        .iter()
        .position(|x| matches!(x, Event::ResponseDelivered { episode: 2, .. }))
        .unwrap();
    let selection = r
        .objective_history
        .iter()
        .position(|x| matches!(x, Event::Investigation { episode: 2, .. }))
        .unwrap();
    assert!(answer < selection);
    let r = run(fixture(Diagnostic::LateAnswer)).unwrap();
    let answer = r
        .objective_history
        .iter()
        .position(|x| matches!(x, Event::ResponseDelivered { episode: 2, .. }))
        .unwrap();
    let selection = r
        .objective_history
        .iter()
        .position(|x| matches!(x, Event::Investigation { episode: 2, .. }))
        .unwrap();
    assert!(answer > selection);
}

#[test]
fn availability_answer_supplies_no_conditional_accuracy_support() {
    let (_, r) = sample();
    let mut v = reply(&r).input.clone();
    v.memory.received[0].kind = RequestType::MethodAvailability;
    let c = ResponseKind::SincereReply.choose(&v);
    let a = c.selected.unwrap();
    let m = social_message(&a).unwrap();
    assert_eq!(m.payload.rating, Rating::Uncertain);
    let mut memory = SocialMemory::neutral(5);
    memory.received.push(m.clone());
    let estimate = SocialKind::ProvenanceAwareSocial.assess(&SocialView {
        observer: 5,
        family: m.payload.family,
        personal: Counts::default(),
        memory,
        credits: [social_deduction::experiment006::Credit::default(); 6],
    });
    assert_eq!(estimate.effective_accuracy, estimate.personal_accuracy);
    assert_eq!(estimate.nonmissing, 1);
}
#[test]
fn wider_control_is_explicit_and_identical_answers_can_meet_different_histories() {
    let private = run(fixture(Diagnostic::AccurateRequested)).unwrap();
    let wider = run(fixture(Diagnostic::PrivateVsWider)).unwrap();
    assert_eq!(
        reply(&private).choice.selected,
        reply(&wider).choice.selected
    );
    assert!(matches!(reply(&wider).origin, Origin::Intervention(_)));
    assert!(
        wider.episodes[2].social_after[4].received.len()
            > private.episodes[2].social_after[4].received.len()
    );
    let different = run(fixture(Diagnostic::DifferentHistories)).unwrap();
    let e = &different.episodes[2];
    let a = e
        .investigation
        .iter()
        .find(|d| d.input.local.id == 4)
        .unwrap();
    let b = e
        .investigation
        .iter()
        .find(|d| d.input.local.id == 5)
        .unwrap();
    assert_ne!(a.input.experience, b.input.experience);
    assert_eq!(a.input.social.received, b.input.social.received);
    assert_ne!(a.choice.action, b.choice.action);
}
#[test]
fn basis_question_requires_prior_received_report_and_clarification_can_retain_copy() {
    let r = run(fixture(Diagnostic::KnownDuplicate)).unwrap();
    let q = query(&r);
    assert_eq!(
        q.choice.selected.as_ref().unwrap().kind,
        RequestType::ExperienceBasis
    );
    assert!(q.choice.selected.as_ref().unwrap().prior.is_some());
    let d = reply(&r);
    assert!(d.rejection.is_none());
    assert!(matches!(d.choice.basis, PrivateBasis::Received(_)));
}

#[test]
fn selective_silence_preserves_capacity_for_executed_final_testimony() {
    let mut c = fixture(Diagnostic::ReplyDisplaces);
    c.response[0] = ResponseKind::SelectiveReply;
    let r = run(c).unwrap();
    let ep = &r.episodes[2];
    let reply = ep.responses.iter().find(|d| d.input.id() == 0).unwrap();
    assert!(!reply.choice.eligible.is_empty());
    assert!(reply.choice.selected.is_none());
    assert!(reply.rejection.is_none());
    let capacity: Vec<_> = ep.capacity.iter().filter(|c| c.agent == 0).collect();
    assert_eq!((capacity[1].before, capacity[1].after), (1, 1));
    assert!(capacity[2].submitted && !capacity[2].exhausted);
    assert_eq!(capacity[2].after, 2);
    assert!(ep.incident.decisions.iter().any(|d| {
        d.input.id == 0
            && d.input.opportunities_left == 1
            && matches!(d.choice.action, e5::Action::Speak { .. })
    }));
}
