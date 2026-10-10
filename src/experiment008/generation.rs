use super::*;
use crate::{Rng, experiment005 as e5, experiment006 as e6};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diagnostic {
    Baseline,
    Accurate,
    Misleading,
    Missing,
    Conflict,
    TrustedWrong,
    UnreliableCorrect,
    KnownDependent,
    HiddenDependent,
    Exhaustion,
    Conservation,
    DifferentObservers,
    PrivateFinding,
    ChangedBehavior,
    Permuted,
    Injected,
}
pub const DIAGNOSTICS: [Diagnostic; 16] = [
    Diagnostic::Baseline,
    Diagnostic::Accurate,
    Diagnostic::Misleading,
    Diagnostic::Missing,
    Diagnostic::Conflict,
    Diagnostic::TrustedWrong,
    Diagnostic::UnreliableCorrect,
    Diagnostic::KnownDependent,
    Diagnostic::HiddenDependent,
    Diagnostic::Exhaustion,
    Diagnostic::Conservation,
    Diagnostic::DifferentObservers,
    Diagnostic::PrivateFinding,
    Diagnostic::ChangedBehavior,
    Diagnostic::Permuted,
    Diagnostic::Injected,
];
fn physical(candidate: Option<u8>, inspectors: Vec<u8>) -> Artifact {
    Artifact {
        descriptor: Descriptor {
            id: 0,
            description: "sealed scene trace".into(),
            visible_parent: None,
        },
        inspectors,
        candidate,
        actual_root: Root::Artifact(0),
    }
}
fn derived(candidate: Option<u8>, owner: u8, visible: bool, inspectors: Vec<u8>) -> Artifact {
    Artifact {
        descriptor: Descriptor {
            id: 1,
            description: "archived observation record".into(),
            visible_parent: visible.then_some(owner),
        },
        inspectors,
        candidate,
        actual_root: Root::Observation(owner),
    }
}
fn config(seed: u64, incidents: Vec<Incident>, kind: PolicyKind, budget: u8) -> Config {
    Config {
        version: "series-008-v1".into(),
        session: seed,
        incidents,
        policy: [kind; 6],
        budgets: [budget; 6],
        action_cost: 1,
        listener: e6::ListenerKind::Credibility,
        feedback: true,
    }
}
pub fn diagnostic_config(family: Diagnostic, seed: u64, kind: PolicyKind) -> Config {
    let mut incidents = vec![];
    for episode in 0..3 {
        let mut game = e5::Config::new(seed * 4 + episode);
        let actor = game.actor();
        let wrong = (actor + 1) % 6;
        game.observations[0] = Some(actor);
        game.observations[1] = Some(wrong);
        if matches!(
            family,
            Diagnostic::TrustedWrong | Diagnostic::UnreliableCorrect | Diagnostic::ChangedBehavior
        ) {
            let correct = if family == Diagnostic::UnreliableCorrect {
                episode == 2
            } else {
                episode < 2
            };
            game.observations[0] = Some(if correct { actor } else { wrong });
            game.observations[1] = None;
        }
        if matches!(family, Diagnostic::Exhaustion | Diagnostic::Conservation) && episode == 0 {
            game.observations = [Some(actor); 6];
        }
        let mut artifacts = vec![physical(Some(actor), (0..6).collect())];
        match family {
            Diagnostic::Baseline => artifacts.clear(),
            Diagnostic::Misleading => artifacts[0].candidate = Some(wrong),
            Diagnostic::Missing => artifacts[0].candidate = None,
            Diagnostic::KnownDependent | Diagnostic::HiddenDependent => {
                artifacts = vec![derived(
                    game.observations[0],
                    0,
                    family == Diagnostic::KnownDependent,
                    (0..6).collect(),
                )]
            }
            Diagnostic::DifferentObservers => artifacts[0].inspectors = vec![2, 4],
            Diagnostic::Permuted => {
                game.order = [3, 0, 5, 1, 4, 2];
                game.observations.rotate_left(2);
            }
            _ => {}
        }
        let communication = if family == Diagnostic::PrivateFinding {
            [CommunicationKind::Silent; 6]
        } else {
            [CommunicationKind::ShareFinding; 6]
        };
        let injected = if family == Diagnostic::Injected {
            (0..6).map(|id| (id, 0)).collect()
        } else {
            vec![]
        };
        incidents.push(Incident {
            game,
            artifacts,
            access_order: if family == Diagnostic::Permuted {
                [5, 3, 1, 4, 2, 0]
            } else {
                [0, 1, 2, 3, 4, 5]
            },
            communication,
            injected,
        });
    }
    config(
        seed,
        incidents,
        if family == Diagnostic::Injected {
            PolicyKind::NeverInvestigate
        } else {
            kind
        },
        if matches!(family, Diagnostic::Conservation | Diagnostic::Exhaustion) {
            1
        } else {
            2
        },
    )
}
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
fn shuffle(rng: &mut Rng) -> [u8; 6] {
    let mut a = [0, 1, 2, 3, 4, 5];
    for i in (1..6).rev() {
        a.swap(i, (rng.draw() % (i as u64 + 1)) as usize);
    }
    a
}
fn wrong(rng: &mut Rng, actor: u8) -> u8 {
    (actor + 1 + (rng.draw() % 5) as u8) % 6
}
impl WorldGenerator for Generator {
    fn generate(&mut self, seed: u64) -> World {
        let mut rng = Rng::new(seed);
        let identities = shuffle(&mut rng);
        let order = shuffle(&mut rng);
        let access_order = shuffle(&mut rng);
        let budget = 1 + (rng.draw() % 2) as u8;
        let private = rng.draw() % 2 == 1;
        let drift = rng.draw() % 2 == 1;
        let mut incidents = vec![];
        for episode in 0..3 {
            let mut game = e5::Config::new(rng.draw());
            let actor = game.actor();
            game.order = order;
            for &id in &identities[..2] {
                let threshold = if episode == 2 && drift { 1 } else { 3 };
                game.observations[usize::from(id)] = Some(if rng.draw() % 4 < threshold {
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
            let candidate = if rng.draw().is_multiple_of(4) {
                None
            } else {
                Some(if rng.draw() % 4 < 3 {
                    actor
                } else {
                    wrong(&mut rng, actor)
                })
            };
            let visible = rng.draw() % 2 == 1;
            let mut inspectors: Vec<_> = (0..6).filter(|_| rng.draw() % 2 == 1).collect();
            if !inspectors.iter().any(|id| identities[2..].contains(id)) {
                inspectors.push(identities[2]);
                inspectors.sort();
                inspectors.dedup();
            }
            let artifacts = vec![
                physical(candidate, inspectors.clone()),
                derived(
                    game.observations[usize::from(identities[0])],
                    identities[0],
                    visible,
                    inspectors,
                ),
            ];
            incidents.push(Incident {
                game,
                artifacts,
                access_order,
                communication: if private {
                    [CommunicationKind::Silent; 6]
                } else {
                    [CommunicationKind::ShareFinding; 6]
                },
                injected: vec![],
            });
        }
        World {
            seed,
            identities,
            base: config(seed, incidents, PolicyKind::NeverInvestigate, budget),
        }
    }
}
pub fn generated_config(world: &World, kind: PolicyKind, injected: bool) -> Config {
    let mut c = world.base.clone();
    c.policy = [if injected {
        PolicyKind::NeverInvestigate
    } else {
        kind
    }; 6];
    if injected {
        for incident in &mut c.incidents {
            incident.injected = incident.artifacts[0]
                .inspectors
                .iter()
                .map(|id| (*id, 0))
                .collect();
        }
    }
    c
}
