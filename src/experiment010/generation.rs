use super::*;
use crate::{
    Rng, experiment005 as e5, experiment006 as e6, experiment007 as e7, experiment009 as e9,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackStream {
    Clean,
    MixedFault,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diagnostic {
    AccurateRecurring,
    InaccurateRecurring,
    EqualQuality,
    MisleadingAdvertisement,
    SparseHistory,
    MissingFeedback,
    AuthenticatedFalse,
    ContradictoryVerification,
    StableEnvironment,
    ReliabilityReversal,
    SelectiveExposure,
    Exploration,
    ResourceScarcity,
    KnownDependent,
    HiddenDependent,
    TrustedMethodFails,
    SeparateTestimony,
    FamiliarMethodNewArtifact,
}
pub const DIAGNOSTICS: [Diagnostic; 18] = [
    Diagnostic::AccurateRecurring,
    Diagnostic::InaccurateRecurring,
    Diagnostic::EqualQuality,
    Diagnostic::MisleadingAdvertisement,
    Diagnostic::SparseHistory,
    Diagnostic::MissingFeedback,
    Diagnostic::AuthenticatedFalse,
    Diagnostic::ContradictoryVerification,
    Diagnostic::StableEnvironment,
    Diagnostic::ReliabilityReversal,
    Diagnostic::SelectiveExposure,
    Diagnostic::Exploration,
    Diagnostic::ResourceScarcity,
    Diagnostic::KnownDependent,
    Diagnostic::HiddenDependent,
    Diagnostic::TrustedMethodFails,
    Diagnostic::SeparateTestimony,
    Diagnostic::FamiliarMethodNewArtifact,
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    pub identities: [u8; 6],
    pub base: Config,
}
pub trait WorldGenerator {
    fn generate(&mut self, seed: u64) -> World;
}
pub struct Generator;
fn shuffle<T>(rng: &mut Rng, a: &mut [T]) {
    for i in (1..a.len()).rev() {
        a.swap(i, (rng.draw() % (i as u64 + 1)) as usize);
    }
}
fn permutation(rng: &mut Rng) -> [u8; 6] {
    let mut a = [0, 1, 2, 3, 4, 5];
    shuffle(rng, &mut a);
    a
}
fn wrong(rng: &mut Rng, actor: u8) -> u8 {
    (actor + 1 + (rng.draw() % 5) as u8) % 6
}
fn draw_result(rng: &mut Rng, p: &MethodProcess, episode: u8, actor: u8) -> Option<u8> {
    if rng.draw() % 4 < p.missingness(episode).numerator {
        None
    } else {
        Some(if rng.draw() % 4 < p.accuracy(episode).numerator {
            actor
        } else {
            wrong(rng, actor)
        })
    }
}
fn clean_feedback(session: u64, incidents: &[Incident]) -> Vec<FeedbackPlan> {
    incidents
        .iter()
        .enumerate()
        .map(|(episode, c)| FeedbackPlan {
            after_episode: episode as u8,
            about_episode: episode as u8,
            recipients: (0..6).collect(),
            token: e7::Token::outcome(session, episode as u8, 0, c.game.actor()),
        })
        .collect()
}
impl WorldGenerator for Generator {
    fn generate(&mut self, seed: u64) -> World {
        let mut rng = Rng::new(seed);
        let identities = permutation(&mut rng);
        let order = permutation(&mut rng);
        let access_order = permutation(&mut rng);
        let budgets = std::array::from_fn(|_| 4 + (rng.draw() % 13) as u8);
        let mode = [
            CommunicationKind::Silent,
            CommunicationKind::ShareFinding,
            CommunicationKind::Sincere,
        ][(rng.draw() % 3) as usize];
        let shifted = seed % 2 == 1;
        let count = 2 + (rng.draw() % 3) as usize;
        let mut names = [
            "instrument-a",
            "instrument-b",
            "instrument-c",
            "instrument-d",
        ];
        shuffle(&mut rng, &mut names);
        let mut processes = vec![];
        let mut specs = vec![];
        for name in names.into_iter().take(count) {
            let accuracy = if rng.draw().is_multiple_of(2) { 1 } else { 3 };
            let missing = (rng.draw() % 3) as u8;
            let family = MethodId::new(name);
            processes.push(MethodProcess {
                family: family.clone(),
                accuracy_before: accuracy,
                accuracy_after: if shifted { 4 - accuracy } else { accuracy },
                missing_before: missing,
                missing_after: if shifted { 2 - missing } else { missing },
                shift_episode: shifted.then_some(4),
            });
            specs.push((
                family,
                1 + (rng.draw() % 3) as u8,
                25 + 25 * (rng.draw() % 4) as u8,
                25 * (rng.draw() % 3) as u8,
                [
                    e9::Category::Partial,
                    e9::Category::Physical,
                    e9::Category::Specialized,
                ][(rng.draw() % 3) as usize],
            ));
        }
        let mut incidents = vec![];
        for episode in 0..8 {
            let mut game = e5::Config::new(rng.draw());
            game.order = order;
            game.observations = [None; 6];
            let actor = game.actor();
            for &id in &identities[..2] {
                game.observations[usize::from(id)] = Some(if rng.draw() % 4 < 3 {
                    actor
                } else {
                    wrong(&mut rng, actor)
                });
            }
            for &id in &identities[2..] {
                if rng.draw().is_multiple_of(4) {
                    game.observations[usize::from(id)] = Some(if rng.draw().is_multiple_of(2) {
                        actor
                    } else {
                        wrong(&mut rng, actor)
                    });
                }
            }
            let mut artifacts = vec![];
            let mut methods = vec![];
            for (p, (family, cost, accuracy, missing, category)) in processes.iter().zip(&specs) {
                let mut id = (rng.draw() % 200) as u8;
                while artifacts.iter().any(|a: &Source| a.descriptor.id == id) {
                    id = id.wrapping_add(1);
                }
                let mut a = e9::source(id, draw_result(&mut rng, p, episode, actor), *cost);
                a.method_key = format!("{}-specimen", family.key);
                a.category = *category;
                a.declared_accuracy = *accuracy;
                a.declared_missing = *missing;
                a.separate_acquisition_disclosed = rng.draw().is_multiple_of(2);
                a.inspectors = (0..6).filter(|_| rng.draw().is_multiple_of(2)).collect();
                if a.inspectors.is_empty() {
                    a.inspectors.push(identities[2]);
                }
                methods.push((id, family.clone()));
                artifacts.push(a);
            }
            shuffle(&mut rng, &mut artifacts);
            incidents.push(Incident {
                game,
                artifacts,
                methods,
                forced: vec![],
                access_order,
                communication: [mode; 6],
                injected: vec![],
            });
        }
        let feedback = clean_feedback(seed, &incidents);
        World {
            seed,
            identities,
            base: Config {
                version: "series-010-v1".into(),
                session: seed,
                incidents,
                policy: [PolicyKind::NeverInvestigate; 6],
                budgets,
                listener: e6::ListenerKind::Credibility,
                feedback,
                processes,
            },
        }
    }
}
pub fn generated_config(world: &World, kind: PolicyKind, stream: FeedbackStream) -> Config {
    let mut c = world.base.clone();
    c.policy = [kind; 6];
    if stream == FeedbackStream::MixedFault {
        let mut rng = Rng::new(world.seed ^ 0x10010);
        c.feedback.clear();
        for (episode, i) in c.incidents.iter().enumerate() {
            if rng.draw().is_multiple_of(4) {
                continue;
            }
            let actor = if rng.draw().is_multiple_of(4) {
                wrong(&mut rng, i.game.actor())
            } else {
                i.game.actor()
            };
            let recipients = (0..6).filter(|_| rng.draw().is_multiple_of(2)).collect();
            c.feedback.push(FeedbackPlan {
                after_episode: episode as u8,
                about_episode: episode as u8,
                recipients,
                token: e7::Token::outcome(c.session, episode as u8, 0, actor),
            });
        }
    }
    c
}
pub fn diagnostic_config(family: Diagnostic, seed: u64, kind: PolicyKind, matched: bool) -> Config {
    let mut rng = Rng::new(seed ^ 0x1010);
    let reversal = family == Diagnostic::ReliabilityReversal;
    let low_a = matches!(
        family,
        Diagnostic::InaccurateRecurring | Diagnostic::MisleadingAdvertisement
    );
    let equal = family == Diagnostic::EqualQuality;
    let mut processes = (0..2)
        .map(|id| MethodProcess {
            family: MethodId::new(format!("instrument-{}", if id == 0 { "a" } else { "b" })),
            accuracy_before: if equal {
                2
            } else if (id == 0) ^ low_a {
                3
            } else {
                1
            },
            missing_before: 0,
            accuracy_after: if equal {
                2
            } else if ((id == 0) ^ low_a) ^ reversal {
                3
            } else {
                1
            },
            missing_after: 0,
            shift_episode: reversal.then_some(4),
        })
        .collect::<Vec<_>>();
    let mut incidents = vec![];
    for episode in 0..8 {
        let mut game = e5::Config::new(seed.wrapping_mul(16).wrapping_add(u64::from(episode)));
        let actor = game.actor();
        game.observations = [None; 6];
        game.observations[0] = Some(actor);
        game.observations[1] = Some((actor + 1) % 6);
        let dependent = matches!(
            family,
            Diagnostic::KnownDependent | Diagnostic::HiddenDependent
        );
        if dependent {
            game.observations[0] = Some(if rng.draw() % 4 < 3 {
                actor
            } else {
                wrong(&mut rng, actor)
            });
        }
        let mut artifacts = vec![];
        let mut methods = vec![];
        for (id, p) in processes.iter().enumerate() {
            let mut a = e9::source(
                id as u8,
                draw_result(&mut rng, p, episode, actor),
                if id == 0 { 1 } else { 2 },
            );
            a.method_key = format!("{}-specimen", p.family.key);
            a.coverage = (0..6).collect();
            a.declared_accuracy = if id == 0 { 95 } else { 50 };
            a.declared_missing = 0;
            if family == Diagnostic::EqualQuality {
                a.cost = 1;
                a.declared_accuracy = 75;
                a.category = e9::Category::Physical;
            }
            if family == Diagnostic::MisleadingAdvertisement {
                a.declared_accuracy = if id == 0 { 95 } else { 25 };
            }
            if family == Diagnostic::SelectiveExposure && episode < 4 {
                a.inspectors = if id == 0 {
                    vec![0, 1, 2]
                } else {
                    vec![3, 4, 5]
                };
            }
            if dependent {
                a.candidate = game.observations[0];
                a.actual_root = Root::Observation(0);
                a.descriptor.visible_parent = (family == Diagnostic::KnownDependent).then_some(0);
                a.category = e9::Category::Derived;
                a.separate_acquisition_disclosed = false;
            }
            if family == Diagnostic::TrustedMethodFails && id == 0 {
                a.candidate = Some(if episode < 4 { actor } else { (actor + 1) % 6 });
            }
            if family == Diagnostic::SparseHistory && (2..4).contains(&episode) {
                a.inspectors.clear();
            }
            methods.push((id as u8, p.family.clone()));
            artifacts.push(a);
        }
        let communication = if family == Diagnostic::SeparateTestimony {
            [CommunicationKind::ShareFinding; 6]
        } else {
            [CommunicationKind::Silent; 6]
        };
        let mut forced = vec![];
        if matched && episode < 4 {
            for id in 0..6 {
                let preferred = episode % 2;
                let a = artifacts
                    .iter()
                    .find(|a| a.descriptor.id == preferred && a.inspectors.contains(&id))
                    .or_else(|| artifacts.iter().find(|a| a.inspectors.contains(&id)));
                forced.push((
                    id,
                    a.map(|a| InvestigationAction::Inspect(a.descriptor.id))
                        .unwrap_or(InvestigationAction::Decline),
                ));
            }
        }
        let injected = if dependent {
            (0..6).map(|id| (id, 1)).collect()
        } else {
            vec![]
        };
        incidents.push(Incident {
            game,
            artifacts,
            methods,
            forced,
            access_order: [0, 1, 2, 3, 4, 5],
            communication,
            injected,
        });
    }
    if matches!(
        family,
        Diagnostic::KnownDependent | Diagnostic::HiddenDependent
    ) {
        for p in &mut processes {
            p.accuracy_before = 3;
            p.accuracy_after = 3;
        }
    }
    if family == Diagnostic::TrustedMethodFails {
        processes[0].accuracy_before = 4;
        processes[0].accuracy_after = 0;
        processes[0].shift_episode = Some(4);
    }
    let mut feedback = clean_feedback(seed, &incidents);
    if family == Diagnostic::MissingFeedback {
        feedback.clear();
    }
    if family == Diagnostic::AuthenticatedFalse {
        for f in &mut feedback {
            let actor = incidents[usize::from(f.about_episode)].game.actor();
            f.token.kind = e7::TokenKind::Outcome((actor + 1) % 6);
        }
    }
    if family == Diagnostic::ContradictoryVerification {
        for (episode, incident) in incidents.iter().enumerate() {
            let actor = incident.game.actor();
            feedback.push(FeedbackPlan {
                after_episode: episode as u8,
                about_episode: episode as u8,
                recipients: (0..6).collect(),
                token: e7::Token::outcome(seed, episode as u8, 1, (actor + 1) % 6),
            });
        }
    }
    Config {
        version: "series-010-v1".into(),
        session: seed,
        incidents,
        policy: [kind; 6],
        budgets: [if family == Diagnostic::ResourceScarcity {
            3
        } else {
            16
        }; 6],
        listener: e6::ListenerKind::Credibility,
        feedback,
        processes,
    }
}

/// Training is executed and paid; only the final assessment is voluntary.
pub fn fixed_current_configs(seed: u64, policy: PolicyKind, corrupt_feedback: bool) -> [Config; 2] {
    let first = diagnostic_config(Diagnostic::EqualQuality, seed, policy, true);
    std::array::from_fn(|variant| {
        let mut c = first.clone();
        c.incidents.truncate(5);
        c.feedback.retain(|f| f.after_episode < 4);
        for (episode, incident) in c.incidents.iter_mut().take(4).enumerate() {
            let actor = incident.game.actor();
            for source in &mut incident.artifacts {
                let accurate = source.descriptor.id == 0;
                let accurate = if corrupt_feedback {
                    accurate
                } else {
                    accurate ^ (variant == 1)
                };
                source.candidate = Some(if accurate { actor } else { (actor + 1) % 6 });
            }
            if corrupt_feedback && variant == 1 {
                c.feedback[episode].token.kind = e7::TokenKind::Outcome((actor + 1) % 6);
            }
        }
        for (id, process) in c.processes.iter_mut().enumerate() {
            process.accuracy_before = if (id == 0) ^ (!corrupt_feedback && variant == 1) {
                4
            } else {
                0
            };
            process.accuracy_after = 2;
            process.shift_episode = Some(4);
        }
        c
    })
}
