use super::*;
use crate::{Rng, experiment011 as e11};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Treatment {
    Ordinary,
    Unsolicited,
    Unanswered,
    Random,
    Targeted,
    Selective,
    Misleading,
    Ignored,
}
pub const TREATMENTS: [Treatment; 8] = [
    Treatment::Ordinary,
    Treatment::Unsolicited,
    Treatment::Unanswered,
    Treatment::Random,
    Treatment::Targeted,
    Treatment::Selective,
    Treatment::Misleading,
    Treatment::Ignored,
];
pub fn generated(seed: u64, treatment: Treatment) -> Config {
    let base = e11::generated(seed, e11::Treatment::SilentDirect);
    let mut c = Config::new(base);
    c.base.social = [SocialKind::ProvenanceAwareSocial; 6];
    c.request = [match treatment {
        Treatment::Ordinary | Treatment::Unsolicited => RequestKind::NeverAsk,
        Treatment::Random => RequestKind::RandomAsk,
        _ => RequestKind::UncertaintyDrivenAsk,
    }; 6];
    c.response = [match treatment {
        Treatment::Ordinary | Treatment::Unsolicited | Treatment::Unanswered => {
            ResponseKind::NoReply
        }
        Treatment::Selective => ResponseKind::SelectiveReply,
        Treatment::Misleading => ResponseKind::MisleadingReplyControl,
        _ => ResponseKind::SincereReply,
    }; 6];
    if treatment == Treatment::Unsolicited {
        c.base.sharing = [SharingKind::SincereExperienceReport; 6];
    }
    if treatment == Treatment::Ignored {
        c.base.social = [SocialKind::DirectOnly; 6];
    }
    let mut rng = Rng::new(seed ^ 0x12012);
    for i in &mut c.base.base.incidents {
        for j in (1..6).rev() {
            let k = (rng.draw() % (j as u64 + 1)) as usize;
            i.game.order.swap(j, k);
        }
        for id in 0..6 {
            i.game.routes[id] = (0..6).filter(|_| rng.draw().is_multiple_of(2)).collect();
            i.game.second_routes[id] = (0..6).filter(|_| rng.draw().is_multiple_of(2)).collect();
        }
    }
    c
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diagnostic {
    AccurateRequested,
    NoResponse,
    NoExperience,
    WrongAnswer,
    SincereWrong,
    WrongRecipient,
    TargetedVsRandom,
    RelevantVsIrrelevant,
    QueryDisplaces,
    ReplyDisplaces,
    MultipleRequests,
    CredibleMisinformed,
    UnreliableInformed,
    LateAnswer,
    PrivateVsWider,
    KnownDuplicate,
    HiddenDuplicate,
    ResponseCorrection,
    ReliabilityReversal,
    Scarcity,
    DifferentHistories,
    IdentityOrder,
}
pub const DIAGNOSTICS: [Diagnostic; 22] = [
    Diagnostic::AccurateRequested,
    Diagnostic::NoResponse,
    Diagnostic::NoExperience,
    Diagnostic::WrongAnswer,
    Diagnostic::SincereWrong,
    Diagnostic::WrongRecipient,
    Diagnostic::TargetedVsRandom,
    Diagnostic::RelevantVsIrrelevant,
    Diagnostic::QueryDisplaces,
    Diagnostic::ReplyDisplaces,
    Diagnostic::MultipleRequests,
    Diagnostic::CredibleMisinformed,
    Diagnostic::UnreliableInformed,
    Diagnostic::LateAnswer,
    Diagnostic::PrivateVsWider,
    Diagnostic::KnownDuplicate,
    Diagnostic::HiddenDuplicate,
    Diagnostic::ResponseCorrection,
    Diagnostic::ReliabilityReversal,
    Diagnostic::Scarcity,
    Diagnostic::DifferentHistories,
    Diagnostic::IdentityOrder,
];
pub fn diagnostic(family: Diagnostic, seed: u64, listener: SocialKind) -> Config {
    use Diagnostic::*;
    let old = match family {
        SincereWrong | CredibleMisinformed => e11::Diagnostic::SincereInaccurate,
        UnreliableInformed => e11::Diagnostic::UnreliableCorrect,
        ResponseCorrection | ReliabilityReversal => e11::Diagnostic::ChangingReliability,
        DifferentHistories => e11::Diagnostic::DifferentHistories,
        _ => e11::Diagnostic::AccurateFirsthand,
    };
    let mut base = e11::diagnostic(old, seed, listener);
    base.sharing = [SharingKind::SilentMethod; 6];
    base.sharing_schedule.clear();
    let mut c = Config::new(base);
    c.start_episode = 2;
    c.request[5] = RequestKind::ExperienceGapAsk;
    c.response = [ResponseKind::SincereReply; 6];
    for (ep, i) in c.base.base.incidents.iter_mut().enumerate() {
        i.game.order = [0, 1, 2, 3, 4, 5];
        if ep < 2 {
            i.forced.retain(|(id, _)| *id != 5);
            i.forced.push((5, InvestigationAction::Inspect(0)));
        }
        if ep == 0 {
            // Declared mixed requester experience: one correct A and one incorrect A.
            if let Some(a) = i.artifacts.iter_mut().find(|a| a.descriptor.id == 0) {
                a.candidate = Some(i.game.actor());
            }
        }
        // Legal routing is a declared fixture constraint; it does not disclose expertise.
        i.game.routes[5] = vec![0];
        if matches!(family, QueryDisplaces | ReplyDisplaces) && ep >= 2 {
            i.game.observations[if family == QueryDisplaces { 5 } else { 0 }] =
                Some(if family == ReplyDisplaces {
                    (i.game.actor() + 1) % 6
                } else {
                    i.game.actor()
                });
            i.communication[if family == QueryDisplaces { 5 } else { 0 }] =
                CommunicationKind::Sincere;
        }
        if family == MultipleRequests && ep >= 2 {
            for id in 1..6 {
                i.game.routes[id] = vec![0];
            }
        }
        if matches!(family, WrongRecipient | NoExperience) {
            i.game.routes[5] = vec![1];
        }
        if family == RelevantVsIrrelevant && ep >= 2 {
            for a in &mut i.artifacts {
                if a.descriptor.id == 1 {
                    a.cost = 3;
                }
            }
        }
        if family == Scarcity {
            c.base.base.budgets = [3; 6];
        }
        if family == DifferentHistories && ep < 2 {
            i.forced.retain(|(id, _)| *id != 4);
            i.forced.push((4, InvestigationAction::Inspect(1)));
        }
        if family == CredibleMisinformed && ep < 2 {
            i.game.observations[0] = Some(i.game.actor());
            i.communication[0] = CommunicationKind::Sincere;
            i.game.second_routes[0] = vec![0];
        }
        if family == IdentityOrder {
            i.game.order = [5, 4, 3, 2, 1, 0];
            i.access_order = [3, 4, 5, 0, 1, 2];
            i.game.routes[5] = vec![0, 2];
        }
    }
    if family == ReplyDisplaces {
        for i in c.base.base.incidents.iter_mut().skip(2) {
            i.forced.retain(|(id, _)| *id != 0);
            i.forced.push((0, InvestigationAction::Inspect(1)));
        }
    }
    if family == NoResponse {
        c.response = [ResponseKind::NoReply; 6];
    }
    if family == NoExperience {
        c.response = [ResponseKind::UncertainReply; 6];
    }
    if family == WrongAnswer {
        c.response[0] = ResponseKind::MisleadingReplyControl;
    }
    if family == TargetedVsRandom {
        c.request[5] = RequestKind::RandomAsk;
        for i in &mut c.base.base.incidents {
            i.game.routes[5] = (0..5).collect();
        }
    }
    if family == MultipleRequests {
        for id in 1..6 {
            c.request[id] = RequestKind::ExperienceGapAsk;
        }
    }
    if matches!(family, CredibleMisinformed | UnreliableInformed) {
        c.request[5] = RequestKind::CredibilityGuidedAsk;
        for i in &mut c.base.base.incidents {
            i.game.routes[5] = vec![0, 1];
        }
    }
    if family == LateAnswer {
        c.late_responses = true;
    }
    if family == DifferentHistories {
        c.request[4] = RequestKind::NeverAsk;
        for i in &mut c.base.base.incidents {
            i.game.routes[4] = vec![0];
        }
    }
    if matches!(family, KnownDuplicate | HiddenDuplicate) {
        c.base.sharing[0] = SharingKind::SincereExperienceReport;
        c.base.sharing[1] = if family == KnownDuplicate {
            SharingKind::RelayExperience
        } else {
            SharingKind::MisleadingReportControl
        };
        c.base.sharing_schedule = (0..8)
            .map(|ep| {
                if ep < 2 {
                    [SharingKind::SilentMethod; 6]
                } else {
                    c.base.sharing
                }
            })
            .collect();
        for i in &mut c.base.base.incidents {
            i.game.routes[5] = vec![1];
        }
        c.request_type = Some(RequestType::ExperienceBasis);
    }
    if matches!(family, PrivateVsWider | DifferentHistories) {
        // Materialize an actual selected legal answer before replacing it with a wider control.
        if let Ok(r) = run(c.clone())
            && let Some(d) = r.episodes[2]
                .responses
                .iter()
                .find(|d| d.choice.selected.is_some() && d.rejection.is_none())
        {
            c.answer_injections.push(AnswerInjection {
                episode: 2,
                speaker: d.input.id(),
                window: Window::Response,
                choice: d.choice.clone(),
                wider: if family == DifferentHistories {
                    vec![4]
                } else {
                    vec![1, 2, 3, 4]
                },
            });
        }
    }
    if family == RelevantVsIrrelevant {
        c.request_type = Some(RequestType::MethodAvailability);
    }
    c
}
pub fn origin_pair(seed: u64) -> Result<[Config; 2], String> {
    let c = diagnostic(
        Diagnostic::AccurateRequested,
        seed,
        SocialKind::ProvenanceAwareSocial,
    );
    let r = run(c.clone())?;
    let d = r.episodes[2]
        .responses
        .iter()
        .find(|d| d.choice.selected.is_some() && d.rejection.is_none())
        .ok_or("no legal selected answer in origin control")?;
    let mut injected = c.clone();
    injected.answer_injections.push(AnswerInjection {
        episode: 2,
        speaker: d.input.id(),
        window: Window::Response,
        choice: d.choice.clone(),
        wider: vec![],
    });
    Ok([c, injected])
}
