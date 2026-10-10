use super::*;
use crate::{Rng, experiment005 as e5, experiment006 as e6, experiment008 as e8};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diagnostic {
    EqualSources,
    CostDifference,
    QualityVsCost,
    MisleadingExpensive,
    MissingExpensive,
    KnownDuplicate,
    HiddenDuplicate,
    CompetingHypotheses,
    StrongInitialBelief,
    ResourceScarcity,
    CostReversal,
    SourceOrderPermutation,
    SourceIdentityPermutation,
    IncorrectPublicCharacteristics,
    SelectedVsInjected,
    ChangedSourceRegime,
}
pub const DIAGNOSTICS: [Diagnostic; 16] = [
    Diagnostic::EqualSources,
    Diagnostic::CostDifference,
    Diagnostic::QualityVsCost,
    Diagnostic::MisleadingExpensive,
    Diagnostic::MissingExpensive,
    Diagnostic::KnownDuplicate,
    Diagnostic::HiddenDuplicate,
    Diagnostic::CompetingHypotheses,
    Diagnostic::StrongInitialBelief,
    Diagnostic::ResourceScarcity,
    Diagnostic::CostReversal,
    Diagnostic::SourceOrderPermutation,
    Diagnostic::SourceIdentityPermutation,
    Diagnostic::IncorrectPublicCharacteristics,
    Diagnostic::SelectedVsInjected,
    Diagnostic::ChangedSourceRegime,
];
pub fn source(id: u8, candidate: Option<u8>, cost: u8) -> Source {
    Source {
        descriptor: e8::Descriptor {
            id,
            description: "declared instrument method; fallible result".into(),
            visible_parent: None,
        },
        method_key: format!("method-{id}"),
        cost,
        category: if cost > 1 {
            Category::Specialized
        } else {
            Category::Partial
        },
        declared_accuracy: if cost > 1 { 90 } else { 50 },
        declared_missing: if cost > 1 { 0 } else { 20 },
        coverage: (0..6).collect(),
        inspectors: (0..6).collect(),
        separate_acquisition_disclosed: id.is_multiple_of(2),
        candidate,
        actual_root: Root::Artifact(id),
    }
}
fn config(seed: u64, incidents: Vec<Incident>, kind: PolicyKind, budgets: [u8; 6]) -> Config {
    Config {
        version: "series-009-v1".into(),
        session: seed,
        incidents,
        policy: [kind; 6],
        budgets,
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
        let mut cheap = source(0, Some(actor), 1);
        cheap.coverage = vec![actor];
        let mut expensive = source(1, Some(actor), 2);
        expensive.coverage = vec![actor, wrong];
        let mut communication = [CommunicationKind::ShareFinding; 6];
        match family {
            Diagnostic::EqualSources | Diagnostic::CostDifference => {
                cheap.coverage = (0..6).collect();
                expensive.coverage = cheap.coverage.clone();
                expensive.declared_accuracy = cheap.declared_accuracy;
                expensive.declared_missing = cheap.declared_missing;
                expensive.category = cheap.category;
                expensive.separate_acquisition_disclosed = cheap.separate_acquisition_disclosed;
                if family == Diagnostic::EqualSources {
                    expensive.cost = 1;
                }
            }
            Diagnostic::MisleadingExpensive | Diagnostic::IncorrectPublicCharacteristics => {
                expensive.candidate = Some(wrong)
            }
            Diagnostic::MissingExpensive => expensive.candidate = None,
            Diagnostic::KnownDuplicate | Diagnostic::HiddenDuplicate => {
                expensive.category = Category::Derived;
                expensive.separate_acquisition_disclosed = false;
                expensive.actual_root = Root::Observation(0);
                expensive.candidate = game.observations[0];
                expensive.descriptor.visible_parent =
                    (family == Diagnostic::KnownDuplicate).then_some(0);
            }
            Diagnostic::CompetingHypotheses => {
                cheap.coverage = vec![(wrong + 1) % 6];
                expensive.coverage = vec![actor, wrong];
            }
            Diagnostic::StrongInitialBelief => {
                game.observations = [Some(actor); 6];
                communication = [CommunicationKind::Silent; 6];
            }
            Diagnostic::ResourceScarcity if episode == 0 => {
                game.observations = [Some(actor); 6];
                communication = [CommunicationKind::Silent; 6];
            }
            Diagnostic::CostReversal => {
                cheap.cost = 2;
                expensive.cost = 1;
            }
            Diagnostic::ChangedSourceRegime => {
                game.observations[0] = Some(if episode < 2 { actor } else { wrong });
                game.observations[1] = None;
                cheap.candidate = game.observations[0];
                expensive.candidate = game.observations[0];
                if episode < 2 {
                    cheap.inspectors.clear();
                    expensive.inspectors.clear();
                }
                communication = [CommunicationKind::Sincere; 6];
            }
            _ => {}
        }
        let mut artifacts = vec![cheap, expensive];
        if family == Diagnostic::SourceOrderPermutation {
            artifacts.reverse();
        }
        if family == Diagnostic::SourceIdentityPermutation {
            // Same semantic methods; bijective identity relabeling changes neither generation nor quality.
            let map = |id: u8| (id + 3) % 6;
            game.seed = (0..u64::MAX)
                .find(|seed| e5::Config::new(*seed).actor() == map(actor))
                .unwrap();
            game.observations = std::array::from_fn(|id| game.observations[(id + 3) % 6].map(map));
            game.order = [3, 4, 5, 0, 1, 2];
            for a in &mut artifacts {
                a.descriptor.id += 10;
                a.actual_root = Root::Artifact(a.descriptor.id);
                a.candidate = a.candidate.map(map);
                a.coverage = a.coverage.iter().copied().map(map).collect();
                a.inspectors = a.inspectors.iter().copied().map(map).collect();
            }
        }
        incidents.push(Incident {
            game,
            artifacts,
            access_order: [0, 1, 2, 3, 4, 5],
            communication,
            injected: vec![],
            forced: vec![],
        });
    }
    config(
        seed,
        incidents,
        kind,
        [if family == Diagnostic::ResourceScarcity {
            2
        } else {
            3
        }; 6],
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
        let budgets = std::array::from_fn(|_| 1 + (rng.draw() % 6) as u8);
        let mode = [
            CommunicationKind::Silent,
            CommunicationKind::ShareFinding,
            CommunicationKind::Sincere,
        ][(rng.draw() % 3) as usize];
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
            let count = 2 + (rng.draw() % 3) as u8;
            let mut artifacts = vec![];
            for id in 0..count {
                let cost = 1 + (rng.draw() % 3) as u8;
                let mut a = source(id, None, cost);
                a.category = match id {
                    0 => Category::Partial,
                    1 => Category::Physical,
                    2 => Category::Specialized,
                    _ => Category::Physical,
                };
                a.declared_accuracy = 25 + 25 * (rng.draw() % 4) as u8;
                a.declared_missing = 25 * (rng.draw() % 3) as u8;
                a.coverage = (0..6).filter(|_| rng.draw() % 2 == 1).collect();
                if a.coverage.is_empty() {
                    a.coverage.push((rng.draw() % 6) as u8);
                }
                a.inspectors = (0..6).filter(|_| rng.draw() % 2 == 1).collect();
                if a.inspectors.is_empty() {
                    a.inspectors.push(identities[2]);
                }
                if id == count - 1 && rng.draw() % 2 == 1 {
                    let owner = identities[0];
                    a.category = Category::Derived;
                    a.separate_acquisition_disclosed = false;
                    a.actual_root = Root::Observation(owner);
                    a.candidate = game.observations[usize::from(owner)];
                    a.descriptor.visible_parent = (rng.draw() % 2 == 1).then_some(owner);
                } else {
                    let threshold = if rng.draw() % 2 == 1 { 3 } else { 1 };
                    let threshold = if episode == 2 && drift {
                        4 - threshold
                    } else {
                        threshold
                    };
                    a.candidate = if rng.draw().is_multiple_of(4) {
                        None
                    } else {
                        Some(if rng.draw() % 4 < threshold {
                            actor
                        } else {
                            wrong(&mut rng, actor)
                        })
                    };
                }
                artifacts.push(a);
            }
            for i in (1..artifacts.len()).rev() {
                artifacts.swap(i, (rng.draw() % (i as u64 + 1)) as usize);
            }
            incidents.push(Incident {
                game,
                artifacts,
                access_order,
                communication: [mode; 6],
                injected: vec![],
                forced: vec![],
            });
        }
        World {
            seed,
            identities,
            base: config(seed, incidents, PolicyKind::NeverInvestigate, budgets),
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
        for i in &mut c.incidents {
            let a = &i.artifacts[0];
            i.injected = a
                .inspectors
                .iter()
                .map(|id| (*id, a.descriptor.id))
                .collect();
        }
    }
    c
}
