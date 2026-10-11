use super::*;

pub trait SharingPolicy {
    fn choose(&mut self, v: &SharingView) -> SharingChoice;
}
fn fallback(eligible: bool, reason: &str) -> SharingChoice {
    SharingChoice {
        action: MethodAction::IncidentSpeech,
        intent: if eligible {
            MethodIntent::Withheld
        } else {
            MethodIntent::NoMethod
        },
        basis: PrivateBasis::Unsupported,
        eligible,
        reason: reason.into(),
    }
}
fn bounded(c: Counts) -> Counts {
    Counts {
        success: c.success.min(100),
        failure: c.failure.min(100),
        nonmissing: c.nonmissing.min(100),
        missing: c.missing.min(100),
    }
}
impl SharingPolicy for SharingKind {
    fn choose(&mut self, v: &SharingView) -> SharingChoice {
        if *self == Self::SilentMethod {
            return fallback(!v.personal.summaries.is_empty(), "silent method baseline");
        }
        if *self == Self::RelayExperience
            || (*self == Self::MisleadingReportControl && v.personal.summaries.is_empty())
        {
            if let Some(parent) = v.social.received.iter().rev().find(|m| {
                m.speaker != v.incident.incident.id
                    && !v
                        .social
                        .sent
                        .iter()
                        .any(|x| x.payload.parent == Some(m.reference))
            }) {
                let mut payload = parent.payload.clone();
                let hidden = *self == Self::MisleadingReportControl;
                payload.basis = if hidden {
                    ClaimedBasis::Firsthand
                } else {
                    ClaimedBasis::Relay
                };
                payload.claimed_origin = if hidden {
                    None
                } else {
                    Some(parent.payload.claimed_origin.unwrap_or(parent.speaker))
                };
                payload.parent = if hidden { None } else { Some(parent.reference) };
                let revision = parent.revision.and_then(|r| {
                    v.social
                        .sent
                        .iter()
                        .rev()
                        .find(|m| m.payload.parent == Some(r))
                        .map(|m| m.reference)
                });
                return SharingChoice {
                    action: MethodAction::Report {
                        payload,
                        revision,
                        retract: parent.retract,
                        recipients: v.incident.incident.permitted_recipients.clone(),
                    },
                    intent: if hidden {
                        MethodIntent::Unsupported
                    } else {
                        MethodIntent::FaithfulRelay
                    },
                    basis: PrivateBasis::Received(parent.reference),
                    eligible: true,
                    reason: "retained exact report forwarding".into(),
                };
            }
            if *self == Self::RelayExperience {
                return fallback(false, "no retained unforwarded report");
            }
        }
        let selected = v
            .personal
            .summaries
            .iter()
            .find(|(m, c)| c.observed() > 0 && v.catalog.iter().any(|x| x.family == **m))
            .or_else(|| v.personal.summaries.iter().find(|(_, c)| c.observed() > 0));
        let (family, counts) = if let Some((m, c)) = selected {
            (m.clone(), *c)
        } else if let Some(m) = v.catalog.first() {
            (m.family.clone(), Counts::default())
        } else {
            return fallback(false, "no recognizable method");
        };
        if matches!(
            *self,
            Self::SincereExperienceReport | Self::SelectiveExperienceReport
        ) && counts.observed() == 0
        {
            return fallback(false, "no personal observation to summarize");
        }
        if *self == Self::SelectiveExperienceReport
            && (counts.observed() < 2 || v.incident.incident.revision_cost >= 2)
        {
            return fallback(
                true,
                "benefit2 does not exceed local speech heuristic cost or insufficient exposure",
            );
        }
        let mut assessment = rating(counts);
        let intent = if *self == Self::MisleadingReportControl {
            if assessment == Rating::Uncertain {
                assessment = Rating::Favorable;
                MethodIntent::Unsupported
            } else {
                assessment = if assessment == Rating::Favorable {
                    Rating::Unfavorable
                } else {
                    Rating::Favorable
                };
                MethodIntent::Contrary
            }
        } else {
            MethodIntent::Sincere
        };
        if *self == Self::UncertainReport {
            if counts.graded() >= 2 && !(counts.success > 0 && counts.failure > 0) {
                return fallback(true, "no local insufficiency/conflict");
            }
            assessment = Rating::Uncertain;
        }
        let payload = Payload {
            family: family.clone(),
            rating: assessment,
            counts: Some(bounded(counts)),
            basis: if counts.observed() > 0 {
                ClaimedBasis::Summary
            } else {
                ClaimedBasis::Unsupported
            },
            claimed_origin: Some(v.incident.incident.id),
            parent: None,
            acquisition: v
                .personal
                .records
                .iter()
                .rev()
                .find(|r| r.method == family)
                .map(|r| r.finding.acquisition),
            verification_claimed: counts.graded() > 0,
        };
        let prior = v
            .social
            .sent
            .iter()
            .rev()
            .find(|m| m.payload.family == family && m.payload.parent.is_none());
        if prior.is_some_and(|m| m.payload == payload) {
            return fallback(true, "retained identical own summary; no repeat");
        }
        let retract = assessment == Rating::Uncertain
            && prior.is_some_and(|m| m.payload.rating != Rating::Uncertain);
        SharingChoice {
            action: MethodAction::Report {
                payload,
                revision: prior.map(|m| m.reference),
                retract,
                recipients: v.incident.incident.permitted_recipients.clone(),
            },
            intent,
            basis: if counts.observed() > 0 {
                PrivateBasis::Personal(family)
            } else {
                PrivateBasis::Unsupported
            },
            eligible: true,
            reason: "bounded local summary; current count state only".into(),
        }
    }
}
pub fn validate_report(v: &SharingView, c: &SharingChoice) -> Result<(), String> {
    v.personal.validate()?;
    v.social.validate()?;
    if v.personal.observer != v.incident.incident.id
        || v.social.observer != v.incident.incident.id
        || !v.reference.valid()
        || v.reference.slot >= 12
        || v.incident.incident.phase != crate::Phase::Discussion
        || v.incident.incident.opportunities_left == 0
        || v.personal
            .latest_episode
            .is_some_and(|episode| episode > v.reference.episode)
        || v.social
            .received
            .iter()
            .chain(&v.social.sent)
            .any(|m| m.reference >= v.reference)
    {
        return Err("invalid sharing identity/opportunity".into());
    }
    let MethodAction::Report {
        payload,
        revision,
        recipients,
        ..
    } = &c.action
    else {
        return Ok(());
    };
    payload.validate()?;
    if !v.catalog.iter().any(|m| m.family == payload.family)
        && !v.personal.summaries.contains_key(&payload.family)
        && !v
            .social
            .received
            .iter()
            .any(|m| m.payload.family == payload.family)
    {
        return Err("unrecognized method family".into());
    }
    if !crate::experiment004::unique_ids(recipients)
        || recipients
            .iter()
            .any(|r| !v.incident.incident.permitted_recipients.contains(r))
    {
        return Err("illegal method recipients".into());
    }
    if let Some(target) = revision
        && (*target >= v.reference
            || !v.social.sent.iter().any(|m| {
                m.reference == *target
                    && m.speaker == v.incident.incident.id
                    && m.payload.family == payload.family
            }))
    {
        return Err("unavailable/foreign method revision target".into());
    }
    if let Some(parent) = payload.parent
        && (parent >= v.reference || !v.social.received.iter().any(|m| m.reference == parent))
    {
        return Err("claimed parent not legitimately retained".into());
    }
    match &c.basis {
        PrivateBasis::Personal(family)
            if *family == payload.family && v.personal.summaries.contains_key(family) => {}
        PrivateBasis::Received(parent) => {
            let original = v
                .social
                .received
                .iter()
                .find(|m| m.reference == *parent && *parent < v.reference)
                .ok_or("unavailable relay parent")?;
            if original.payload.family != payload.family
                || original.payload.rating != payload.rating
                || original.payload.counts != payload.counts
                || original.payload.verification_claimed != payload.verification_claimed
                || original.payload.acquisition != payload.acquisition
            {
                return Err("relay content does not match retained parent".into());
            }
        }
        PrivateBasis::Unsupported => {}
        _ => return Err("unavailable personal basis".into()),
    }
    Ok(())
}
/// Resolver metadata stays outside the wire and outside both policy views.
pub fn resolve_report(
    input: SharingView,
    choice: SharingChoice,
    origin: Origin,
    ancestry: &std::collections::BTreeMap<Reference, (Vec<Reference>, u8)>,
) -> SocialDecision {
    let mut rejection = validate_report(&input, &choice).err();
    let (actual_origins, actual_depth) = match choice.basis {
        PrivateBasis::Received(parent) => ancestry
            .get(&parent)
            .map(|(roots, d)| (roots.clone(), d.saturating_add(1)))
            .unwrap_or((vec![], 4)),
        _ => (vec![input.reference], 0),
    };
    if actual_depth > 3 && matches!(choice.action, MethodAction::Report { .. }) {
        rejection = Some("method relay depth exceeds three".into());
    }
    let message = if rejection.is_none() {
        if let MethodAction::Report {
            payload,
            revision,
            retract,
            ..
        } = &choice.action
        {
            Some(Message {
                version: "method-report-011-v1".into(),
                reference: input.reference,
                speaker: input.incident.incident.id,
                payload: payload.clone(),
                revision: *revision,
                retract: *retract,
            })
        } else {
            None
        }
    } else {
        None
    };
    SocialDecision {
        input,
        choice,
        origin,
        rejection,
        message,
        actual_origins,
        actual_depth,
        deliveries: vec![],
    }
}
