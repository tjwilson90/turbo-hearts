# Complete development screens

Classic rules; 300 original deals per row, four seat replays each. Paired 95% intervals are clustered by deal. Seeds start at 140000. All candidates compare with frozen `defend:128:12`. These reused-deal screens are exploratory.

| Result | Candidate | Opponents | Money gain [95% CI] | Play / pre-play ms per hand |
| --- | --- | --- | ---: | ---: |
| [cautious-heuristic-screen](cautious-heuristic-screen.json) | `cautious:128:12` | `heuristic` | +0.00 [0.00, 0.00] | 16.53 / 0.00 |
| [cautious-mixed-screen](cautious-mixed-screen.json) | `cautious:128:12` | `mixed` | +0.82 [-0.66, 2.29] | 15.49 / 0.00 |
| [cautious-search-screen](cautious-search-screen.json) | `cautious:128:12` | `defend:32:12` | -0.03 [-0.10, 0.03] | 16.00 / 0.00 |
| [cautious60-heuristic-screen](cautious60-heuristic-screen.json) | `cautious:128:12:none:0:32:0:0.6` | `heuristic` | +0.00 [0.00, 0.00] | 16.61 / 0.00 |
| [cautious60-mixed-screen](cautious60-mixed-screen.json) | `cautious:128:12:none:0:32:0:0.6` | `mixed` | +1.71 [-0.68, 4.11] | 15.63 / 0.00 |
| [cautious60-search-screen](cautious60-search-screen.json) | `cautious:128:12:none:0:32:0:0.6` | `defend:32:12` | +0.58 [-0.40, 1.56] | 16.31 / 0.00 |
| [cautious75-heuristic-screen](cautious75-heuristic-screen.json) | `cautious:128:12:none:0:32:0:0.75` | `heuristic` | +0.00 [0.00, 0.00] | 16.67 / 0.00 |
| [cautious75-mixed-screen](cautious75-mixed-screen.json) | `cautious:128:12:none:0:32:0:0.75` | `mixed` | +0.86 [-0.93, 2.65] | 15.48 / 0.00 |
| [cautious75-search-screen](cautious75-search-screen.json) | `cautious:128:12:none:0:32:0:0.75` | `defend:32:12` | -0.26 [-0.61, 0.09] | 16.24 / 0.00 |
| [cautiousdynamic60-fullsearch-screen](cautiousdynamic60-fullsearch-screen.json) | `cautiousdynamic:128:12:none:0:32:0:0.6` | `defend:128:12` | -0.33 [-3.81, 3.15] | 16.37 / 0.00 |
| [cautiousdynamic60-heuristic-screen](cautiousdynamic60-heuristic-screen.json) | `cautiousdynamic:128:12:none:0:32:0:0.6` | `heuristic` | -1.13 [-4.14, 1.89] | 17.45 / 0.00 |
| [cautiousdynamic60-mixed-screen](cautiousdynamic60-mixed-screen.json) | `cautiousdynamic:128:12:none:0:32:0:0.6` | `mixed` | +7.12 [2.02, 12.22] | 16.39 / 0.00 |
| [cautiousdynamic60-search-screen](cautiousdynamic60-search-screen.json) | `cautiousdynamic:128:12:none:0:32:0:0.6` | `defend:32:12` | +0.81 [-2.99, 4.61] | 17.32 / 0.00 |
| [cautiousguard50-heuristic-screen](cautiousguard50-heuristic-screen.json) | `cautiousguard:128:12:none:0:32:0.5` | `heuristic` | +1.70 [-2.12, 5.51] | 17.22 / 0.00 |
| [cautiousguard50-mixed-screen](cautiousguard50-mixed-screen.json) | `cautiousguard:128:12:none:0:32:0.5` | `mixed` | -0.55 [-5.43, 4.32] | 16.20 / 0.00 |
| [cautiousguard50-search-screen](cautiousguard50-search-screen.json) | `cautiousguard:128:12:none:0:32:0.5` | `defend:32:12` | +0.87 [-3.97, 5.70] | 17.01 / 0.00 |
| [consensus-heuristic-screen](consensus-heuristic-screen.json) | `consensus:128:12` | `heuristic` | +1.21 [-0.34, 2.76] | 12.53 / 0.00 |
| [consensus-mixed-screen](consensus-mixed-screen.json) | `consensus:128:12` | `mixed` | -0.84 [-2.90, 1.22] | 11.77 / 0.00 |
| [consensus-search-screen](consensus-search-screen.json) | `consensus:128:12` | `defend:32:12` | +0.08 [-0.98, 1.13] | 12.16 / 0.00 |
| [dynamic-heuristic-screen](dynamic-heuristic-screen.json) | `dynamic:128:12` | `heuristic` | -1.13 [-4.14, 1.89] | 13.09 / 0.00 |
| [dynamic-mixed-screen](dynamic-mixed-screen.json) | `dynamic:128:12` | `mixed` | +5.99 [1.51, 10.48] | 12.33 / 0.00 |
| [dynamic-search-screen](dynamic-search-screen.json) | `dynamic:128:12` | `defend:32:12` | +0.14 [-3.34, 3.62] | 13.54 / 0.00 |
| [dynamiccharge-fullsearch-screen](dynamiccharge-fullsearch-screen.json) | `dynamiccharge:128:12` | `defend:128:12` | -3.73 [-10.80, 3.34] | 12.55 / 0.46 |
| [dynamiccharge-heuristic-screen](dynamiccharge-heuristic-screen.json) | `dynamiccharge:128:12` | `heuristic` | +13.49 [7.30, 19.67] | 13.33 / 0.45 |
| [dynamiccharge-mixed-screen](dynamiccharge-mixed-screen.json) | `dynamiccharge:128:12` | `mixed` | +10.81 [3.34, 18.29] | 12.49 / 0.43 |
| [dynamiccharge-search-screen](dynamiccharge-search-screen.json) | `dynamiccharge:128:12` | `defend:32:12` | -5.25 [-12.51, 2.01] | 12.28 / 0.43 |
| [dynamicguard50-heuristic-screen](dynamicguard50-heuristic-screen.json) | `dynamicguard:128:12:none:0:32:0.5` | `heuristic` | +0.12 [-3.78, 4.02] | 15.21 / 0.00 |
| [dynamicguard50-mixed-screen](dynamicguard50-mixed-screen.json) | `dynamicguard:128:12:none:0:32:0.5` | `mixed` | +2.90 [-2.49, 8.30] | 13.77 / 0.00 |
| [dynamicguard50-search-screen](dynamicguard50-search-screen.json) | `dynamicguard:128:12:none:0:32:0.5` | `defend:32:12` | +2.25 [-2.84, 7.34] | 14.83 / 0.00 |
| [exact16-heuristic-screen](exact16-heuristic-screen.json) | `cached:128:16` | `heuristic` | -3.86 [-6.68, -1.05] | 55.98 / 0.00 |
| [exact16-mixed-screen](exact16-mixed-screen.json) | `cached:128:16` | `mixed` | +0.27 [-3.01, 3.55] | 60.51 / 0.00 |
| [exact16-search-screen](exact16-search-screen.json) | `cached:128:16` | `defend:32:12` | -0.37 [-2.39, 1.64] | 52.76 / 0.00 |
| [guard-heuristic-screen](guard-heuristic-screen.json) | `guard:128:12` | `heuristic` | +1.18 [-3.52, 5.87] | 13.58 / 0.00 |
| [guard-mixed-screen](guard-mixed-screen.json) | `guard:128:12` | `mixed` | -3.82 [-9.26, 1.61] | 12.75 / 0.00 |
| [guard-search-screen](guard-search-screen.json) | `guard:128:12` | `defend:32:12` | -0.82 [-6.00, 4.37] | 13.32 / 0.00 |
| [guard50-heuristic-screen](guard50-heuristic-screen.json) | `guard:128:12:none:0:32:0.5` | `heuristic` | +1.70 [-2.12, 5.51] | 13.91 / 0.00 |
| [guard50-mixed-screen](guard50-mixed-screen.json) | `guard:128:12:none:0:32:0.5` | `mixed` | -1.93 [-6.54, 2.69] | 13.13 / 0.00 |
| [guard50-search-screen](guard50-search-screen.json) | `guard:128:12:none:0:32:0.5` | `defend:32:12` | +0.94 [-3.87, 5.75] | 13.85 / 0.00 |
| [guardconsensus50-heuristic-screen](guardconsensus50-heuristic-screen.json) | `guardconsensus:128:12:none:0:32:0.5` | `heuristic` | +2.01 [-2.01, 6.03] | 13.92 / 0.00 |
| [guardconsensus50-mixed-screen](guardconsensus50-mixed-screen.json) | `guardconsensus:128:12:none:0:32:0.5` | `mixed` | -2.48 [-7.26, 2.29] | 13.16 / 0.00 |
| [guardconsensus50-search-screen](guardconsensus50-search-screen.json) | `guardconsensus:128:12:none:0:32:0.5` | `defend:32:12` | +0.09 [-4.58, 4.75] | 13.92 / 0.00 |
| [selective60-heuristic-screen](selective60-heuristic-screen.json) | `selective:128:12:none:0:32:0:0.6` | `heuristic` | +0.00 [0.00, 0.00] | 17.48 / 0.00 |
| [selective60-mixed-screen](selective60-mixed-screen.json) | `selective:128:12:none:0:32:0:0.6` | `mixed` | +0.90 [-2.21, 4.01] | 16.56 / 0.00 |
| [selective60-search-screen](selective60-search-screen.json) | `selective:128:12:none:0:32:0:0.6` | `defend:32:12` | +0.78 [-0.54, 2.11] | 16.65 / 0.00 |

Timing was collected during concurrent work; use it as descriptive cost information. Cache parity tests are separate JSON files because they test equivalent algorithms rather than playing-strength changes.
