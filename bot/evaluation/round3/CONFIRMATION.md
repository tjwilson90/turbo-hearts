# Confirmation and final selection

These comparisons follow the locked choice in [PLAN.md](PLAN.md). All primary matches use Classic rules, fresh seeds beginning at 150000, and paired intervals clustered by original deal. Each deal is replayed in all four seats for both versions.

| Candidate | Opponents | Deals | Gain [95% CI] |
| --- | --- | ---: | ---: |
| [cautiousdynamic60](cautiousdynamic60-mixed-confirm.json) | `mixed` | 2000 | +4.23 [2.34, 6.11] |
| [cautiousdynamic60](cautiousdynamic60-heuristic-confirm.json) | `heuristic` | 1000 | +1.95 [0.41, 3.49] |
| [cautiousdynamic60](cautiousdynamic60-search-confirm.json) | `defend:128:12` | 1000 | -0.33 [-2.44, 1.77] |
| [cautious60](cautious60-mixed-confirm.json) | `mixed` | 2000 | +2.25 [1.21, 3.30] |
| [cautious60](cautious60-heuristic-confirm.json) | `heuristic` | 1000 | +0.00 [0.00, 0.00] |
| [cautious60](cautious60-search-confirm.json) | `defend:128:12` | 1000 | +0.32 [-0.10, 0.74] |

Both candidates passed the prespecified criteria. Their mixed-lineup 97.5% intervals (accounting for two candidate choices) are [2.07, 6.38] for the combined model and [1.06, 3.45] for selective modeling alone. The combined model was preferred in advance if both passed, so it is the production default. The search-opponent comparisons are inconclusive; these results do not prove equivalence or gains against every opponent.

Production uses `SearchExperiments::production()` through `NeuralNetworkBot::new()` and `with_seed()`. The server already constructs that bot for `BotStrategy::NeuralNet`. The arena `default` uses the same configuration, with its time cap disabled for reproducibility. `with_config` and the `defend` profile preserve the prior baseline unless extensions are explicitly supplied. No new weights, pass/charge search, defender override, tree search, or longer exact horizon is enabled.

## Additional scope checks

These fixed-candidate checks did not select parameters. They share original seeds and some rule variants produce identical event sequences; the rows are not independent replications.

| Rules | Opponents | Deals | Gain [95% CI] |
| --- | --- | ---: | ---: |
| [blind](scope-blind-mixed.json) | `mixed` | 300 | +4.98 [0.12, 9.83] |
| [blind_bridge](scope-blind_bridge-mixed.json) | `mixed` | 300 | +4.98 [0.12, 9.83] |
| [blind_chain](scope-blind_chain-mixed.json) | `mixed` | 300 | +4.98 [0.12, 9.83] |
| [bridge](scope-bridge-mixed.json) | `mixed` | 300 | +4.70 [-0.34, 9.74] |
| [chain](scope-chain-mixed.json) | `mixed` | 300 | +4.70 [-0.34, 9.74] |
| [classic](scope-classic-gottatry.json) | `gottatry` | 500 | +9.73 [3.00, 16.46] |
| [classic](scope-classic-random.json) | `random` | 500 | +0.07 [-1.59, 1.73] |

The full workspace test suite passed after enabling the production policy, including server integration, legal games under all six rulesets, and equality between the production arena profile and the confirmed candidate.
