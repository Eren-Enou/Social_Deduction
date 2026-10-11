use super::*;
use crate::{Rng, experiment011 as e11};
use std::collections::BTreeSet;

pub trait RequestPolicy {
    fn choose(&mut self, v: &InquiryView) -> RequestChoice;
}
pub trait ResponsePolicy {
    fn choose(&mut self, v: &InquiryView) -> ResponseChoice;
}
pub struct Requester {
    pub kind: RequestKind,
    rng: Rng,
}
impl Requester {
    pub fn new(kind: RequestKind, seed: u64, id: u8) -> Self {
        Self {
            kind,
            rng: Rng::new(seed ^ 0x12012 ^ u64::from(id)),
        }
    }
}
fn recognized(v: &InquiryView, family: &MethodId) -> bool {
    v.local.catalog.iter().any(|m| &m.family == family)
        || v.local.personal.summaries.contains_key(family)
        || v.local
            .social
            .received
            .iter()
            .any(|m| &m.payload.family == family)
}
pub fn request_options(v: &InquiryView, kind: RequestKind) -> Vec<RequestOption> {
    let mut options = vec![];
    let mut families = BTreeSet::new();
    let estimates: Vec<_> = v
        .local
        .catalog
        .iter()
        .map(|m| {
            SocialKind::ProvenanceAwareSocial
                .assess(&SocialView {
                    observer: v.id(),
                    family: m.family.clone(),
                    personal: v
                        .local
                        .personal
                        .summaries
                        .get(&m.family)
                        .copied()
                        .unwrap_or_default(),
                    memory: v.local.social.clone(),
                    credits: v.local.credits,
                })
                .effective_accuracy
        })
        .collect();
    let close = estimates.iter().enumerate().any(|(i, a)| {
        estimates.iter().skip(i + 1).any(|b| {
            (a.numerator * b.denominator).abs_diff(b.numerator * a.denominator) * 6
                <= a.denominator * b.denominator
        })
    });
    for m in &v.local.catalog {
        if m.source.cost > v.balance || !families.insert(m.family.clone()) {
            continue;
        }
        let c = v
            .local
            .personal
            .summaries
            .get(&m.family)
            .copied()
            .unwrap_or_default();
        let estimate = v.local.social.clone();
        let e = SocialKind::ProvenanceAwareSocial.assess(&SocialView {
            observer: v.id(),
            family: m.family.clone(),
            personal: c,
            memory: estimate,
            credits: v.local.credits,
        });
        for &recipient in &v.local.incident.incident.permitted_recipients {
            if recipient == v.id() {
                continue;
            }
            let prior = v
                .local
                .social
                .received
                .iter()
                .rev()
                .find(|x| x.speaker == recipient && x.payload.family == m.family);
            for request_type in [
                RequestType::MethodReliability,
                RequestType::MethodAvailability,
                RequestType::ExperienceBasis,
            ] {
                if v.permitted_request_type
                    .is_some_and(|kind| kind != request_type)
                    || (request_type == RequestType::ExperienceBasis && prior.is_none())
                {
                    continue;
                }
                let credit = v.local.credits[usize::from(recipient)].score;
                let history = &v.memory.responsiveness[usize::from(recipient)];
                let responsiveness =
                    i32::from(history.replies.min(10)) - i32::from(history.requests.min(10));
                let mut score = match request_type {
                    RequestType::MethodReliability => {
                        (if c.graded() < 3 { 300 } else { 0 })
                            + if c.success > 0 && c.failure > 0 {
                                100
                            } else {
                                0
                            }
                            + if close { 100 } else { 0 }
                    }
                    RequestType::MethodAvailability => {
                        (if c.observed() < 3 { 200 } else { 0 })
                            + if c.missing > 0 && c.nonmissing > 0 {
                                100
                            } else {
                                0
                            }
                    }
                    RequestType::ExperienceBasis => {
                        if prior.is_some_and(|p| {
                            p.payload.rating == Rating::Uncertain
                                || matches!(
                                    p.payload.basis,
                                    ClaimedBasis::Relay | ClaimedBasis::Unsupported
                                )
                        }) {
                            200
                        } else {
                            0
                        }
                    }
                };
                if prior.is_some() {
                    score += 100;
                }
                if kind == RequestKind::ExperienceGapAsk {
                    score = if c.graded() == 0 {
                        500
                    } else {
                        (300 - 100 * i32::try_from(c.graded()).unwrap_or(300)).max(0)
                    } + if prior.is_some() { 100 } else { 0 };
                }
                if kind == RequestKind::CredibilityGuidedAsk && score > 0 {
                    score += 100 * i32::from(credit) + 10 * responsiveness;
                }
                if kind == RequestKind::CostAwareAsk {
                    score -= if v.local.incident.incident.current.candidate.is_some()
                        && v.local.incident.incident.current.margin >= 2
                    {
                        350
                    } else {
                        100
                    };
                }
                options.push(RequestOption {
                    recipient,
                    family: m.family.clone(),
                    kind: request_type,
                    prior: if request_type == RequestType::ExperienceBasis {
                        prior.map(|p| p.reference)
                    } else {
                        None
                    },
                    counts: c,
                    accuracy: e.effective_accuracy,
                    missingness: e.effective_missing,
                    prior_report: prior.is_some(),
                    credit,
                    responsiveness,
                    score,
                });
            }
        }
    }
    options.sort_by(|a, b| (&a.family, a.recipient, a.kind).cmp(&(&b.family, b.recipient, b.kind)));
    options
}
impl RequestPolicy for Requester {
    fn choose(&mut self, v: &InquiryView) -> RequestChoice {
        let options = request_options(v, self.kind);
        let index = if v.capacity == 0
            || v.window != Window::First
            || self.kind == RequestKind::NeverAsk
            || options.is_empty()
        {
            None
        } else if self.kind == RequestKind::RandomAsk {
            if self.rng.draw().is_multiple_of(2) {
                Some((self.rng.draw() % options.len() as u64) as usize)
            } else {
                None
            }
        } else {
            options
                .iter()
                .enumerate()
                .filter(|(_, o)| o.score > 0)
                .max_by(|(a, x), (b, y)| x.score.cmp(&y.score).then_with(|| b.cmp(a)))
                .map(|(i, _)| i)
        };
        let selected = index.map(|i| {
            let o = &options[i];
            Request {
                version: "request-012-v1".into(),
                reference: v.local.reference,
                requester: v.id(),
                recipient: o.recipient,
                family: o.family.clone(),
                kind: o.kind,
                prior: o.prior,
            }
        });
        RequestChoice {
            options,
            selected,
            reason: format!(
                "inquiry-policy-012-v1 {:?}; strict local relevance gate",
                self.kind
            ),
        }
    }
}
pub fn validate_request(v: &InquiryView, q: &Request) -> Result<(), String> {
    q.validate()?;
    v.memory.validate()?;
    v.local.personal.validate()?;
    v.local.social.validate()?;
    if v.permitted_request_type.is_some_and(|kind| kind != q.kind)
        || v.capacity == 0
        || v.window != Window::First
        || q.reference != v.local.reference
        || q.requester != v.id()
        || v.memory.observer != v.id()
        || v.local.personal.observer != v.id()
        || v.local.social.observer != v.id()
        || !v
            .local
            .incident
            .incident
            .permitted_recipients
            .contains(&q.recipient)
        || !recognized(v, &q.family)
    {
        return Err("unavailable request opportunity/recipient/family".into());
    }
    if v.memory.sent.iter().any(|x| {
        x.reference == q.reference
            || (x.reference.episode == q.reference.episode
                && x.recipient == q.recipient
                && x.family == q.family
                && x.kind == q.kind)
    }) {
        return Err("duplicate request".into());
    }
    if let Some(prior) = q.prior
        && !v.local.social.received.iter().any(|p| {
            p.reference == prior
                && p.reference < q.reference
                && p.speaker == q.recipient
                && p.payload.family == q.family
        })
    {
        return Err("inaccessible prior report".into());
    }
    Ok(())
}
fn bounded(c: Counts) -> Counts {
    Counts {
        success: c.success.min(100),
        failure: c.failure.min(100),
        missing: c.missing.min(100),
        nonmissing: c.nonmissing.min(100),
    }
}
impl ResponsePolicy for ResponseKind {
    fn choose(&mut self, v: &InquiryView) -> ResponseChoice {
        let mut requests: Vec<_> = v
            .memory
            .received
            .iter()
            .filter(|q| {
                q.reference < v.local.reference && !v.memory.answered.contains(&q.reference)
            })
            .collect();
        requests.sort_by_key(|q| {
            (
                !v.local
                    .personal
                    .summaries
                    .get(&q.family)
                    .is_some_and(|c| c.observed() > 0),
                q.reference,
            )
        });
        let eligible = requests.iter().map(|q| q.reference).collect();
        let mut out = ResponseChoice {
            eligible,
            selected: None,
            basis: PrivateBasis::Unsupported,
            intent: MethodIntent::Withheld,
            reason: format!("response-policy-012-v1 {:?}", self),
        };
        if v.capacity == 0 || v.window == Window::First || *self == Self::NoReply {
            return out;
        }
        let Some(q) = requests.first() else {
            return out;
        };
        if *self == Self::SelectiveReply
            && v.local.incident.incident.current.candidate.is_some()
            && v.local.incident.incident.current.margin >= 2
        {
            out.reason
                .push_str("; preserve capacity for incident testimony");
            return out;
        }
        let c = v
            .local
            .personal
            .summaries
            .get(&q.family)
            .copied()
            .unwrap_or_default();
        let insufficient = c.observed() == 0;
        let prior = q.prior.and_then(|r| {
            v.local
                .social
                .sent
                .iter()
                .find(|m| m.reference == r && m.payload.family == q.family)
        });
        let answer = if *self == Self::SelectiveReply && insufficient {
            Answer::Refuse
        } else if (insufficient
            && *self != Self::MisleadingReplyControl
            && !(q.kind == RequestType::ExperienceBasis && prior.is_some()))
            || (q.kind == RequestType::ExperienceBasis && prior.is_none())
        {
            Answer::InsufficientInformation
        } else {
            let mut payload = if q.kind == RequestType::ExperienceBasis {
                prior.unwrap().payload.clone()
            } else {
                Payload {
                    family: q.family.clone(),
                    rating: rating(c),
                    counts: Some(bounded(c)),
                    basis: if insufficient {
                        ClaimedBasis::Unsupported
                    } else {
                        ClaimedBasis::Summary
                    },
                    claimed_origin: Some(v.id()),
                    parent: None,
                    acquisition: v
                        .local
                        .personal
                        .records
                        .iter()
                        .rev()
                        .find(|r| r.method == q.family)
                        .map(|r| r.finding.acquisition),
                    verification_claimed: c.graded() > 0,
                }
            };
            out.basis = if insufficient {
                PrivateBasis::Unsupported
            } else {
                PrivateBasis::Personal(q.family.clone())
            };
            out.intent = MethodIntent::Sincere;
            if q.kind == RequestType::ExperienceBasis {
                let retained_parent = payload.parent.or_else(|| {
                    v.local
                        .social
                        .received
                        .iter()
                        .find(|m| {
                            m.payload.family == payload.family
                                && m.payload.rating == payload.rating
                                && m.payload.counts == payload.counts
                                && m.payload.acquisition == payload.acquisition
                                && m.payload.verification_claimed == payload.verification_claimed
                        })
                        .map(|m| m.reference)
                });
                out.basis = if let Some(parent) = retained_parent {
                    PrivateBasis::Received(parent)
                } else if insufficient {
                    PrivateBasis::Unsupported
                } else {
                    PrivateBasis::Personal(q.family.clone())
                };
            }
            if *self == Self::UncertainReply && (c.graded() < 2 || (c.success > 0 && c.failure > 0))
            {
                payload.rating = Rating::Uncertain;
            }
            if q.kind == RequestType::MethodAvailability {
                payload.rating = Rating::Uncertain;
            }
            if *self == Self::MisleadingReplyControl {
                out.intent = if payload.rating == Rating::Uncertain {
                    MethodIntent::Unsupported
                } else {
                    MethodIntent::Contrary
                };
                payload.rating = if payload.rating == Rating::Favorable {
                    Rating::Unfavorable
                } else {
                    Rating::Favorable
                };
                if q.kind == RequestType::MethodAvailability {
                    payload.rating = Rating::Uncertain;
                    if let Some(counts) = &mut payload.counts {
                        out.intent = if counts.missing == counts.nonmissing {
                            MethodIntent::Unsupported
                        } else {
                            MethodIntent::Contrary
                        };
                        std::mem::swap(&mut counts.missing, &mut counts.nonmissing);
                    }
                }
                // A deliberately constructed contrary assessment is not an exact relay.
                payload.parent = None;
                payload.basis = if insufficient {
                    ClaimedBasis::Unsupported
                } else {
                    ClaimedBasis::Summary
                };
                out.basis = if insufficient {
                    PrivateBasis::Unsupported
                } else {
                    PrivateBasis::Personal(q.family.clone())
                };
            }
            let old = v
                .local
                .social
                .sent
                .iter()
                .rev()
                .find(|m| m.payload.family == q.family);
            Answer::Assessment {
                retract: payload.rating == Rating::Uncertain
                    && old.is_some_and(|m| m.payload.rating != Rating::Uncertain),
                payload,
                revision: old.map(|m| m.reference),
            }
        };
        out.selected = Some(Response {
            version: "response-012-v1".into(),
            reference: v.local.reference,
            speaker: v.id(),
            recipient: q.requester,
            request: q.reference,
            answer,
        });
        out
    }
}
pub fn response_sharing(_v: &InquiryView, c: &ResponseChoice) -> Option<SharingChoice> {
    let a = c.selected.as_ref()?;
    let Answer::Assessment {
        payload,
        revision,
        retract,
    } = &a.answer
    else {
        return None;
    };
    Some(SharingChoice {
        action: MethodAction::Report {
            payload: payload.clone(),
            revision: *revision,
            retract: *retract,
            recipients: vec![a.recipient],
        },
        intent: c.intent,
        basis: c.basis.clone(),
        eligible: true,
        reason: c.reason.clone(),
    })
}
pub fn validate_response(v: &InquiryView, c: &ResponseChoice) -> Result<(), String> {
    v.memory.validate()?;
    v.local.personal.validate()?;
    v.local.social.validate()?;
    let Some(a) = &c.selected else {
        return Ok(());
    };
    a.validate()?;
    if v.capacity == 0
        || v.window == Window::First
        || (v.window == Window::Response && !(6..12).contains(&a.reference.slot))
        || (v.window == Window::Final && !(12..18).contains(&a.reference.slot))
        || a.reference != v.local.reference
        || a.speaker != v.id()
        || v.memory.observer != v.id()
        || v.local.personal.observer != v.id()
        || v.local.social.observer != v.id()
    {
        return Err("unavailable reply opportunity/identity".into());
    }
    let q = v
        .memory
        .received
        .iter()
        .find(|q| q.reference == a.request && q.recipient == v.id() && q.requester == a.recipient)
        .ok_or("reply without addressed retained request")?;
    if v.memory.answered.contains(&q.reference) {
        return Err("already answered request".into());
    }
    if let Answer::Assessment { payload, .. } = &a.answer {
        if payload.family != q.family {
            return Err("answer family mismatch".into());
        }
        let mut local = v.local.clone();
        local.incident.incident.permitted_recipients = vec![a.recipient];
        validate_report012(&local, &response_sharing(v, c).unwrap())?;
    }
    Ok(())
}
pub fn remember_request(m: &InquiryMemory, q: Request, own: bool) -> Result<InquiryEffect, String> {
    m.validate()?;
    q.validate()?;
    let mut after = m.clone();
    let mut evicted = vec![];
    if (own && q.requester != m.observer) || (!own && q.recipient != m.observer) {
        return Err("foreign request delivery".into());
    }
    let list = if own {
        &mut after.sent
    } else {
        &mut after.received
    };
    let duplicate = list.iter().any(|x| x.reference == q.reference);
    if !duplicate {
        list.push(q.clone());
        if list.len() > 12 {
            evicted.push(list.remove(0).reference);
        }
        if own {
            let s = &mut after.responsiveness[usize::from(q.recipient)];
            s.requests = s.requests.saturating_add(1);
        }
    }
    after.validate()?;
    Ok(InquiryEffect {
        before: m.clone(),
        after,
        evicted,
        duplicate,
        kind: if own {
            "sent request"
        } else {
            "received request"
        }
        .into(),
    })
}
pub fn remember_response(
    m: &InquiryMemory,
    a: Response,
    own: bool,
) -> Result<InquiryEffect, String> {
    m.validate()?;
    a.validate()?;
    let mut after = m.clone();
    let mut evicted = vec![];
    if (own && a.speaker != m.observer) || (!own && a.recipient != m.observer) {
        return Err("foreign response delivery".into());
    }
    let duplicate = if own {
        after.answered.contains(&a.request)
    } else {
        after.responses.iter().any(|x| x.reference == a.reference)
    };
    if !duplicate {
        if own {
            after.answered.push(a.request);
            if after.answered.len() > 12 {
                evicted.push(after.answered.remove(0));
            }
        } else {
            after.responses.push(a.clone());
            if after.responses.len() > 12 {
                evicted.push(after.responses.remove(0).reference);
            }
            let s = &mut after.responsiveness[usize::from(a.speaker)];
            s.replies = s.replies.saturating_add(1);
        }
    }
    after.validate()?;
    Ok(InquiryEffect {
        before: m.clone(),
        after,
        evicted,
        duplicate,
        kind: if own {
            "answered request"
        } else {
            "received response"
        }
        .into(),
    })
}
/// Adapt only the slot bound; keep the historical wire and its interpretation unchanged.
pub fn validate_report012(v: &SharingView, c: &SharingChoice) -> Result<(), String> {
    validate_report_extended(v, c)
}
pub fn social_message(a: &Response) -> Option<e11::Message> {
    if let Answer::Assessment {
        payload,
        revision,
        retract,
    } = &a.answer
    {
        Some(e11::Message {
            version: "method-report-011-v1".into(),
            reference: a.reference,
            speaker: a.speaker,
            payload: payload.clone(),
            revision: *revision,
            retract: *retract,
        })
    } else {
        None
    }
}
