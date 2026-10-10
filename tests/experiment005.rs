use social_deduction::{Phase, experiment004 as e4, experiment005::*};
use std::collections::BTreeSet;
fn unknown(messages: Vec<Message>) -> ListenerView {
    ListenerView {
        id: 5,
        original: IndividualState {
            observation: None,
            briefing: None,
            belief: None,
            incentives: Incentives::ACCURACY,
        },
        observations: vec![],
        prior: Belief::default(),
        received: messages,
    }
}
fn assertion(id: usize, speaker: u8, c: u8) -> Message {
    Message {
        id,
        speaker,
        body: Body::Assert(c),
        attribution: Attribution::Firsthand,
    }
}
fn revision(id: usize, speaker: u8, target: usize, c: Option<u8>) -> Message {
    Message {
        id,
        speaker,
        body: Body::Revise {
            target,
            replacement: c,
            forwarded: None,
        },
        attribution: Attribution::Unspecified,
    }
}
fn fixture(f: Fixture, l: ListenerKind) -> Report {
    run(fixture_config(f, 0, l)).unwrap()
}

#[test]
fn explicit_revisions_change_standing_without_erasing_history_or_extra_support() {
    let messages = vec![assertion(0, 0, 1), revision(1, 0, 0, Some(2))];
    let v = unknown(messages.clone());
    let a = ListenerKind::Revision.assess(&v);
    assert_eq!(v.received, messages);
    assert_eq!(a.after.scores[1], 0);
    assert_eq!(a.after.scores[2], 1);
    assert_eq!(a.groups.len(), 1);
    assert_eq!(a.interpretations[0].standing, Standing::Superseded);
    assert_eq!(a.interpretations[1].standing, Standing::Active);
    assert!(a.interpretations.iter().all(|a| !a.origin_verified));
    let frozen = ListenerKind::Frozen004.assess(&v);
    assert_eq!(frozen.after.candidate, Some(1));
    assert_eq!(
        frozen.interpretations[1].standing,
        Standing::IgnoredFrozenRevision
    );
    let old = e4::Listener::assess(
        &mut e4::ListenerKind::Provenance,
        &e4::ListenerView {
            id: 5,
            original: v.original.clone(),
            prior: v.prior.clone(),
            messages: original_messages(&v.received),
        },
    );
    assert_eq!(frozen.after, old.after);
    let retracted =
        ListenerKind::Revision.assess(&unknown(vec![assertion(0, 0, 1), revision(1, 0, 0, None)]));
    assert_eq!(retracted.after.candidate, None);
    assert_eq!(retracted.interpretations[1].standing, Standing::Retracted);
}
#[test]
fn branches_dispute_sequential_links_revise_and_other_speakers_cannot_retract() {
    let mut messages = vec![
        assertion(0, 0, 1),
        revision(1, 0, 0, Some(2)),
        revision(2, 0, 0, Some(3)),
    ];
    let a = ListenerKind::Revision.assess(&unknown(messages.clone()));
    assert_eq!(a.after.candidate, None);
    assert_eq!(a.interpretations[1].standing, Standing::Disputed);
    assert!(a.groups[0].contested);
    messages[2] = revision(2, 0, 1, Some(3));
    assert_eq!(
        ListenerKind::Revision
            .assess(&unknown(messages))
            .after
            .candidate,
        Some(3)
    );
    let foreign = ListenerKind::Revision.assess(&unknown(vec![
        assertion(0, 0, 1),
        revision(1, 1, 0, Some(2)),
    ]));
    assert_eq!(foreign.after.candidate, Some(1));
    assert_eq!(
        foreign.interpretations[1].standing,
        Standing::InvalidRevision
    );
    let repeat =
        ListenerKind::Revision.assess(&unknown(vec![assertion(0, 0, 1), assertion(1, 0, 2)]));
    assert_eq!(repeat.after.candidate, Some(1));
    assert_eq!(repeat.interpretations[1].standing, Standing::IgnoredRepeat);
    let same = ListenerKind::Revision.assess(&unknown(vec![
        assertion(0, 0, 1),
        revision(1, 0, 0, Some(2)),
        revision(2, 0, 0, Some(2)),
    ]));
    assert_eq!(same.after.scores[2], 1);
}
#[test]
fn missing_invalid_and_unverified_references_never_fetch_the_archive() {
    let missing = ListenerKind::Revision.assess(&unknown(vec![revision(5, 0, 0, Some(2))]));
    assert_eq!(missing.after.candidate, None);
    assert_eq!(
        missing.interpretations[0].standing,
        Standing::MissingOriginal
    );
    assert!(missing.interpretations[0].checked.is_empty());
    for bad in [revision(0, 0, 0, Some(2)), revision(0, 0, 1, Some(2))] {
        assert_eq!(
            ListenerKind::Revision
                .assess(&unknown(vec![bad]))
                .interpretations[0]
                .standing,
            Standing::InvalidRevision
        );
    }
    let mut m = assertion(0, 0, 1);
    m.attribution = Attribution::NamedSource(2);
    let mut r = revision(1, 0, 0, Some(2));
    r.attribution = Attribution::NamedSource(3);
    let a = ListenerKind::Revision.assess(&unknown(vec![m, r]));
    assert_eq!(a.groups[0].group, Group::Speaker(2));
    assert_eq!(a.groups.len(), 1);
    assert!(a.interpretations.iter().all(|i| !i.origin_verified));
    let malformed = Message {
        attribution: Attribution::Reference(0),
        ..assertion(0, 0, 1)
    };
    let a = ListenerKind::Revision.assess(&unknown(vec![malformed, revision(1, 0, 0, Some(2))]));
    assert_eq!(a.after.candidate, None);
}
#[test]
fn own_relay_revision_cannot_update_origin_or_stale_peers() {
    let mut relay = assertion(1, 1, 1);
    relay.attribution = Attribution::Reference(0);
    let a = ListenerKind::Revision.assess(&unknown(vec![
        assertion(0, 0, 1),
        relay.clone(),
        revision(2, 0, 0, Some(2)),
    ]));
    assert_eq!(a.after.candidate, None);
    assert_eq!(a.groups.len(), 1);
    assert!(a.groups[0].contested);
    let a = ListenerKind::Revision.assess(&unknown(vec![
        assertion(0, 0, 1),
        relay.clone(),
        revision(2, 0, 0, None),
    ]));
    assert_eq!(a.after.candidate, Some(1));
    let mut forwarded = revision(3, 1, 1, Some(2));
    if let Body::Revise { forwarded, .. } = &mut forwarded.body {
        *forwarded = Some(2);
    }
    let a = ListenerKind::Revision.assess(&unknown(vec![
        assertion(0, 0, 1),
        relay,
        revision(2, 0, 0, Some(2)),
        forwarded,
    ]));
    assert_eq!(a.after.scores[2], 1);
    assert_eq!(a.groups.len(), 1);
}
#[test]
fn ownership_receipt_copy_scope_phase_and_budget_are_enforced() {
    let r = fixture(Fixture::Sincere, ListenerKind::Revision);
    let v = &r.decisions[6].input;
    let mut a = Action::Speak {
        body: Body::Revise {
            target: 0,
            replacement: Some(r.config.actor()),
            forwarded: None,
        },
        attribution: Attribution::Unspecified,
        basis: Basis::Private,
        recipients: vec![5],
    };
    assert!(validate_action(v, &a).is_ok());
    let mut alien = v.clone();
    alien.id = 1;
    assert!(validate_action(&alien, &a).is_err());
    alien = v.clone();
    alien.sent.clear();
    assert!(validate_action(&alien, &a).is_err());
    alien = v.clone();
    alien.opportunities_left = 0;
    assert!(validate_action(&alien, &a).is_err());
    alien = v.clone();
    alien.phase = Phase::Judgment;
    assert!(validate_action(&alien, &a).is_err());
    alien = v.clone();
    alien.permitted_recipients = vec![0];
    assert!(validate_action(&alien, &a).is_err());
    if let Action::Speak { basis, body, .. } = &mut a {
        *basis = Basis::ReceivedRevision(99);
        if let Body::Revise { forwarded, .. } = body {
            *forwarded = Some(99)
        }
    }
    assert!(validate_action(v, &a).is_err());
    let relay = fixture(Fixture::RelayBoth, ListenerKind::Revision);
    let d = &relay.decisions[7];
    assert_eq!(d.classification, Classification::FaithfulCorrection);
    assert!(validate_action(&d.input, &d.choice.action).is_ok());
    let mut wrong = d.choice.action.clone();
    if let Action::Speak {
        body: Body::Revise { replacement, .. },
        ..
    } = &mut wrong
    {
        *replacement = Some((relay.config.actor() + 2) % 6);
    }
    assert!(validate_action(&d.input, &wrong).is_err());
}
#[test]
fn private_change_withholding_and_selected_costs_are_separate_from_recipient_updates() {
    for seed in 0..16 {
        for l in [ListenerKind::Frozen004, ListenerKind::Revision] {
            let a = run(fixture_config(Fixture::Retain, seed, l)).unwrap();
            let b = run(fixture_config(Fixture::SelectiveHigh, seed, l)).unwrap();
            let low = run(fixture_config(Fixture::SelectiveLow, seed, l)).unwrap();
            assert_eq!(
                a.decisions[6].input.current.candidate,
                Some(a.config.actor())
            );
            assert_eq!(b.decisions[6].choice.action, Action::Silence);
            assert!(b.decisions[6].choice.recognized_change);
            assert_eq!(b.decisions[6].classification, Classification::Withheld);
            assert_eq!(b.decisions[6].choice.revise_score, Some(-1));
            assert_eq!(low.decisions[6].choice.revise_score, Some(1));
            assert!(matches!(
                low.decisions[6].choice.action,
                Action::Speak {
                    body: Body::Revise { .. },
                    ..
                }
            ));
            let au: Vec<_> = a.updates.iter().filter(|u| u.input.id == 5).collect();
            let bu: Vec<_> = b.updates.iter().filter(|u| u.input.id == 5).collect();
            for (x, y) in au.iter().zip(bu) {
                assert_eq!(x.input, y.input);
                assert_eq!(x.assessment, y.assessment);
            }
            assert_eq!(judgment(&a, 5).choice.action, judgment(&b, 5).choice.action);
        }
    }
    let a = fixture(Fixture::Retain, ListenerKind::Revision);
    let mut c = a.config.clone();
    c.later[0].candidate = Some((c.actor() + 2) % 6);
    let b = run(c).unwrap();
    for (x, y) in a
        .updates
        .iter()
        .filter(|u| u.input.id == 5)
        .zip(b.updates.iter().filter(|u| u.input.id == 5))
    {
        assert_eq!(x.input, y.input);
        assert_eq!(x.assessment, y.assessment);
    }
}
#[test]
fn correction_accuracy_intent_retraction_and_false_forwarding_are_separate() {
    let sincere = fixture(Fixture::Sincere, ListenerKind::Revision);
    assert_eq!(
        sincere.testimony[1].classification,
        Classification::SincereRepair
    );
    assert_eq!(sincere.testimony[0].correct, Some(false));
    assert_eq!(
        sincere.testimony[0].classification,
        Classification::SincereAssertion
    );
    assert_eq!(sincere.correct, Some(true));
    let mistaken = fixture(Fixture::WrongSincere, ListenerKind::Revision);
    assert_eq!(
        mistaken.testimony[1].classification,
        Classification::SincereStillWrong
    );
    assert_eq!(mistaken.correct, Some(false));
    let misleading = fixture(Fixture::Misleading, ListenerKind::Revision);
    assert_eq!(
        misleading.testimony[1].classification,
        Classification::IntentionalContradiction
    );
    assert_eq!(misleading.testimony[1].correct, Some(false));
    let retracted = fixture(Fixture::Retraction, ListenerKind::Revision);
    assert_eq!(
        retracted.testimony[1].classification,
        Classification::UncertainRetraction
    );
    assert_eq!(retracted.verdict, None);
    let mut c = fixture_config(Fixture::RelayBoth, 0, ListenerKind::Revision);
    c.later[0].candidate = Some((c.actor() + 2) % 6);
    let false_relay = run(c).unwrap();
    assert_eq!(
        false_relay.testimony.last().unwrap().classification,
        Classification::FaithfulCorrection
    );
    assert_eq!(false_relay.testimony.last().unwrap().correct, Some(false));
}
#[test]
fn relay_delivery_variants_preserve_local_knowledge_and_contested_groups() {
    let both = fixture(Fixture::RelayBoth, ListenerKind::Revision);
    assert_eq!(both.correct, Some(true));
    assert_eq!(
        both.testimony.last().unwrap().classification,
        Classification::FaithfulCorrection
    );
    {
        let r = fixture(Fixture::RelayFinal, ListenerKind::Revision);
        assert_eq!(judgment(&r, 5).input.current.candidate, None);
        assert!(
            r.updates[judgment(&r, 5).update]
                .assessment
                .groups
                .iter()
                .any(|g| g.contested)
        );
    }
    let intermediate = fixture(Fixture::RelayIntermediate, ListenerKind::Revision);
    assert_eq!(intermediate.correct, Some(false));
    assert_eq!(
        intermediate.testimony.last().unwrap().classification,
        Classification::FaithfulCorrection
    );
    assert!(
        intermediate.updates[judgment(&intermediate, 5).update]
            .assessment
            .interpretations
            .iter()
            .any(|i| i.standing == Standing::InvalidRevision)
    );
    let neither = fixture(Fixture::RelayNeither, ListenerKind::Revision);
    assert_eq!(neither.correct, Some(false));
    assert_eq!(neither.testimony.len(), 3);
    let withheld = fixture(Fixture::RelayWithheld, ListenerKind::Revision);
    assert_eq!(
        withheld.decisions[7].classification,
        Classification::Withheld
    );
    assert_eq!(withheld.correct, Some(false));
    assert_eq!(withheld.testimony.len(), 3);
    let missing = fixture(Fixture::MissingOriginal, ListenerKind::Revision);
    assert!(
        judgment(&missing, 5)
            .input
            .received
            .iter()
            .all(|m| m.id != 0)
    );
    assert_eq!(judgment(&missing, 5).input.current.candidate, None);
    assert_eq!(
        missing.updates[judgment(&missing, 5).update]
            .assessment
            .interpretations[0]
            .standing,
        Standing::MissingOriginal
    );
}
#[test]
fn chronological_snapshots_original_observations_and_lineage_are_immutable() {
    for f in [
        Fixture::Sincere,
        Fixture::RelayBoth,
        Fixture::Injected,
        Fixture::Branches,
        Fixture::MissingOriginal,
    ] {
        let r = fixture(f, ListenerKind::Revision);
        let mut received = vec![vec![]; 6];
        let mut sent = vec![vec![]; 6];
        let mut observations = vec![vec![]; 6];
        let mut decided = BTreeSet::new();
        for e in &r.objective_history {
            match e {
                Event::Assigned {
                    agent, observation, ..
                }
                | Event::Observed {
                    agent, observation, ..
                } => observations[usize::from(*agent)].push(observation.clone()),
                Event::Decided { decision } => {
                    decided.insert(*decision);
                }
                Event::Delivered { message, recipient } => {
                    let t = &r.testimony[*message];
                    assert!(t.recipients.contains(recipient));
                    assert!(t.lineage.evidence_parents.iter().all(|p| *p < t.message.id));
                    assert!(
                        t.lineage.revision_of.is_none_or(|p| p < t.message.id
                            && r.testimony[p].message.speaker == t.message.speaker)
                    );
                    if let Origin::Selected { decision } = t.origin {
                        assert!(decided.contains(&decision));
                        let d = &r.decisions[decision];
                        assert_eq!(t.belief, d.input.current.candidate);
                        assert_eq!(
                            d.input.observations,
                            observations[usize::from(t.message.speaker)]
                        );
                        for parent in &t.lineage.evidence_parents {
                            assert!(d.input.received.iter().any(|m| m.id == *parent));
                        }
                    }
                    received[usize::from(*recipient)].push(t.message.clone());
                }
                Event::Updated { update } => {
                    let u = &r.updates[*update];
                    assert_eq!(u.input.received, received[usize::from(u.input.id)]);
                    assert_eq!(u.input.observations, observations[usize::from(u.input.id)]);
                }
                Event::Resolved {
                    decision, message, ..
                } => {
                    let d = &r.decisions[*decision];
                    assert_eq!(d.input.current, r.updates[d.update].assessment.after);
                    if let Some(id) = message {
                        sent[usize::from(d.input.id)].push(r.testimony[*id].message.clone());
                    }
                }
                _ => {}
            }
        }
        assert_eq!(r.decisions.len(), 18);
        assert_eq!(r.updates.len(), 18);
        assert_eq!(
            r.config.observations[0],
            r.decisions[6].input.original.observation
        );
        assert_eq!(r.decisions[0].input.observations.len(), 1);
        assert_eq!(r.decisions[6].input.observations.len(), 2);
        for t in &r.testimony {
            assert!(t.lineage.depth <= 3);
            if let Some(parent) = t.lineage.revision_of {
                assert_eq!(
                    r.testimony[parent].message.body.candidate(),
                    Some((r.config.actor() + 1) % 6)
                );
            }
        }
        if f == Fixture::Injected || f == Fixture::Branches {
            assert_eq!(r.injection_inputs.len(), r.config.injections.len());
            assert!(
                r.testimony
                    .iter()
                    .all(|t| matches!(t.origin, Origin::Injected { .. }))
            );
        }
    }
}
#[test]
fn typed_local_inputs_never_contain_correctness_intent_or_audit_history() {
    let r = fixture(Fixture::RelayBoth, ListenerKind::Revision);
    for u in &r.updates {
        let json = serde_json::to_value(&u.input).unwrap();
        for k in ["actor", "correct", "config", "testimony", "lineage"] {
            assert!(json.get(k).is_none());
        }
        for m in &u.input.received {
            let wire = serde_json::to_value(m).unwrap();
            assert_eq!(wire.as_object().unwrap().len(), 4);
            for k in [
                "roots",
                "basis",
                "belief",
                "intent",
                "correct",
                "recipients",
                "origin",
            ] {
                assert!(wire.get(k).is_none());
            }
        }
    }
    let mut view = unknown(vec![assertion(0, 0, 1)]);
    let before = ListenerKind::Revision.assess(&view);
    view.prior = Belief::from_scores([0, 0, 0, 0, 10, 0]);
    assert_eq!(before, ListenerKind::Revision.assess(&view));
}
struct Illegal;
impl CommunicationPolicy for Illegal {
    fn choose(&mut self, _: &AgentView) -> Choice {
        let mut c = Choice::silence("invalid test action");
        c.action = Action::Speak {
            body: Body::Revise {
                target: 99,
                replacement: Some(99),
                forwarded: None,
            },
            attribution: Attribution::Unspecified,
            basis: Basis::Inference,
            recipients: vec![5],
        };
        c
    }
}
impl JudgmentPolicy for Illegal {
    fn choose(&mut self, _: &AgentView) -> Choice {
        let mut c = Choice::silence("invalid ballot");
        c.action = Action::Vote(Some(99));
        c
    }
}
#[test]
fn invalid_actions_configs_and_assessments_preserve_bounded_termination() {
    let c = Config::new(0);
    let mut policies = Policies::from_config(&c);
    policies.speakers = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn CommunicationPolicy>)
        .collect();
    policies.judges = (0..6)
        .map(|_| Box::new(Illegal) as Box<dyn JudgmentPolicy>)
        .collect();
    let r = run_with(c.clone(), &mut policies).unwrap();
    assert_eq!(r.decisions.len(), 18);
    assert!(
        r.decisions
            .iter()
            .all(|d| d.classification == Classification::Rejected)
    );
    assert!(r.testimony.is_empty());
    assert_eq!(r.tally, [0; 6]);
    assert_eq!(r.verdict, None);
    assert!(matches!(
        r.objective_history[r.objective_history.len() - 2],
        Event::Transition(Phase::Terminated)
    ));
    let mut bad = c.clone();
    bad.order = [0; 6];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.version = "invalid".into();
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.later = vec![
        LaterObservation {
            agent: 0,
            candidate: None,
            before_slot: 6
        };
        13
    ];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.second_routes[0] = vec![5, 5];
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.later.push(LaterObservation {
        agent: 6,
        candidate: None,
        before_slot: 6,
    });
    assert!(run(bad).is_err());
    let mut bad = c.clone();
    bad.later.push(LaterObservation {
        agent: 0,
        candidate: Some(99),
        before_slot: 6,
    });
    assert!(run(bad).is_err());
    policies.listeners.pop();
    assert!(run_with(c, &mut policies).is_err());
}
#[test]
fn primary_replay_and_matched_controls_pass_before_freezing() {
    let e = evaluate().unwrap();
    let frozen = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/evidence/experiment-005.json"
    ))
    .unwrap();
    assert_eq!(serde_json::to_string_pretty(&e).unwrap(), frozen);
    assert_eq!(e.runs, 576);
    assert_eq!(e.replay_matches, 576);
    assert_eq!(e.listener_pairs.len(), 288);
    assert_eq!(e.mechanism_pairs.len(), 384);
    assert!(
        e.fixtures
            .iter()
            .all(|f| f.counts.rejected == 0 && f.counts.max_depth <= 3)
    );
    assert!(
        e.listener_pairs
            .iter()
            .all(|p| p.changed_config_fields == vec!["listeners"])
    );
    for p in &e.mechanism_pairs {
        if p.control == Fixture::Retain && p.treatment == Fixture::SelectiveHigh {
            assert!(p.identical_target_inputs && p.identical_target_assessments && !p.vote_changed);
        }
        if p.control == Fixture::Sincere && p.treatment == Fixture::Injected {
            assert!(
                p.identical_wire_and_recipients
                    && p.identical_target_inputs
                    && p.identical_target_assessments
            );
        }
    }
    for row in &e.fixtures {
        if row.listener == ListenerKind::Revision {
            match row.fixture {
                Fixture::Sincere
                | Fixture::SelectiveLow
                | Fixture::RelayBoth
                | Fixture::Injected => assert_eq!(row.counts.correct_verdicts, 16),
                Fixture::Retraction
                | Fixture::RelayFinal
                | Fixture::MissingOriginal
                | Fixture::Independent
                | Fixture::Branches
                | Fixture::Dispute => assert_eq!(row.counts.no_verdicts, 16, "{:?}", row.fixture),
                _ => assert_eq!(row.counts.wrong_verdicts, 16, "{:?}", row.fixture),
            }
        }
    }
}

#[test]
fn equal_visible_corrections_cannot_reveal_hidden_intent() {
    for l in [ListenerKind::Frozen004, ListenerKind::Revision] {
        let a = fixture(Fixture::WrongSincere, l);
        let b = fixture(Fixture::Misleading, l);
        assert!(
            a.testimony
                .iter()
                .map(|t| &t.message)
                .eq(b.testimony.iter().map(|t| &t.message))
        );
        assert_ne!(a.testimony[1].classification, b.testimony[1].classification);
        for (x, y) in a
            .updates
            .iter()
            .filter(|u| u.input.id == 5)
            .zip(b.updates.iter().filter(|u| u.input.id == 5))
        {
            assert_eq!(x.input, y.input);
            assert_eq!(x.assessment, y.assessment);
        }
        assert_eq!(judgment(&a, 5).choice.action, judgment(&b, 5).choice.action);
    }
}

#[test]
fn revision_evidence_depth_three_passes_and_four_is_rejected() {
    let mut c = Config::new(0);
    let p = (c.actor() + 1) % 6;
    let q = c.actor();
    c.observations[0] = Some(p);
    c.later.push(LaterObservation {
        before_slot: 6,
        agent: 0,
        candidate: Some(q),
    });
    for speaker in 0..4 {
        let mut choice = Choice::silence("bounded chain validation injection");
        choice.action = Action::Speak {
            body: Body::Assert(p),
            attribution: if speaker == 0 {
                Attribution::Firsthand
            } else {
                Attribution::Reference(speaker as usize - 1)
            },
            basis: if speaker == 0 {
                Basis::Private
            } else {
                Basis::ReceivedAssertion(speaker as usize - 1)
            },
            recipients: (0..6).collect(),
        };
        choice.intent = if speaker == 0 {
            Intent::Sincere
        } else {
            Intent::Faithful
        };
        c.injections.push(Injection {
            before_slot: 0,
            speaker,
            choice,
        });
    }
    for speaker in 0..4 {
        let mut choice = Choice::silence("bounded forwarded revision validation injection");
        let parent = if speaker == 0 {
            None
        } else {
            Some(3 + speaker as usize)
        };
        choice.action = Action::Speak {
            body: Body::Revise {
                target: speaker as usize,
                replacement: Some(q),
                forwarded: parent,
            },
            attribution: Attribution::Unspecified,
            basis: parent
                .map(Basis::ReceivedRevision)
                .unwrap_or(Basis::Private),
            recipients: (0..6).collect(),
        };
        choice.intent = if speaker == 0 {
            Intent::Sincere
        } else {
            Intent::Faithful
        };
        c.injections.push(Injection {
            before_slot: 6,
            speaker,
            choice,
        });
    }
    let r = run(c.clone()).unwrap();
    assert_eq!(r.testimony[7].lineage.depth, 3);
    assert_eq!(r.testimony[7].lineage.roots, vec![Root::Observation(6)]);
    let mut choice = Choice::silence("invalid depth four");
    choice.action = Action::Speak {
        body: Body::Assert(p),
        attribution: Attribution::Reference(3),
        basis: Basis::ReceivedAssertion(3),
        recipients: (0..6).collect(),
    };
    c.injections.insert(
        4,
        Injection {
            before_slot: 0,
            speaker: 4,
            choice,
        },
    );
    assert!(run(c).unwrap_err().contains("depth"));
}

#[test]
fn cli_example_matches_library_and_refuses_existing_files() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/experiments/005/sincere.json");
    let c: Config = serde_json::from_str(include_str!("../experiments/005/sincere.json")).unwrap();
    assert_eq!(
        c,
        fixture_config(Fixture::Sincere, 0, ListenerKind::Revision)
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_experiment005"))
        .args(["run", path])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<Report>(&output.stdout).unwrap(),
        run(c).unwrap()
    );
    let failed = std::process::Command::new(env!("CARGO_BIN_EXE_experiment005"))
        .args(["run", path, path])
        .output()
        .unwrap();
    assert!(!failed.status.success());
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        include_str!("../experiments/005/sincere.json")
    );
}

#[test]
fn misleading_revision_can_replace_a_correct_verdict() {
    // Engineering-only adverse control; no primary fixture or policy changes.
    let mut c = fixture_config(Fixture::Misleading, 0, ListenerKind::Revision);
    c.observations[0] = Some(c.actor());
    let aware = run(c.clone()).unwrap();
    c.listeners = [ListenerKind::Frozen004; 6];
    let frozen = run(c).unwrap();
    assert_eq!(frozen.correct, Some(true));
    assert_eq!(aware.correct, Some(false));
    assert_eq!(
        aware.testimony[1].classification,
        Classification::IntentionalContradiction
    );
    assert!(!aware.decisions[6].choice.recognized_change);
    assert!(
        aware
            .testimony
            .iter()
            .map(|t| &t.message)
            .eq(frozen.testimony.iter().map(|t| &t.message))
    );
}

struct FixedVote(Option<u8>);
impl JudgmentPolicy for FixedVote {
    fn choose(&mut self, _: &AgentView) -> Choice {
        let mut c = Choice::silence("declared engineering ballot control");
        c.action = Action::Vote(self.0);
        c
    }
}
#[test]
fn four_vote_threshold_and_abstention_are_resolved_without_substitution() {
    for (votes, expected) in [
        ([Some(0), Some(0), Some(0), Some(1), Some(1), None], None),
        ([Some(0), Some(0), Some(0), Some(0), None, None], Some(0)),
    ] {
        let c = Config::new(0);
        let mut p = Policies::from_config(&c);
        p.judges = votes
            .into_iter()
            .map(|v| Box::new(FixedVote(v)) as Box<dyn JudgmentPolicy>)
            .collect();
        let r = run_with(c, &mut p).unwrap();
        assert_eq!(r.verdict, expected);
        assert!(r.decisions.iter().all(|d| d.rejection.is_none()));
        assert_eq!(
            r.tally.iter().sum::<u8>(),
            votes.iter().filter(|v| v.is_some()).count() as u8
        );
    }
}
