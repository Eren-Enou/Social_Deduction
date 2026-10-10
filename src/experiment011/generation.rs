use super::*;
use crate::{Rng, experiment007 as e7, experiment010 as e10};
use e10::WorldGenerator;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Treatment {
    SilentDirect,
    SincereDirect,
    SincereNaive,
    SincereProvenance,
    SincereCredibility,
    MisleadingNaive,
    MisleadingProvenance,
    RelayProvenance,
    SelectiveProvenance,
    UncertainProvenance,
    FaultDirect,
    FaultProvenance,
}
pub const TREATMENTS: [Treatment; 12] = [
    Treatment::SilentDirect,
    Treatment::SincereDirect,
    Treatment::SincereNaive,
    Treatment::SincereProvenance,
    Treatment::SincereCredibility,
    Treatment::MisleadingNaive,
    Treatment::MisleadingProvenance,
    Treatment::RelayProvenance,
    Treatment::SelectiveProvenance,
    Treatment::UncertainProvenance,
    Treatment::FaultDirect,
    Treatment::FaultProvenance,
];
pub fn generated(seed: u64, treatment: Treatment) -> Config {
    let world = e10::Generator.generate(seed);
    let stream = if matches!(
        treatment,
        Treatment::FaultDirect | Treatment::FaultProvenance
    ) {
        e10::FeedbackStream::MixedFault
    } else {
        e10::FeedbackStream::Clean
    };
    let base = e10::generated_config(&world, e10::PolicyKind::ExperienceAdjusted, stream);
    let (sharing, listener) = match treatment {
        Treatment::SilentDirect => (SharingKind::SilentMethod, SocialKind::DirectOnly),
        Treatment::SincereDirect | Treatment::FaultDirect => {
            (SharingKind::SincereExperienceReport, SocialKind::DirectOnly)
        }
        Treatment::SincereNaive => (
            SharingKind::SincereExperienceReport,
            SocialKind::NaiveSocial,
        ),
        Treatment::SincereCredibility => (
            SharingKind::SincereExperienceReport,
            SocialKind::CredibilityWeightedSocial,
        ),
        Treatment::MisleadingNaive => (
            SharingKind::MisleadingReportControl,
            SocialKind::NaiveSocial,
        ),
        Treatment::MisleadingProvenance => (
            SharingKind::MisleadingReportControl,
            SocialKind::ProvenanceAwareSocial,
        ),
        Treatment::SelectiveProvenance => (
            SharingKind::SelectiveExperienceReport,
            SocialKind::ProvenanceAwareSocial,
        ),
        Treatment::UncertainProvenance => (
            SharingKind::UncertainReport,
            SocialKind::ProvenanceAwareSocial,
        ),
        _ => (
            SharingKind::SincereExperienceReport,
            SocialKind::ProvenanceAwareSocial,
        ),
    };
    let mut c = Config::new(base, sharing, listener);
    if treatment == Treatment::RelayProvenance {
        c.sharing[usize::from(world.identities[1])] = SharingKind::RelayExperience;
        c.sharing[usize::from(world.identities[2])] = SharingKind::RelayExperience;
    }
    let mut rng = Rng::new(seed ^ 0x11011);
    for i in &mut c.base.incidents {
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
    NoCommunication,
    AccurateFirsthand,
    SincereInaccurate,
    DeliberatelyMisleading,
    SelectiveWithholding,
    IndependentCorroboration,
    SharedOriginRelay,
    HiddenCommonOrigin,
    InconsistentReports,
    CorrectionAll,
    CorrectionSome,
    CorrectionNone,
    CorruptedHistory,
    CredibleFalse,
    UnreliableCorrect,
    SparseDirect,
    ContradictoryDirect,
    ChangingReliability,
    BeforeInvestigation,
    AfterInvestigation,
    DifferentHistories,
    IdentityOrder,
}
pub const DIAGNOSTICS: [Diagnostic; 22] = [
    Diagnostic::NoCommunication,
    Diagnostic::AccurateFirsthand,
    Diagnostic::SincereInaccurate,
    Diagnostic::DeliberatelyMisleading,
    Diagnostic::SelectiveWithholding,
    Diagnostic::IndependentCorroboration,
    Diagnostic::SharedOriginRelay,
    Diagnostic::HiddenCommonOrigin,
    Diagnostic::InconsistentReports,
    Diagnostic::CorrectionAll,
    Diagnostic::CorrectionSome,
    Diagnostic::CorrectionNone,
    Diagnostic::CorruptedHistory,
    Diagnostic::CredibleFalse,
    Diagnostic::UnreliableCorrect,
    Diagnostic::SparseDirect,
    Diagnostic::ContradictoryDirect,
    Diagnostic::ChangingReliability,
    Diagnostic::BeforeInvestigation,
    Diagnostic::AfterInvestigation,
    Diagnostic::DifferentHistories,
    Diagnostic::IdentityOrder,
];
pub fn diagnostic(family: Diagnostic, seed: u64, listener: SocialKind) -> Config {
    use Diagnostic::*;
    let mut base = e10::diagnostic_config(
        e10::Diagnostic::AccurateRecurring,
        seed,
        e10::PolicyKind::ExperienceAdjusted,
        false,
    );
    base.budgets = [16; 6];
    let good_b = !matches!(
        family,
        DeliberatelyMisleading
            | SincereInaccurate
            | CorruptedHistory
            | CredibleFalse
            | ContradictoryDirect
    );
    let change = matches!(
        family,
        CorrectionAll | CorrectionSome | CorrectionNone | ChangingReliability
    );
    for p in &mut base.processes {
        let good = (p.family.key.ends_with('b')) == good_b;
        p.accuracy_before = if good { 4 } else { 0 };
        p.accuracy_after = if good ^ change { 4 } else { 0 };
        p.missing_before = 0;
        p.missing_after = 0;
        p.shift_episode = change.then_some(3);
    }
    for (episode, i) in base.incidents.iter_mut().enumerate() {
        let actor = i.game.actor();
        i.game.observations = [None; 6];
        i.communication = [CommunicationKind::Silent; 6];
        i.game.routes = std::array::from_fn(|_| (0..6).collect());
        i.game.second_routes = i.game.routes.clone();
        i.forced.clear();
        for a in &mut i.artifacts {
            a.cost = 1;
            a.declared_accuracy = 50;
            a.declared_missing = 0;
            a.inspectors = (0..6).collect();
            a.candidate = Some(
                if (a.descriptor.id == 1) == (good_b ^ (change && episode >= 3)) {
                    actor
                } else {
                    (actor + 1) % 6
                },
            );
            a.separate_acquisition_disclosed = false;
        }
        if episode < 2 {
            for id in 0..6 {
                i.forced.push((
                    id,
                    if id == 0 {
                        InvestigationAction::Inspect(1)
                    } else {
                        InvestigationAction::Decline
                    },
                ));
            }
            if matches!(family, IndependentCorroboration | InconsistentReports) {
                i.forced[1].1 = if episode == 1 {
                    InvestigationAction::Inspect(1)
                } else {
                    InvestigationAction::Decline
                };
            }
            if matches!(family, ContradictoryDirect | DifferentHistories) {
                i.forced[5].1 = InvestigationAction::Inspect(1);
            }
            if matches!(family, CredibleFalse | UnreliableCorrect) {
                i.game.observations[0] = Some(if family == CredibleFalse {
                    actor
                } else {
                    (actor + 1) % 6
                });
                i.communication[0] = CommunicationKind::Sincere;
                i.game.second_routes[0] = vec![0];
            }
        }
        if change {
            i.forced.retain(|(id, _)| *id != 0);
            i.forced.push((0, InvestigationAction::Inspect(1)));
        }
        if matches!(family, SharedOriginRelay | HiddenCommonOrigin) {
            for id in [1, 2] {
                i.forced.retain(|(x, _)| *x != id);
                i.forced.push((id, InvestigationAction::Decline));
            }
        }
        if matches!(family, CorrectionSome | CorrectionNone) && episode >= 3 {
            i.game.routes[0] = if family == CorrectionNone {
                vec![0]
            } else {
                vec![0, 1, 3]
            };
            i.game.second_routes[0] = i.game.routes[0].clone();
        }
        if family == SelectiveWithholding {
            i.game.revision_cost[0] = 3;
        }
        if family == IdentityOrder {
            i.game.order = [5, 4, 3, 2, 1, 0];
            i.access_order = [3, 4, 5, 0, 1, 2];
        }
    }
    base.feedback.clear();
    for (episode, i) in base.incidents.iter().enumerate() {
        let actor = i.game.actor();
        let false_speaker = matches!(
            family,
            SincereInaccurate | CorruptedHistory | ContradictoryDirect | DifferentHistories
        );
        if false_speaker || family == InconsistentReports {
            let bad = if family == InconsistentReports { 1 } else { 0 };
            base.feedback.push(e10::FeedbackPlan {
                after_episode: episode as u8,
                about_episode: episode as u8,
                recipients: vec![bad],
                token: e7::Token::outcome(base.session, episode as u8, 0, (actor + 1) % 6),
            });
            base.feedback.push(e10::FeedbackPlan {
                after_episode: episode as u8,
                about_episode: episode as u8,
                recipients: (0..6).filter(|id| *id != bad).collect(),
                token: e7::Token::outcome(base.session, episode as u8, 0, actor),
            });
        } else {
            base.feedback.push(e10::FeedbackPlan {
                after_episode: episode as u8,
                about_episode: episode as u8,
                recipients: (0..6).collect(),
                token: e7::Token::outcome(base.session, episode as u8, 0, actor),
            });
        }
    }
    let mut c = Config::new(base, SharingKind::SilentMethod, listener);
    c.sharing[0] = match family {
        NoCommunication => SharingKind::SilentMethod,
        SelectiveWithholding => SharingKind::SelectiveExperienceReport,
        DeliberatelyMisleading | CredibleFalse | DifferentHistories => {
            SharingKind::MisleadingReportControl
        }
        _ => SharingKind::SincereExperienceReport,
    };
    if matches!(family, IndependentCorroboration | InconsistentReports) {
        c.sharing[1] = SharingKind::SincereExperienceReport;
    }
    if matches!(family, SharedOriginRelay | HiddenCommonOrigin) {
        for id in [1, 2] {
            c.sharing[id] = if family == HiddenCommonOrigin {
                SharingKind::MisleadingReportControl
            } else {
                SharingKind::RelayExperience
            };
        }
    }
    c.sharing_schedule = (0..8)
        .map(|episode| {
            if episode < 2 {
                [SharingKind::SilentMethod; 6]
            } else {
                c.sharing
            }
        })
        .collect();
    if family == SparseDirect {
        c.base.feedback.retain(|f| f.about_episode != 1);
    }
    c.report_round = match family {
        BeforeInvestigation => Some(0),
        AfterInvestigation => Some(1),
        _ => None,
    };
    if family == IdentityOrder {
        let map = [3, 4, 5, 0, 1, 2];
        fn mapped<T: Clone>(a: &[T; 6], map: [usize; 6]) -> [T; 6] {
            let mut out = a.clone();
            for old in 0..6 {
                out[map[old]] = a[old].clone();
            }
            out
        }
        c.sharing = mapped(&c.sharing, map);
        c.social = mapped(&c.social, map);
        for s in &mut c.sharing_schedule {
            *s = mapped(s, map);
        }
        for i in &mut c.base.incidents {
            i.game.observations = mapped(&i.game.observations, map);
            i.communication = mapped(&i.communication, map);
            i.game.routes = mapped(&i.game.routes, map);
            i.game.second_routes = mapped(&i.game.second_routes, map);
            for route in i.game.routes.iter_mut().chain(&mut i.game.second_routes) {
                for id in route {
                    *id = map[usize::from(*id)] as u8;
                }
            }
            for id in i.game.order.iter_mut().chain(&mut i.access_order) {
                *id = map[usize::from(*id)] as u8;
            }
            for (id, _) in &mut i.forced {
                *id = map[usize::from(*id)] as u8;
            }
        }
        for f in &mut c.base.feedback {
            for id in &mut f.recipients {
                *id = map[usize::from(*id)] as u8;
            }
        }
    }
    c
}
/// Executed social histories, equal present observations/catalog/budget for observer5.
pub fn history_pair(seed: u64) -> [Config; 2] {
    let mut a = diagnostic(
        Diagnostic::AccurateFirsthand,
        seed,
        SocialKind::ProvenanceAwareSocial,
    );
    for (episode, i) in a.base.incidents.iter_mut().enumerate() {
        i.forced.retain(|(id, _)| *id != 0 && *id != 5);
        i.forced.push((0, InvestigationAction::Inspect(1)));
        i.forced.push((5, InvestigationAction::Decline));
        if episode == 7 {
            i.forced.retain(|(id, _)| *id != 5);
        }
    }
    a.sharing_schedule[7] = [SharingKind::SilentMethod; 6];
    let mut b = a.clone();
    for f in b.base.feedback.iter_mut().filter(|f| f.about_episode < 7) {
        f.recipients.retain(|r| *r != 0);
    }
    for episode in 0..7 {
        b.base.feedback.push(e10::FeedbackPlan {
            after_episode: episode,
            about_episode: episode,
            recipients: vec![0],
            token: e7::Token::outcome(
                b.session,
                episode,
                0,
                (b.incidents[usize::from(episode)].game.actor() + 1) % 6,
            ),
        });
    }
    [a, b]
}
