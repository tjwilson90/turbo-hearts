# Independent confirmation

The first four comparisons were selected before these results were opened; see [PLAN.md](PLAN.md). The fifth checks the practical policy-versus-search gap afterward. All rows use Classic rules and seeds starting at 90000. Each original deal is replayed in all four seats per version, and intervals use paired deal-clustered differences.

| Result | Opponents | Deals | Candidate money | Baseline money | Gain [95% CI] |
| --- | --- | ---: | ---: | ---: | ---: |
| [model-mixed-confirm](model-mixed-confirm.json) | `mixed` | 2000 | 67.29 | 49.60 | +17.69 [14.32, 21.05] |
| [model-heuristic-confirm](model-heuristic-confirm.json) | `heuristic` | 1000 | 36.68 | 41.73 | -5.05 [-8.54, -1.55] |
| [model-search-confirm](model-search-confirm.json) | `defend:128:12` | 1000 | -7.82 | 0.00 | -7.82 [-10.93, -4.72] |
| [rl-3-mixed-confirm](rl-3-mixed-confirm.json) | `mixed` | 1000 | 13.49 | 6.96 | +6.53 [2.74, 10.32] |
| [rl-3-vs-search-confirm](rl-3-vs-search-confirm.json) | `mixed` | 1000 | 13.49 | 50.08 | -36.59 [-41.77, -31.41] |

The first three rows compare behavioral opponent modeling with frozen search. It wins on mixed opponents but loses against both heuristic and full-strength search opponents, so it was **not promoted**. The fourth row compares the final self-play policy with its initial direct policy. The fifth compares that improved policy with frozen search: the policy is faster but substantially weaker. The fifth comparison was added after the standalone policy result to check that practical strength gap, not to select or tune a checkpoint.

At the conclusion of this round, `NeuralNetworkBot::new()` and the server's `BotStrategy::NeuralNet` path remained unchanged. The later [third-round follow-up](../round3/README.md) selected a more conservative continuation model; no neural checkpoint is loaded by default.
