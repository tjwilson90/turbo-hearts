# Engine throughput and learned preplay

Preserve round-eight work and freeze the current round7 gameplay policy and
128-sample baseline. This round has three independent tracks:

1. Measure engine/simulation/sampling hot paths. Preserve legal actions, seeded
   random streams, scores, and decisions for exact optimizations. Compare old
   and optimized executables on the same positions/deals, then compare search
   work and payoff under the same soft decision-time budget. Test any deeper
   endgame/sample-budget changes separately from semantics-preserving changes.
2. Learn charging from actual terminal money in responsive population play,
   with repeated policy improvement and archived policies. Evaluate all charge
   subsets, include the decision to decline, respect public/blind information,
   repeated charging and keeper's first/second charging stages. Do not select
   the best play plan with privileged knowledge of the sampled hand.
3. Learn passing with explicit left/right/across/keeper conditioning. Directed
   passes transfer cards to the correct opponent; keeper redraws pool cards and
   can return one's own discards. Evaluate direction-specific results, and keep
   passing and charging ablations separate before combining them.

Use whole-deal training/validation splits; no human identity features. Training
seeds start at 1800000, development at 1900000; reserve 2000000+ for final
confirmation. Use actual money as the primary outcome, paired by original
deal, against current search and several human-oriented responders. Check all
six charging rules for legality and privacy. Report preplay and play time
separately. Preserve defaults unless performance evidence supports promotion.

Preplay generation 1 uses 12,000 deals across all six rules and four directions,
in shards of 6,000 / 3,000 / 3,000 starting at 1800000 / 1806000 / 1809000.
Each action has two paired continuation streams. Passing compares sixteen
candidates (current/heuristic/shooting/duck, nearby swaps, random triples);
charging enumerates every legal subset. The root's responsive play uses
round7:8:8, with a population of direct skilled/human policies, heuristic,
shooting, and round7:8:8 opponents. This is an inexpensive policy-improvement
teacher; full-budget match testing determines whether its recommendations
transfer. More initial deals are collected before fitting because rule,
direction, and keeper-stage conditioning divide the examples substantially.
Validation uses a mixed hash of the whole deal ID, avoiding the seat-parity
bias of a plain ID modulo split. Train separate 256-input/32-unit monetary
heads for passing and charging with centered MSE, which retains rare large
money outcomes rather than clipping the payoff objective.

Generation-one development compares passing alone, charging alone, and both
against `actionclaim`, `actionhybrid`, and `round7:32:12`, using 100 deals at
1901000 and full 128-sample root play. These are selection screens. Generation
two uses 6,000 new deals at 1812000, with the generation-one combined policy
as learner and an additional archived population member. Retain generation-one
data when fitting generation two; initialize from its corresponding head.
The first generation's validation estimates are +9.44 passing and +3.03
charging over their recorded parent decisions, but full matches are required:
passing inference considers all 286 legal triples, beyond the 16 sampled
training candidates. No model is promoted on target-fitting metrics alone.

Engine-budget development separately compares 160/192 samples and 16-card
exact endgames against the frozen 128-sample/12-card policy. Only fixed-sample
matches may run alongside training. Run the timing measurements sequentially
after generation/training completes. Final confirmation starts at 2000000;
select policies and budgets before inspecting it.

After generation-one screening, keeper passing is excluded from the candidate
for larger development: all three keeper point estimates were negative. A
direction mask permits this selection without changing other decisions.
Generation-one passing alone with mask 7 (left/right/across) is evaluated on
300 new development deals at 1903000 against the same three responders.
Generation-two screens repeat the original 100 development deals, now with
mask 7, for passing alone, charging alone, and both. These overlapping screens
remain development, and any selected direction restriction must be frozen
before confirmation. Generation two trains on both generations, 18,000 deals
total; both heads selected epoch 2 and stopped at epoch 23.

The second-generation passing screen is positive in all three responder point
estimates. Extend pass two, mask 7, to the same 300 development deals at
1903000. Select between pass one/two using these shared larger comparisons,
with emphasis on the human/search hybrid and avoiding a search regression.
Charge-only screens remain negative against search; no charging head is
selected for default use on those results. Confirm the selected passing head
on 500 unseen deals at 2000000 against each of `actionclaim`, `actionhybrid`,
`round7:32:12`, and `neuralhybrid:assets/models/human-history32.thh:32:12`.
Report each responder and pass direction separately; the four responders
share the same original deals and their intervals must not be pooled.

Selected for fresh confirmation: **pass two, margin 0, mask 7; heuristic
charging and keeper passing; unchanged 128 samples/12-card exact play**.
On the 300 new development deals, its gains were +18.42 direct human,
+19.93 human/search hybrid, and +4.84 search, versus +17.95/+12.62/+2.32
for pass one. Model selection is now frozen. Also audit the selected policy
on 200 unseen deals at 2001000 for each of the other five charging rules,
against `actionhybrid`, before deciding whether any default promotion should
extend beyond Classic. These shared deals are not independent across rules.

Before promotion, check simultaneous replacement of all bots in mixed tables,
using accurate public human/bot identities. The primary arena changes only
one policy at a time and marks everyone human, so this is a separate transfer
check. Freeze pass two/mask 7 and compare it with round7 on 300 deals at
2002000, with one or two human responders (`actionclaim` or `actionhybrid`).
Rotate every placement of the human seats, cluster by deal, and report money
per bot. Run these after the sequential engine measurements to avoid load
contamination. Classic is the only ruleset with a confirmed primary gain;
the smaller other-rule checks are uniformly positive but inconclusive.

The unrestricted simultaneous replacement regresses significantly in all four
mixed-table checks (one/two humans, direct/hybrid). Do not promote it globally.
Hypothesis: all bots retaining control removes the helpful high cards the
learner received from heuristic-passing bots during training. Explore a public
recipient restriction: use the learned pass only when its destination is a
human. This preserves heuristic bot-to-bot passing. Screen the four mixed
table configurations on 100 development deals at 1908000, using the frozen
pass-two checkpoint, margin 0 and mask 7. Independently confirm the unchanged
pass-two policy in the lone-bot/three-human case with accurate public roles,
300 deals at 2003000 against both direct and hybrid human responders.

Final decision: the recipient-restricted mixed-table screens also fail. The
lone-bot confirmations give +26.72 [17.71, 35.73] direct and +8.71 [0.36, 17.06]
hybrid money/hand. Enable pass two only for Classic, three public human
opponents, and left/right/across. Retain heuristic passing elsewhere, all
heuristic charging, and the existing card-play budget. The general preplay
profiles, recipient restriction, and all checkpoints remain opt-in research.
