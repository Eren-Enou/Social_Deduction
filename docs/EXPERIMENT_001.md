# Experiment 001: The Incident

Preregistered implementation evaluation (before source implementation):

Hypothesis: all three baselines complete legal bounded games using only local inputs, with byte-identical replay under identical configuration. This is a software invariant hypothesis, not a behavioral intelligence hypothesis.

Run seeds 0–31 for all-silent, all-truthful, all-random, and mixed [truthful, truthful, random, silent, random, silent]. Record legal/rejected decisions, claims, verdicts, and correctness. Negative control: silent games must have no claims and no verdict. Incomplete-information control: truthful games should have only three claims and no verdict (two correct beliefs, one mistaken belief). Random outcomes have no success threshold. Run replay for every case.

Rules: incident → observation → two rounds of discussion → simultaneous judgment → terminal result. Six individuals, 12 discussion decisions, six ballot decisions; accepted claims persist. Four votes are necessary for a verdict. See DECISIONS.md for interventions and policy semantics.

Findings and frozen evidence are recorded after executing evaluations; do not infer findings from code alone.
