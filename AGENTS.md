# SocialDeduction engineering and research instructions

This file is the first-stop instruction set for coding assistants working on this repository.

## Purpose and scope

Build a **controlled, original, game-like social-deduction laboratory** for studying how artificial individuals may choose to tell truths, make mistakes, deceive, withhold information, infer motives, and update relationships. The research target is **causal social behavior**, not beating a specific game's metagame, scripting dramatic moments, or demonstrating consciousness.

Read, in order:
- `docs/SIMULATION_CONSTITUTION.md`
- `docs/RESEARCH_OBJECTIVES.md`
- `docs/DECISIONS.md`
- `docs/ARCHITECTURE.md`
- the active experiment specification and `docs/EVALUATION_PROTOCOL.md`

`README.md` is the navigation entry point; design docs are proposals unless explicitly decided. Read actual source and tests when code exists rather than inferring behavior from docs.

## Non-negotiable research principles

Preserve all ten principles in `docs/SIMULATION_CONSTITUTION.md`: identifiable causality, information locality, persistent consequences, separation of individual perspective from objective events, resource consistency, reproducibility, no predetermined narrative outcomes, historical traceability, replaceable behavior policies, and measured complexity.

Additional domain-specific constraints:

- The **engine/observer** can possess ground truth; an **agent policy** cannot unless an explicit observation legally reveals it.
- Define a narrow, typed `AgentView`/observation interface. Never pass the entire game state to a policy and rely on discipline not to inspect secrets.
- Separate **world truth**, **an agent's private belief**, **what the agent communicates**, and **the agent's chosen communication intent or strategy**. These can disagree.
- A factually false statement is not automatically a lie. Confidence, mistake, bluff, concealment, and intentional contradiction are distinct categories.
- Agent decisions, including silence, information disclosure, accusation, lying, abstention, and failure, must not be forced merely to produce interesting research outcomes.
- The experiment controller can schedule opportunities, define legal roles and incentives, seed starting conditions, and impose declared interventions. Such controls must be logged and must not be mislabeled as agent initiative.
- A listener never receives speaker belief, private intent, actual role, hidden causal roots, or simulation-only ground truth through ordinary dialogue.
- Prior beliefs can be revised; historical claims and events are immutable. Do not back-edit a speaker's old mental state or statement because later evidence changes an interpretation.
- If memories are bounded, record evictions. Do not reconstruct forgotten cognition from an audit archive; separately retained consequences may legitimately persist.
- Legal actions, communication budgets, game transitions, tie handling, and termination require tests.

## Research integrity

- Before each new mechanism, specify hypothesis, outcomes, baselines, and negative controls.
- Use deterministic seeds, stable ordering, explicit rules/config versions, and reproducible reports.
- Capture policy inputs/selected actions/resolver effects at the time of decision; never explain past decisions by rerunning a changed policy.
- Keep observer/audit storage separate from agent-accessible storage.
- Preserve surprising, null, and negative results. Do not tune a policy on the evaluation set and claim independent validation.
- Distinguish *a behavior present because of a hard-coded rule* from *an observed consequence of broader mechanisms*.
- Preserve reproducible evidence before adding a more complex policy.
- Do not make unsupported claims about psychological realism, calibrated beliefs, emergent social intelligence, or generality.

## Implementation conventions

The proposed implementation is headless Rust. Bevy ECS is optional until measured need justifies it. Prefer clear domain types, simple replaceable policies, explicit transition validation, and deterministic event records over a large framework.

At the start of a coding task: inspect repository status and relevant files, define the requested change's boundary, and avoid overwriting unrelated work. After changes: run relevant tests and any established repository validation gate, review the diff, and update the experiment spec/findings and decision log when affected. Do not overwrite frozen evidence. Only commit/push when the task authorizes it.

Do **not** blindly copy Eldoria's frozen 001–012 experiment modes, data models, food economy, results, validation commands, or repository workflow. See `docs/ELDORIA_REFERENCE.md` for targeted reuse rationale.

## Implementation order

Build the minimal rules-only game and observation boundary first. Once they are validated, introduce voluntary communication; then compare truthful, mistaken, intentionally false, and strategically silent policies in controlled fixtures. Advanced memory, social prediction, reputation, coalitions, and visualizations follow evidence, not enthusiasm.

## Project boundary

This project is **not Eldoria canon** and does not simulate its fictional civilizations. Commercial social-deduction properties may inspire design, but do not embed protected role descriptions or branded material without permission.
