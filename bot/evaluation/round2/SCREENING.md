# Development screens

These are exploratory comparisons on reused development deals, not independent confirmations. `default` in these records is the frozen `defend:128:12` configuration. All rules are Classic. See [README.md](README.md) for methods, caveats, and the fresh-deal decision.

Play and pre-play times are milliseconds per whole hand for the candidate, measured during concurrent work. They are descriptive, not controlled latency benchmarks. Neural rows compare greedy direct policies unless labeled as guided search.

| Result file | Candidate | Baseline | Opponents | Deals | Money gain [95% CI] | Play / pre-play ms |
| --- | --- | --- | --- | ---: | ---: | ---: |
| [allocate32-mixed-screen](allocate32-mixed-screen.json) | `allocate:128:12` | `default` | `mixed` | 300 | -0.80 [-6.62, 5.01] | 12.74 / 0.00 |
| [belief-mixed-screen](belief-mixed-screen.json) | `belief:128:12` | `default` | `mixed` | 300 | -2.79 [-9.97, 4.39] | 14.44 / 0.00 |
| [charge-mixed-screen](charge-mixed-screen.json) | `charge:128:12` | `default` | `mixed` | 300 | +9.23 [3.52, 14.95] | 11.78 / 0.40 |
| [charge-search-screen](charge-search-screen.json) | `charge:128:12` | `default` | `defend:32:12` | 300 | -3.57 [-8.61, 1.47] | 11.67 / 0.41 |
| [combined-mixed-screen](combined-mixed-screen.json) | `combined:128:12` | `default` | `mixed` | 300 | +72.65 [58.50, 86.81] | 14.66 / 23.62 |
| [combined-search-screen](combined-search-screen.json) | `combined:128:12` | `default` | `defend:32:12` | 300 | -26.54 [-37.86, -15.22] | 15.24 / 23.08 |
| [guided16-vs-allocate-screen](guided16-vs-allocate-screen.json) | `guided:16:12:policy-moon.thp:0:4` | `allocate:16:12:none:0:4` | `mixed` | 300 | -1.97 [-7.48, 3.54] | 2.04 / 0.00 |
| [guided32-mixed-screen](guided32-mixed-screen.json) | `guided:128:12:policy-moon.thp:0` | `default` | `mixed` | 300 | -0.48 [-6.61, 5.65] | 13.19 / 0.00 |
| [guided8-mixed-screen](guided8-mixed-screen.json) | `guided:128:12:policy-moon.thp:0` | `default` | `mixed` | 300 | -5.92 [-13.58, 1.73] | 13.19 / 0.00 |
| [hard-long-vs-initial-screen](hard-long-vs-initial-screen.json) | `policy:hard-long.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | -2.14 [-7.27, 3.00] | 0.40 / 0.00 |
| [hard-long-vs-uniform-screen](hard-long-vs-uniform-screen.json) | `policy:hard-long.thp:0` | `policy:uniform-long.thp:0` | `mixed` | 1000 | -2.92 [-7.67, 1.84] | 0.41 / 0.00 |
| [hard-vs-initial-screen](hard-vs-initial-screen.json) | `policy:hard.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | -1.36 [-6.41, 3.69] | 0.28 / 0.00 |
| [hard-vs-uniform-screen](hard-vs-uniform-screen.json) | `policy:hard.thp:0` | `policy:uniform.thp:0` | `mixed` | 1000 | -2.14 [-6.92, 2.64] | 0.28 / 0.00 |
| [model-heuristic-screen](model-heuristic-screen.json) | `model:128:12` | `defend:128:12` | `heuristic` | 500 | +0.55 [-4.92, 6.01] | 17.18 / 0.00 |
| [model-mixed-screen](model-mixed-screen.json) | `model:128:12` | `default` | `mixed` | 300 | +12.35 [5.00, 19.71] | 14.66 / 0.00 |
| [model-search-screen](model-search-screen.json) | `model:128:12` | `default` | `defend:32:12` | 300 | +1.11 [-6.68, 8.91] | 15.83 / 0.00 |
| [modelcharge-heuristic-screen](modelcharge-heuristic-screen.json) | `modelcharge:128:12` | `defend:128:12` | `heuristic` | 500 | +10.12 [3.34, 16.90] | 16.87 / 0.43 |
| [modelcharge-mixed-screen](modelcharge-mixed-screen.json) | `modelcharge:128:12` | `default` | `mixed` | 300 | +27.74 [19.09, 36.39] | 14.34 / 0.39 |
| [modelcharge-search-screen](modelcharge-search-screen.json) | `modelcharge:128:12` | `default` | `defend:32:12` | 300 | -4.27 [-12.38, 3.85] | 14.89 / 0.41 |
| [modeltree-mixed-screen](modeltree-mixed-screen.json) | `modeltree:128:12` | `posterior:128:12` | `mixed` | 300 | -1.57 [-9.87, 6.73] | 15.91 / 0.00 |
| [pass-mixed-screen](pass-mixed-screen.json) | `pass:128:12` | `default` | `mixed` | 300 | +13.93 [3.92, 23.94] | 12.26 / 23.02 |
| [posterior-heuristic-screen](posterior-heuristic-screen.json) | `posterior:128:12` | `defend:128:12` | `heuristic` | 500 | +4.50 [-1.29, 10.29] | 17.06 / 0.00 |
| [posterior-search-screen](posterior-search-screen.json) | `posterior:128:12` | `default` | `defend:32:12` | 300 | -6.86 [-14.16, 0.45] | 15.39 / 0.00 |
| [posterior-vs-model-screen](posterior-vs-model-screen.json) | `posterior:128:12` | `model:128:12` | `mixed` | 300 | +9.82 [1.59, 18.04] | 15.20 / 0.00 |
| [posteriorcharge-mixed-screen](posteriorcharge-mixed-screen.json) | `posteriorcharge:128:12` | `default` | `mixed` | 300 | +40.52 [30.57, 50.47] | 14.13 / 0.39 |
| [posteriorcharge-search-screen](posteriorcharge-search-screen.json) | `posteriorcharge:128:12` | `default` | `defend:32:12` | 300 | -4.65 [-13.09, 3.78] | 14.80 / 0.41 |
| [preplay-mixed-screen](preplay-mixed-screen.json) | `preplay:128:12` | `defend:128:12` | `mixed` | 300 | +35.45 [22.75, 48.15] | 12.44 / 24.92 |
| [preplay-search-screen](preplay-search-screen.json) | `preplay:128:12` | `defend:128:12` | `defend:32:12` | 300 | -7.93 [-17.73, 1.88] | 12.91 / 24.83 |
| [preplay-v1-mixed-screen](preplay-v1-mixed-screen.json) | `preplay:128:12` | `default` | `mixed` | 300 | +35.28 [22.57, 47.99] | 11.71 / 23.10 |
| [preplay-v1-search-screen](preplay-v1-search-screen.json) | `preplay:128:12` | `default` | `defend:32:12` | 300 | -7.76 [-17.62, 2.09] | 12.77 / 24.22 |
| [rl-1-mixed-screen](rl-1-mixed-screen.json) | `policy:policy-rl-1.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | +0.52 [-2.72, 3.76] | 0.28 / 0.00 |
| [rl-2-mixed-screen](rl-2-mixed-screen.json) | `policy:policy-rl-2.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | +0.62 [-2.96, 4.19] | 0.28 / 0.00 |
| [rl-3-mixed-screen](rl-3-mixed-screen.json) | `policy:policy-rl-3.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | +1.98 [-1.97, 5.92] | 0.28 / 0.00 |
| [tree-mixed-screen](tree-mixed-screen.json) | `tree:128:12` | `default` | `mixed` | 300 | -0.81 [-8.53, 6.92] | 13.29 / 0.00 |
| [tree512-mixed-screen](tree512-mixed-screen.json) | `tree:512:12` | `defend:512:12` | `mixed` | 300 | +0.18 [-7.71, 8.07] | 54.52 / 0.00 |
| [uniform-long-vs-initial-screen](uniform-long-vs-initial-screen.json) | `policy:uniform-long.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | +0.78 [-4.19, 5.74] | 0.41 / 0.00 |
| [uniform-vs-initial-screen](uniform-vs-initial-screen.json) | `policy:uniform.thp:0` | `policy:policy-moon.thp:0` | `mixed` | 1000 | +0.78 [-4.19, 5.74] | 0.28 / 0.00 |

`guided8` used the original eight-round elimination threshold; append `:8` explicitly to reproduce it. `preplay-v1` predates the root keeper-discard correction. Early records without a pre-play timing field display zero in that column; all those variants used the inexpensive heuristic pre-play. Checkpoint paths are shortened in this table; JSON files preserve the actual invocation arguments.
