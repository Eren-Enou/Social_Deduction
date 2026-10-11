use super::*;
pub fn validate_report_extended(v: &SharingView, c: &SharingChoice) -> Result<(), String> {
    v.personal.validate()?;
    v.social.validate()?;
    if v.personal.observer != v.incident.incident.id
        || v.social.observer != v.incident.incident.id
        || !v.reference.valid()
        || v.reference.slot >= 18
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
    let mut rejection = validate_report_extended(&input, &choice).err();
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
