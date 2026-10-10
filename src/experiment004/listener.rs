use super::*;
use crate::{N, experiment003 as e3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListenerView {
    pub id: u8,
    pub original: IndividualState,
    pub prior: Belief,
    pub messages: Vec<Message>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Group {
    Speaker(u8),
    OpaqueMessage(usize),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verification {
    ClaimOnly,
    VisibleLink,
    MissingReference,
    InconsistentReference,
    DepthExceeded,
    MalformedReference,
    NotUsed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Treatment {
    Accepted,
    DuplicateDiscount,
    SelfIgnored,
    OwnOriginIgnored,
    UncertaintyIgnored,
    RepeatIgnored,
    UncertainProvenance,
    InvalidIgnored,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAssessment {
    pub message: usize,
    pub group: Option<Group>,
    pub treatment: Treatment,
    pub verification: Verification,
    pub origin_verified: bool,
    pub checked_messages: Vec<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assessment {
    pub after: Belief,
    pub sources: Vec<SourceAssessment>,
    pub perceived_groups: Vec<Group>,
    pub mechanism: String,
}
pub trait Listener {
    fn assess(&mut self, view: &ListenerView) -> Assessment;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListenerKind {
    SourceCount,
    Provenance,
}
pub fn resolve_source(
    message: &Message,
    received: &[Message],
) -> (Option<Group>, Verification, Vec<usize>) {
    let mut current = message;
    let mut checked = vec![];
    let mut links = 0;
    loop {
        if checked.contains(&current.id) || current.speaker >= N {
            return (None, Verification::MalformedReference, checked);
        }
        checked.push(current.id);
        match current.attribution {
            Attribution::Firsthand | Attribution::Unspecified => {
                return (
                    Some(Group::Speaker(current.speaker)),
                    if links > 0 {
                        Verification::VisibleLink
                    } else {
                        Verification::ClaimOnly
                    },
                    checked,
                );
            }
            Attribution::NamedSource(id) => {
                return (
                    if id < N {
                        Some(Group::Speaker(id))
                    } else {
                        None
                    },
                    if links > 0 {
                        Verification::VisibleLink
                    } else {
                        Verification::ClaimOnly
                    },
                    checked,
                );
            }
            Attribution::Reference(parent) => {
                if parent >= current.id {
                    return (None, Verification::MalformedReference, checked);
                }
                links += 1;
                if links > MAX_DEPTH {
                    return (None, Verification::DepthExceeded, checked);
                }
                let matches: Vec<_> = received.iter().filter(|m| m.id == parent).collect();
                if matches.len() > 1 {
                    return (None, Verification::MalformedReference, checked);
                }
                let Some(next) = matches.first() else {
                    return (
                        Some(Group::OpaqueMessage(parent)),
                        Verification::MissingReference,
                        checked,
                    );
                };
                if current.speech != next.speech {
                    return (None, Verification::InconsistentReference, checked);
                }
                current = next;
            }
        }
    }
}
impl Listener for ListenerKind {
    fn assess(&mut self, view: &ListenerView) -> Assessment {
        if *self == Self::SourceCount {
            let baseline =
                e3::Listener::assess(&mut e3::ListenerKind::Evidence, &source_count_view(view));
            let sources: Vec<_> = baseline
                .messages
                .iter()
                .map(|a| {
                    let message = view.messages.iter().find(|m| m.id == a.message_id).unwrap();
                    let treatment = match a.handling {
                        e3::Handling::SelfIgnored => Treatment::SelfIgnored,
                        e3::Handling::UncertaintyIgnored => Treatment::UncertaintyIgnored,
                        e3::Handling::RepeatIgnored => Treatment::RepeatIgnored,
                        e3::Handling::InvalidIgnored => Treatment::InvalidIgnored,
                        _ => Treatment::Accepted,
                    };
                    SourceAssessment {
                        message: a.message_id,
                        group: if treatment == Treatment::Accepted {
                            Some(Group::Speaker(message.speaker))
                        } else {
                            None
                        },
                        treatment,
                        verification: Verification::NotUsed,
                        origin_verified: false,
                        checked_messages: vec![],
                    }
                })
                .collect();
            let perceived_groups = sources.iter().filter_map(|s| s.group).collect();
            return Assessment {
                after: baseline.after,
                sources,
                perceived_groups,
                mechanism: "unchanged source-count-003-v1 with attribution stripped".into(),
            };
        }
        let mut scores = initial_belief(view.id, &view.original).scores;
        let mut seen = [false; 6];
        let mut groups = vec![];
        let mut sources = vec![];
        for message in &view.messages {
            let mut source = SourceAssessment {
                message: message.id,
                group: None,
                treatment: Treatment::InvalidIgnored,
                verification: Verification::NotUsed,
                origin_verified: false,
                checked_messages: vec![],
            };
            if message.speaker >= N {
                sources.push(source);
                continue;
            }
            if message.speaker == view.id {
                source.treatment = Treatment::SelfIgnored;
            } else {
                match message.speech {
                    Speech::Uncertain => source.treatment = Treatment::UncertaintyIgnored,
                    Speech::Assert(candidate) if candidate >= N => {}
                    Speech::Assert(_) if seen[usize::from(message.speaker)] => {
                        source.treatment = Treatment::RepeatIgnored
                    }
                    Speech::Assert(candidate) => {
                        seen[usize::from(message.speaker)] = true;
                        let (group, verification, checked) =
                            resolve_source(message, &view.messages);
                        source.group = group;
                        source.verification = verification;
                        source.checked_messages = checked;
                        source.treatment = match group {
                            None => Treatment::UncertainProvenance,
                            Some(Group::Speaker(id)) if id == view.id => {
                                Treatment::OwnOriginIgnored
                            }
                            Some(group) if groups.contains(&group) => Treatment::DuplicateDiscount,
                            Some(group) => {
                                groups.push(group);
                                scores[usize::from(candidate)] += 1;
                                Treatment::Accepted
                            }
                        };
                    }
                }
            }
            sources.push(source);
        }
        Assessment {after:Belief::from_scores(scores),sources,perceived_groups:groups,mechanism:"provenance-group-004-v1: first declared group only; origin claims remain unverified".into()}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input(messages: Vec<Message>) -> ListenerView {
        ListenerView {
            id: 5,
            original: IndividualState {
                observation: None,
                briefing: None,
                belief: None,
                incentives: Incentives::ACCURACY,
            },
            prior: Belief::default(),
            messages,
        }
    }
    #[test]
    fn copying_one_of_two_conflicting_chains_does_not_break_the_tie() {
        let view = input(vec![
            Message {
                id: 0,
                speaker: 0,
                speech: Speech::Assert(1),
                attribution: Attribution::Firsthand,
            },
            Message {
                id: 1,
                speaker: 1,
                speech: Speech::Assert(2),
                attribution: Attribution::Firsthand,
            },
            Message {
                id: 2,
                speaker: 2,
                speech: Speech::Assert(1),
                attribution: Attribution::Reference(0),
            },
        ]);
        let aware = ListenerKind::Provenance.assess(&view);
        assert_eq!(aware.after.candidate, None);
        assert_eq!(aware.after.scores[1], 1);
        assert_eq!(aware.after.scores[2], 1);
        assert_eq!(
            ListenerKind::SourceCount.assess(&view).after.candidate,
            Some(1)
        );
    }
    #[test]
    fn a_genuine_correction_after_malformed_first_assertion_remains_deferred() {
        let view = input(vec![
            Message {
                id: 0,
                speaker: 0,
                speech: Speech::Assert(1),
                attribution: Attribution::Firsthand,
            },
            Message {
                id: 1,
                speaker: 1,
                speech: Speech::Assert(2),
                attribution: Attribution::Reference(0),
            },
            Message {
                id: 2,
                speaker: 1,
                speech: Speech::Assert(2),
                attribution: Attribution::Firsthand,
            },
        ]);
        let result = ListenerKind::Provenance.assess(&view);
        assert_eq!(result.sources[1].treatment, Treatment::UncertainProvenance);
        assert_eq!(result.sources[2].treatment, Treatment::RepeatIgnored);
        assert_eq!(result.after.scores[2], 0);
        assert_eq!(result.after.candidate, Some(1));
    }
}
