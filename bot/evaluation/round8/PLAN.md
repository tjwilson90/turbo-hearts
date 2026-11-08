# Capacity, history, coherent plans, and payoff training

Freeze round-seven production as `round7`. Keep all earlier profiles reproducible.
Use the existing training-only successful-human cohort and chronological split.
The later block was examined in earlier rounds: describe it as later-game
validation, never a pristine human holdout. No live-human gain is inferred from
simulated responders. No deployment is part of this task.

1. Export actor-only rich observations: cards, public behavior history, and known
   own passes. Fit small conditional neural move models (32/64 hidden units),
   comparing old-context and rich-context features. Validation selects checkpoint;
   test likelihood is reported afterward. Train with masked private pass features
   too, because simulated opponents' private passes are not known to the root.
2. Test coherent opponent continuations: draw a latent policy component once per
   rollout, retaining that plan rather than redrawing it at every simulated move.
   Condition its posterior on observed public actions and plausible sampled hands.
   Keep noise and abandon impossible shooting plans through existing policy rules.
3. Generate action comparisons using responsive mixed-opponent continuations on
   training-game positions. Train a regularized payoff/advantage head from those
   counterfactual targets, not from the human's eventual score as action credit.
4. Screen capacity/history, coherent plans, and payoff changes separately on
   validation starts against frozen direct-human and human/search hybrid policies.
   Use independent models as responders, not just each candidate's own policy.
5. Select a finalist before fresh-deal confirmation at seeds 1600000+; also check
   later human-shaped starts and the known Dan/ace/charged-ten/nine fixtures.
   Compare both fixed samples and equal decision-time budgets; inference speed
   and completed search work are part of the decision. Preserve the prior default
   unless the human-focused payoff evidence supports changing it.

After the 40-game screens, enlarge validation to 100 games for three finalists:
legacy-feature 32-unit neural policy, card-aware 32-unit neural policy, and
50% payoff blending. Compare each with actionclaim, actionhybrid, and an
independently fitted history-neural/search hybrid. Prefer aggregate improvement
across these human-focused responders, not a single best subgroup. The richer
history policy, 50% history mixture, and coherent-component model have not shown
consistent payoff gains and remain diagnostics. Check time budgets after the
fixed-sample batch finishes, so concurrent training does not determine results.

The two small-context networks reached their best validation loss near the
80-epoch ceiling. Extend those exact seeded fits to at most 200 epochs, retaining
the same early stopping and checkpoint rule, to distinguish a training ceiling
from convergence. Keep these checkpoints separate from the already selected
100-game match candidates; report the training ablation without quietly changing
a model midway through an evaluation.

Select the original 80-epoch capacity32 checkpoint for confirmation. On the
100-game checks it has the best mean across the three responders (+3.04 money,
descriptive only), versus +2.01 for the card-aware model. The payoff blend has
a clear direct-responder regression and poor Dan continuations; its last
neural-hybrid job is still finishing and cannot reverse that concern.
Confirm capacity32 on 500 fresh deals per responder (seed 1600000), plus
100 later historical games per responder (reused later block). Do not change
checkpoint or tune it on these results. Then compare at a 5 ms soft decision
budget with an 8192 rollout ceiling, sequentially with no concurrent research
jobs. Use 100 development deals starting at 1510000, followed by 500 fresh
deals starting at 1610000 per responder. A promotion requires robust payoff
evidence, including the decision-time comparison; one favorable responder is
insufficient.

Before the equal-time jobs, remove unused rich-input construction for 18-input
models and skip checkpoint output columns whose weights and biases are all
zero. No weights or nonzero accumulation orders change. Verify exact score,
moon, win-share, and search-count parity on the 20-deal timing fixture; add a
NumPy coefficient reference for the pruned capacity control. Fixed-sample
confirmation started with the preceding implementation; equal-time jobs use
the optimized implementation. Do not interpret concurrent before/after wall
times as a controlled benchmark.

The first two equal-time development runs favor round7, which completes about
twice as many terminal evaluations as capacity32 at the same 43 ms/hand.
This does not establish that more search beats production's 128 samples,
because the existing timed comparison gives both players the larger ceiling.
Add exactly that control: round7:8192:12:5 versus round7 on 500 fresh deals
starting at 1620000, separately against the same three responders. Run these
sequentially after the model's timed confirmation. Report the additional
latency as a tradeoff, not as a gain at equal computation. No tuning of the
5 ms cap or checkpoint on these new seeds.

After timing work, isolate the new Dan Q-diamond choice: force QD, 8H, or 9D
at event 145 and keep every subsequent bot on round7. Use 100 identical seeds
per move under actionclaim, gottatry, and round7 human responders. This checks
the first discard separately from the payoff model's other changed decisions;
it remains a single known-hand diagnostic, not a general performance test.

Completed: the capacity finalist fails the equal-time promotion criterion,
including a regression against actionhybrid. Extra search yields positive but
inconclusive estimates for all three responders at roughly twice the cost.
Keep round7's policy and 128-sample production budget. Retain research models,
profiles, exporters, inference optimizations, and bounded timed inference
setup. The forced Dan comparison does not blame the first QD discard for the
payoff model's later regression. All planned runs and checks are complete.
