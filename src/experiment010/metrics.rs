use super::*;
use crate::{Phase, experiment005 as e5, experiment009 as e9};
use e9::{Category, Metrics};
pub fn incident_metrics(r: &Report, assessment: bool) -> Result<Metrics, String> {
    let mut m = Metrics {
        spent: r
            .episodes
            .iter()
            .filter(|e| !assessment || e.id >= 4)
            .flat_map(|e| &e.deliveries)
            .map(|d| u64::from(d.balance_before - d.balance_after))
            .sum(),
        conserved: r.ledgers.iter().map(|l| u64::from(l.remaining)).sum(),
        full_report_bytes: serde_json::to_vec(r).map_err(|e| e.to_string())?.len() as u64,
        final_memory_bytes: serde_json::to_vec(&r.final_memory)
            .map_err(|e| e.to_string())?
            .len() as u64,
        ..Metrics::default()
    };
    for e in r.episodes.iter().filter(|e| !assessment || e.id >= 4) {
        m.communication_rejections += e
            .incident
            .decisions
            .iter()
            .filter(|d| d.input.phase == Phase::Discussion && d.rejection.is_some())
            .count() as u64;
        m.opportunities += e.investigation.len() as u64;
        for d in &e.investigation {
            m.inspections += d.assessment.inspections as u64;
            m.accessible_options += d.input.local.catalog.len() as u64;
            let affordable = d
                .input
                .local
                .catalog
                .iter()
                .filter(|s| s.cost <= d.input.local.balance)
                .count();
            m.affordable_options += affordable as u64;
            m.multiple_legal += u64::from(affordable >= 2);
            m.forced_actions += u64::from(d.forced);
            m.hypothetical_assessments += d
                .choice
                .considered
                .iter()
                .map(|e| e.outcomes.len() as u64)
                .sum::<u64>();
            m.known_overlap_options += d
                .choice
                .considered
                .iter()
                .filter(|e| e.known_overlap)
                .count() as u64;
            let initial = r.ledgers[usize::from(d.input.local.id)].initial;
            m.blocked_by_prior_spending += u64::from(
                affordable == 0 && d.input.local.catalog.iter().any(|s| s.cost <= initial),
            );
            if let Some(di) = d.delivery {
                let delivery = &e.deliveries[di];
                let estimate = d
                    .choice
                    .considered
                    .iter()
                    .find(|x| x.source == delivery.finding.source.id)
                    .ok_or("selected source lacks expected-value audit")?;
                let changed = delivery.before_assessment.after.candidate
                    != delivery.after_assessment.after.candidate;
                if !d.forced {
                    let source = d
                        .input
                        .local
                        .catalog
                        .iter()
                        .find(|s| s.descriptor.id == estimate.source)
                        .unwrap();
                    let category = match source.category {
                        Category::Partial => 0,
                        Category::Physical => 1,
                        Category::Specialized => 2,
                        Category::Derived => 3,
                    };
                    m.selected_categories[category] += 1;
                    m.selected_costs[usize::from(source.cost - 1)] += 1;
                }
                m.selected_expected_usefulness += u64::from(estimate.usefulness);
                m.forecast_positive_but_no_change += u64::from(estimate.usefulness > 0 && !changed);
                m.forecast_zero_but_change += u64::from(estimate.usefulness == 0 && changed);
                m.selected_local_changes += u64::from(changed);
                if changed {
                    m.units_for_changed_findings +=
                        u64::from(delivery.balance_before - delivery.balance_after);
                }
            }
            m.eligible += u64::from(
                d.input
                    .local
                    .catalog
                    .iter()
                    .any(|s| s.cost <= d.input.local.balance),
            );
            m.rejected += u64::from(d.rejection.is_some());
            m.selected += u64::from(
                d.rejection.is_none()
                    && !d.forced
                    && matches!(d.choice.action, InvestigationAction::Inspect(_)),
            );
            m.declined += u64::from(
                d.rejection.is_none()
                    && !d.forced
                    && d.choice.action == InvestigationAction::Decline,
            );
        }
        for d in &e.deliveries {
            m.inspections +=
                (d.before_assessment.inspections + d.after_assessment.inspections) as u64;
            if matches!(d.origin, DeliveryOrigin::Intervention(_)) {
                m.injected_findings += 1;
            }
            match d.correct {
                Some(true) => m.correct_findings += 1,
                Some(false) => m.wrong_findings += 1,
                None => m.missing_findings += 1,
            }
            let before = &d.before_assessment.after;
            let after = &d.after_assessment.after;
            m.candidate_changes += u64::from(before.candidate != after.candidate);
            m.support_changes += u64::from(before.scores != after.scores);
            let actor = e.incident.config.actor();
            let diff = i8::from(after.candidate == Some(actor))
                - i8::from(before.candidate == Some(actor));
            if diff > 0 {
                m.acquisition_improved += 1;
            } else if diff < 0 {
                m.acquisition_worsened += 1;
            } else {
                m.acquisition_null += 1;
            }
            m.unnecessary +=
                u64::from(before == after && matches!(d.origin, DeliveryOrigin::Selected(_)));
            m.useful_private += u64::from(
                before != after
                    && !e.communication.iter().any(|c| {
                        c.input.incident.id == d.finding.acquisition.observer
                            && c.decision >= 6
                            && c.decision < 12
                            && e.incident.decisions[c.decision].rejection.is_none()
                            && matches!(c.choice.choice.action, e5::Action::Speak { .. })
                    })
                    && !e
                        .reports
                        .iter()
                        .any(|p| p.finding.acquisition == d.finding.acquisition),
            );
            m.known_dependent += u64::from(
                matches!(d.actual_root, Root::Observation(_))
                    && d.finding.source.visible_parent.is_some(),
            );
            m.hidden_dependent += u64::from(
                matches!(d.actual_root, Root::Observation(_))
                    && d.finding.source.visible_parent.is_none(),
            );
            m.ignored_findings +=
                u64::from(d.finding.candidate.is_none() || before.scores == after.scores);
        }
        for c in &e.cognition {
            m.inspections += c.assessment.inspections as u64;
            m.used_groups += c
                .assessment
                .contributions
                .iter()
                .filter(|g| g.weight > 0)
                .count() as u64;
            m.contested_groups += c
                .assessment
                .contributions
                .iter()
                .filter(|g| g.contested)
                .count() as u64;
            m.duplicate_exposures += c
                .assessment
                .contributions
                .iter()
                .map(|g| g.observations.saturating_sub(1) as u64)
                .sum::<u64>();
            m.max_findings = m.max_findings.max(c.input.findings.len() as u64);
            m.max_messages = m.max_messages.max(c.input.incident.received.len() as u64);
        }
        for d in &e.incident.decisions {
            if d.input.phase == Phase::Judgment {
                match d.choice.action {
                    e5::Action::Vote(Some(c)) if c == e.incident.config.actor() => {
                        m.correct_ballots += 1
                    }
                    e5::Action::Vote(Some(_)) => m.wrong_ballots += 1,
                    _ => m.abstentions += 1,
                }
            }
        }
        match e.incident.correct {
            Some(true) => m.correct_verdicts += 1,
            Some(false) => m.wrong_verdicts += 1,
            None => m.unresolved += 1,
        }
        m.max_records = m.max_records.max(
            e.memory_after
                .iter()
                .map(|x| x.local.records.len() as u64)
                .max()
                .unwrap_or(0),
        );
    }
    Ok(m)
}
