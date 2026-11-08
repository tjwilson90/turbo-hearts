# Early human signals and preventive defense

Freeze four research candidates before match screens. Each builds on current
production (128 samples, 12-card exact endgames, stopper margin 8):

- `chargebelief`: weight feasible deals by the human charging likelihood to
  power 0.5. All four chargeable cards; first opportunity only; Classic non-keeper.
- `intent`: a soft early shooting mixture when a player forgoes a safe high-card
  disposal opportunity. Rank likelihood is a heuristic, not learned intent.
- `disrupt`: a third own continuation that supplies early hearts onto completed
  tricks and sloughs off-suit nines without own suit control. Also cashes a master
  to break an established run. Continuation means are maximized across deals.
- `humanaware`: all three additions together.

Charging probabilities use training games only; validation chooses one of four
shrinkage settings. Test likelihood is reported after this choice. No charging
probability is interpreted as a probability of shooting. Known ownership and
void constraints are never relaxed. Keeper and non-Classic charge inference
remain disabled because the training corpus does not support those histories.

Initial screens: 200 fresh deals against mixed, 100 against the fitted claiming
human policy, and 100 against 32-sample search, starting at seed 600000. All seats
are replayed, with intervals clustered by deal. These are development results.
Human-shaped starts: first 30 validation games, if a candidate merits follow-up.
The recorded Dan game and the original ace-slough example are diagnostic only.

Promote at most one candidate only after a separate fresh comparison, starting
at 800000, establishes a positive interval in its target population and excludes
a loss worse than 1 money/hand against the other tested populations. Otherwise
leave production unchanged and report limitations. Do not reuse development
results as confirmation. Simulated opponent policies are not live humans.
