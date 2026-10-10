use super::*;
use crate::{Rng, experiment004 as e4, experiment005 as e5, experiment006 as e6};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fault {
    Clean,
    AuthenticatedFalse,
    UnauthenticatedFalse,
    Conflict,
    Missing,
    Selective,
}
pub const FAULTS: [Fault; 6] = [
    Fault::Clean,
    Fault::AuthenticatedFalse,
    Fault::UnauthenticatedFalse,
    Fault::Conflict,
    Fault::Missing,
    Fault::Selective,
];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    pub sources: Vec<u8>,
    pub target: u8,
    pub changed: bool,
    pub corrections: bool,
    pub thresholds: Vec<u8>,
    pub incidents: Vec<e5::Config>,
}
pub trait WorldGenerator {
    fn generate(&mut self, seed: u64) -> World;
}
pub struct Generator;
fn shuffle(rng: &mut Rng) -> [u8; 6] {
    let mut ids = [0, 1, 2, 3, 4, 5];
    for i in (1..6).rev() {
        let j = (rng.draw() % (i as u64 + 1)) as usize;
        ids.swap(i, j);
    }
    ids
}
fn observed(rng: &mut Rng, actor: u8, threshold: u8) -> u8 {
    if rng.draw() % 4 < u64::from(threshold) {
        actor
    } else {
        (actor + 1 + (rng.draw() % 5) as u8) % 6
    }
}
fn truthful() -> e5::SpeakerKind {
    e5::SpeakerKind::Retain(e4::SpeakerKind::baseline(e5::PolicyKind::Truthful))
}
impl WorldGenerator for Generator {
    fn generate(&mut self, seed: u64) -> World {
        let mut rng = Rng::new(seed);
        let identities = shuffle(&mut rng);
        let order = shuffle(&mut rng);
        let source_count = (rng.draw() % 3 + 1) as usize;
        let histories = (rng.draw() % 5) as usize;
        let changed = rng.draw() % 2 == 1;
        let sources = identities[..source_count].to_vec();
        let target = identities[5];
        let thresholds: Vec<u8> = sources
            .iter()
            .map(|_| if rng.draw().is_multiple_of(2) { 1 } else { 3 })
            .collect();
        let corrections = rng.draw() % 2 == 1;
        let nonsources = identities[source_count..].to_vec();
        let mut incidents = vec![];
        for episode in 0..=histories {
            let mut c = e5::Config::new(rng.draw());
            c.order = order;
            let actor = c.actor();
            for (&source, &threshold) in sources.iter().zip(&thresholds) {
                let id = usize::from(source);
                c.routes[id] = nonsources.clone();
                c.second_routes[id] = nonsources.clone();
                c.speakers[id] = if episode < histories && corrections {
                    e5::SpeakerKind::Sincere
                } else {
                    truthful()
                };
                c.observations[id] = if episode < histories && rng.draw().is_multiple_of(4) {
                    None
                } else {
                    Some(observed(
                        &mut rng,
                        actor,
                        if episode == histories && changed {
                            4 - threshold
                        } else {
                            threshold
                        },
                    ))
                };
                if episode < histories && corrections {
                    c.later.push(e5::LaterObservation {
                        before_slot: 6,
                        agent: source,
                        candidate: Some(actor),
                    });
                }
            }
            if episode == histories {
                for &id in &nonsources {
                    if id != target && rng.draw().is_multiple_of(4) {
                        c.observations[usize::from(id)] = Some(if rng.draw().is_multiple_of(2) {
                            actor
                        } else {
                            (actor + 1 + (rng.draw() % 5) as u8) % 6
                        });
                    }
                }
            }
            incidents.push(c);
        }
        World {
            seed,
            sources,
            target,
            changed,
            corrections,
            thresholds,
            incidents,
        }
    }
}
pub fn generated_config(
    world: &World,
    fault: Fault,
    acceptance: VerificationKind,
    listener: e6::ListenerKind,
) -> Config {
    let mut c = Config::new(
        world.seed,
        world.incidents.clone(),
        world.sources.clone(),
        world.target,
    );
    c.acceptance = acceptance;
    c.listener = listener;
    let recipient = (0..6).find(|id| !world.sources.contains(id)).unwrap();
    for episode in 0..c.incidents.len() - 1 {
        if fault == Fault::Missing {
            continue;
        }
        let actor = c.incidents[episode].actor();
        let mut token = Token::outcome(
            c.session,
            episode as u8,
            0,
            if matches!(
                fault,
                Fault::AuthenticatedFalse | Fault::UnauthenticatedFalse
            ) {
                (actor + 1) % 6
            } else {
                actor
            },
        );
        token.authenticated_channel = fault != Fault::UnauthenticatedFalse;
        let recipients = if fault == Fault::Selective {
            vec![recipient]
        } else {
            (0..6).collect()
        };
        c.feedback.push(Plan {
            after_episode: episode as u8,
            actual_about: episode as u8,
            source: "declared laboratory issuer intervention".into(),
            fault: format!("{fault:?}"),
            recipients: recipients.clone(),
            token,
        });
        if fault == Fault::Conflict {
            c.feedback.push(Plan {
                after_episode: episode as u8,
                actual_about: episode as u8,
                source: "declared contradictory issuance".into(),
                fault: "Conflict".into(),
                recipients,
                token: Token::outcome(c.session, episode as u8, 1, (actor + 1) % 6),
            });
        }
    }
    c
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Diagnostic {
    Accurate,
    RewardFalse,
    PenalizeTrue,
    Unauthenticated,
    ConflictTrueFirst,
    ConflictFalseFirst,
    DelayedContradiction,
    Missing,
    Selective,
    Duplicate,
    Repeated,
    ReplayWrongSession,
    UnknownEpisode,
    Misassociated,
    EvictedConflict,
    Verdict,
}
pub const DIAGNOSTICS: [Diagnostic; 16] = [
    Diagnostic::Accurate,
    Diagnostic::RewardFalse,
    Diagnostic::PenalizeTrue,
    Diagnostic::Unauthenticated,
    Diagnostic::ConflictTrueFirst,
    Diagnostic::ConflictFalseFirst,
    Diagnostic::DelayedContradiction,
    Diagnostic::Missing,
    Diagnostic::Selective,
    Diagnostic::Duplicate,
    Diagnostic::Repeated,
    Diagnostic::ReplayWrongSession,
    Diagnostic::UnknownEpisode,
    Diagnostic::Misassociated,
    Diagnostic::EvictedConflict,
    Diagnostic::Verdict,
];
pub fn diagnostic_config(
    d: Diagnostic,
    seed: u64,
    acceptance: VerificationKind,
    listener: e6::ListenerKind,
) -> Config {
    let old = e6::fixture_config(
        match d {
            Diagnostic::RewardFalse | Diagnostic::DelayedContradiction => e6::Fixture::Inaccurate,
            Diagnostic::EvictedConflict => e6::Fixture::Eviction,
            Diagnostic::PenalizeTrue
            | Diagnostic::Unauthenticated
            | Diagnostic::ConflictTrueFirst
            | Diagnostic::ConflictFalseFirst
            | Diagnostic::Duplicate
            | Diagnostic::Repeated
            | Diagnostic::ReplayWrongSession
            | Diagnostic::UnknownEpisode
            | Diagnostic::Verdict => e6::Fixture::Corrected,
            _ => e6::Fixture::Accurate,
        },
        seed,
        e6::ListenerKind::Credibility,
    );
    let mut incidents = old.incidents;
    // These families use one ordinary, uncorrected history; corrections remain generator variation.
    if matches!(
        d,
        Diagnostic::PenalizeTrue
            | Diagnostic::Unauthenticated
            | Diagnostic::ConflictTrueFirst
            | Diagnostic::ConflictFalseFirst
            | Diagnostic::Duplicate
            | Diagnostic::Repeated
            | Diagnostic::ReplayWrongSession
            | Diagnostic::UnknownEpisode
            | Diagnostic::Verdict
    ) {
        let h = &mut incidents[0];
        h.observations[0] = Some(h.actor());
        h.later.clear();
        h.speakers[0] = truthful();
    }
    if matches!(
        d,
        Diagnostic::RewardFalse | Diagnostic::DelayedContradiction
    ) {
        let current = incidents.last_mut().unwrap();
        current.observations[0] = Some((current.actor() + 1) % 6);
        current.observations[2] = Some(current.actor());
    }
    if d == Diagnostic::DelayedContradiction {
        incidents.push(incidents.last().unwrap().clone());
    }
    let mut c = Config::new(seed, incidents, vec![0, 2], 5);
    c.acceptance = acceptance;
    c.listener = listener;
    let histories = if d == Diagnostic::DelayedContradiction {
        1
    } else {
        c.incidents.len() - 1
    };
    for episode in 0..histories {
        if d == Diagnostic::Missing {
            continue;
        }
        let actor = c.incidents[episode].actor();
        let wrong = matches!(
            d,
            Diagnostic::RewardFalse
                | Diagnostic::PenalizeTrue
                | Diagnostic::DelayedContradiction
                | Diagnostic::ConflictFalseFirst
        ) || (d == Diagnostic::EvictedConflict && episode == 0);
        let mut token = Token::outcome(
            seed,
            episode as u8,
            0,
            if wrong { (actor + 1) % 6 } else { actor },
        );
        if d == Diagnostic::Unauthenticated {
            token.authenticated_channel = false;
            token.kind = TokenKind::Outcome((actor + 1) % 6);
        }
        if d == Diagnostic::ReplayWrongSession {
            token.session = seed + 1;
        }
        if d == Diagnostic::UnknownEpisode {
            token.episode = 255;
            token.serial = 1020;
        }
        if d == Diagnostic::Misassociated && episode == 0 {
            token.episode = 1;
        }
        if d == Diagnostic::Verdict {
            token.kind = TokenKind::Verdict(Some((actor + 1) % 6));
        }
        c.feedback.push(Plan {
            after_episode: if d == Diagnostic::Misassociated {
                1
            } else {
                episode as u8
            },
            actual_about: episode as u8,
            source: "diagnostic controller issuance".into(),
            fault: format!("{d:?}"),
            recipients: if d == Diagnostic::Selective {
                vec![3]
            } else {
                (0..6).collect()
            },
            token: token.clone(),
        });
        if matches!(
            d,
            Diagnostic::Duplicate
                | Diagnostic::Repeated
                | Diagnostic::ConflictTrueFirst
                | Diagnostic::ConflictFalseFirst
        ) {
            if d != Diagnostic::Duplicate {
                token.serial += 1;
            }
            if matches!(
                d,
                Diagnostic::ConflictTrueFirst | Diagnostic::ConflictFalseFirst
            ) {
                token.kind = TokenKind::Outcome(if wrong { actor } else { (actor + 1) % 6 });
            }
            c.feedback.push(Plan {
                after_episode: episode as u8,
                actual_about: episode as u8,
                source: "diagnostic repeated/contradictory issuance".into(),
                fault: format!("{d:?}"),
                recipients: (0..6).collect(),
                token,
            });
        }
    }
    if matches!(
        d,
        Diagnostic::DelayedContradiction | Diagnostic::EvictedConflict
    ) {
        c.feedback.push(Plan {
            after_episode: if d == Diagnostic::DelayedContradiction {
                1
            } else {
                7
            },
            actual_about: 0,
            source: "delayed diagnostic contradiction".into(),
            fault: format!("{d:?}"),
            recipients: (0..6).collect(),
            token: Token::outcome(seed, 0, 1, c.incidents[0].actor()),
        });
    }
    c
}
