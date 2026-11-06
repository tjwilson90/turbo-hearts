# Follow-up: robust continuations and selective adaptation

The second round improved weak-opponent matches but exposed overconfident moon
planning. This round tests two specific explanations before combining them:

1. **Defender-aware rollouts.** A public-information policy takes a heart or the
   queen to interrupt an established opposing moon, or supplies a heart to a
   different defender who wins the completed trick. Test defender fractions
   1.0 and 0.5. The actual move is still chosen by terminal-money search.
2. **Selective behavior adaptation.** Start from a 90% heuristic style prior;
   switch an opponent to duck/shooter only with posterior probability at least
   90% and three non-forced observations. Otherwise use the frozen baseline's
   opposing-shooter mixture. Infer on a cloned random stream so uncertain
   evidence leaves baseline sampled hands and rollout streams unchanged.
3. Combine selective adaptation and defensive continuations. Known duck or
   shooting policies are not artificially converted into defenders.

Development seeds: 140000–140999, separate from all previous training/matches.
Reserve 150000–151999 for confirmation after selecting the configuration.
Primary comparison: `defend:128:12`, the existing server configuration.
Screen on mixed, heuristic, and 32-sample search opponents; confirm promising
variants against full-strength search as well. Use paired deal-clustered money
intervals and four seat replays, track computation, and preserve failed tests.

If a continuation improves play, revisit the joint pass/charge planner with
that continuation rather than adding the previous optimistic planner unchanged.
Promote only after fresh matches support gains without a clear loss on the
other lineups. Preserve the prior server default as the `defend` benchmark.

## Development iterations

- Test selective style thresholds 0.75 and 0.60 after the 0.90 prototype
  preserved heuristic play exactly but recovered only a small mixed-lineup gain.
- Merge posterior mass for duck and shooting styles when the opponent can no
  longer shoot: those continuations then have identical play policies.
- Require both alternating halves of the sampled deals to favor shooting before
  valuing it above avoidance. This tests continuation-selection noise without
  adding simulation work.
- Reassess emerging opposing moon threats during rollouts, rather than only at
  their initial state.
- Extend exact search from 12 to 16 remaining cards. The initial unoptimized
  20-deal smoke run was expensive (47 ms/hand; maximum move 351 ms), so add a
  bounded per-decision cache of exact states and verify identical answers before
  evaluating strength. This tests removing rollout error for one more trick.

All these choices use development data only. Confirmation seeds remain reserved.

The changing-shooter continuation gave +5.99 [1.51, 10.48] on the development
mixed lineup, with inconclusive heuristic/search differences. Its combination
with the 0.60 selective model gave +7.12 [2.02, 12.22] and also no clear loss
in the other development lineups. Test it against full-strength search before
confirmation. Revisit charge search with the changing-shooter continuation;
its stronger heuristic result must also generalize to search opponents.

The first exact cache preserved results but barely improved speed. A canonical
cache, preserving ownership order, nines, chargeable cards and trick control,
roughly halved 16-card solver time with identical matched-game outcomes. The
16-card horizon itself failed to improve strength and regressed on heuristic
opponents, so retain 12-card endgames. Evaluate the cache separately at 12 cards.

## Locked confirmation decision

Before reading seeds 150000+: confirm two candidates against frozen
`defend:128:12`, with 2,000 mixed deals, 1,000 heuristic deals, and 1,000
full-strength `defend:128:12` opponent deals each.

A (preferred): `cautiousdynamic:128:12:none:0:32:0:0.6`.
B (simpler fallback): `cautious:128:12:none:0:32:0:0.6`.

For promotion, require a positive mixed-lineup gain even with a 97.5% interval
(two prespecified candidates), and no statistically clear loss in either other
lineup at 95%. Report all three intervals and limitations. Prefer A if both
qualify; otherwise promote whichever qualifies, or neither. Do not tune on
these results. Additional rule/lineup checks are for scope and regression
detection, not parameter selection. Keep passes and charges heuristic and the
exact horizon at 12 cards. Caches remain optional unless separately justified.

The inference implementation now skips work before three observed opponent
plays and during exact endgames, and avoids allocating tree paths when no tree
is enabled. These do not alter the candidate's decisions.

## Completed decision

Both candidates met the locked confirmation criteria. The preferred combined
candidate A is enabled in the production constructor. Full-strength search
results remain inconclusive. See [CONFIRMATION.md](CONFIRMATION.md) for all
primary and scope results. Post-promotion workspace tests passed, and the
release arena default matched the confirmed candidate exactly across 300
original deals replayed in all four seats. No deployment was performed.
