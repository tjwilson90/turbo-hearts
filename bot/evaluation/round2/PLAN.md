# Second exploration round

Baseline: commit 826b6cc (`defend:128:12`). No new experiment becomes the default
without fresh-seed confirmation. Development seeds: 70000–70999; confirmation:
90000–91999. Training starts at 100000 and excludes both match sets.

1. Opponent modeling: infer a per-seat mixture of heuristic, duck, and shooting
   policies from observed legal choices, integrating over sampled hands.
2. Behavioral belief: importance-weight otherwise legal deals by the likelihood
   of observed choices, with a noise floor and tempering to limit overconfidence.
3. Population self-play: policy-gradient updates from terminal money against
   diverse opponents and archived policies, initialized from the existing model.
4. Information-set planning: pool own future decisions by public observation
   histories, exploring alternatives beyond fixed rollout continuations.
5. Pre-play planning: compare pass and charge candidates with shooting-aware
   continuations, preserving the exact charging and keeper rules.
6. Hard-position training: mine disagreements and ambiguous moon decisions,
   label with larger search budgets, and compare with uniform-data training.
7. Neural allocation: use the policy to order exploratory samples, then eliminate
   weak moves and spend the remaining root-evaluation budget on contenders.

For each: retain code, commands, match results, costs, and limitations. Screening
is exploratory; use paired original-deal confidence intervals and independent
confirmation, including search opponents and mixed lineups. The purpose is to
explore each direction; an unsuccessful prototype is not evidence that the whole
research direction is unpromising.

## Confirmation choice (before opening fresh results)

Advance `model:128:12` alone: it improved the mixed development lineup and
showed no clear regression against heuristic or search opponents. Pass search,
behavior-weighted deals, and charge search stay experimental because their
combined gains depended strongly on the lineup. Confirm against the frozen
`defend:128:12` baseline on mixed (2,000 deals), heuristic (1,000), and
`defend:128:12` opponents (1,000), starting at 90000. Promote only if the mixed
gain persists and the other lineups show no statistically clear loss; report
the remaining uncertainty instead of claiming equivalence. Also confirm the
third RL generation against its initial direct policy on 1,000 fresh mixed
deals; it remains a research checkpoint rather than a calibrated search prior.

The 45-epoch hard/uniform trainers both selected their final epoch. Extend both
to 120 epochs with identical starting weights and optimizer settings, then
repeat the development comparison; this does not consume new confirmation
seeds or affect the locked opponent-model candidate.
