# Development screens

Exploratory paired gains in money per hand; generated-deal screens use 200 original deals starting at 170000. Intervals cluster four seat replays per deal. Historical screens cluster by original game.

`baseline: default` in development JSONs means the frozen round-three build. Reproduce it using `cautiousdynamic:128:12:none:0:32:0:0.6`; the current default includes the promoted stopper preference.

| Variant | Opponents | Gain | 95% interval |
| --- | --- | ---: | --- |
| claims:128:12 | defend:128:12 | +1.27 | [-2.03, +4.58] |
| claims:128:12 | humanclaim | -1.92 | [-5.56, +1.73] |
| claims:128:12 | mixed | +6.51 | [+1.48, +11.53] |
| early:128:12 | mixed | +1.33 | [-4.10, +6.77] |
| humanmodel:128:12 | defend:128:12 (historical) | -10.40 | [-21.22, +0.41] |
| humanmodel:128:12 | humanclaim (historical) | +9.57 | [-0.71, +19.85] |
| preserve:128:12:none:0:32:8 | defend:128:12 (historical) | +0.33 | [-0.48, +1.14] |
| preserve:128:12:none:0:32:8 | humanclaim (historical) | +0.06 | [-0.58, +0.70] |
| humanmix:128:12 | defend:128:12 | +5.49 | [-2.04, +13.03] |
| humanmix:128:12 | human | -2.43 | [-9.61, +4.76] |
| humanmix:128:12 | mixed | +2.25 | [-7.17, +11.67] |
| humanmodel:128:12 | defend:128:12 | -6.74 | [-16.02, +2.55] |
| humanmodel:128:12 | human | +11.31 | [+2.54, +20.08] |
| humanmodel:128:12 | mixed | +9.69 | [-1.94, +21.33] |
| plan:128:12 | mixed | -1.68 | [-5.27, +1.91] |
| preserve:128:12 | defend:128:12 | -0.06 | [-0.89, +0.76] |
| preserve:128:12 | heuristic | -0.02 | [-0.26, +0.23] |
| preserve:128:12 | human | -0.43 | [-1.41, +0.56] |
| preserve:128:12 | mixed | +1.30 | [+0.26, +2.35] |
| preserve:128:12:none:0:32:8 | defend:128:12 | +0.70 | [-0.52, +1.93] |
| preserve:128:12:none:0:32:8 | human | -0.53 | [-1.55, +0.50] |
| preserve:128:12:none:0:32:8 | mixed | +2.42 | [+1.02, +3.83] |
| safe:128:12 | defend:128:12 | -3.50 | [-7.57, +0.58] |
| safe:128:12 | human | -2.76 | [-6.29, +0.77] |
| safe:128:12 | mixed | +2.89 | [-2.56, +8.33] |
| safeearly:128:12 | defend:128:12 | -0.65 | [-4.63, +3.34] |
| safeearly:128:12 | humanclaim | -3.33 | [-8.22, +1.56] |
| safeearly:128:12 | mixed | +9.29 | [+2.57, +16.02] |

The `claim-v1-*` files retain an initial claim prototype that used `can_claim` rather than the server’s `should_claim` eligibility. They are not results for the final claim implementation.
