# Human-play follow-up

Development positions include the reported ace-of-hearts slough in game
`a29f8493-60f9-445c-9992-bd91b6593caf`. It is a regression case, not independent
evidence of strength. Bot development screens use 200 original deals beginning
at 170000, four seat replays, and paired confidence intervals clustered by deal.

Directions: a separate own defensive continuation, first-heart shooter
recognition, claims in continuations, a bounded preference for preserving a
supported master heart, and a human action model. Combinations and unsuccessful
screens are retained. Claim prototypes initially used `can_claim` unconditionally;
the final implementation uses the server's `should_claim` rule so it does not
voluntarily collect avoidable penalties after a moon is broken. Earlier claim
screens are labeled `claim-v1-*` and are not evidence for the corrected variant.

Historical data is exported read-only, excluding chats and replacing identities
and seeds. All four hands of each game share a chronological split: 1,189
training games, 255 validation games, 255 test games. One training game has no
non-forced human plays. The fitted mixture is selected only by validation action
likelihood. Test likelihood is computed after selection. No position from the
reported November game occurs in this database ending in July.

Historical match screens use the first 50 validation games, with actual human
passes and charges. Subsequent moves come from simulated opponents; recorded
actions are never forced after a counterfactual choice. Match confidence
intervals cluster all four hands and seat replays by original game. These are
not observed win rates against humans.

## Fresh confirmation rule (fixed before running fresh matches)

Preferred candidate: production round-three search plus a maximum 8-money
preference for retaining a current master heart with at least two supporting
hearts, only when discarding onto another player's trick while an opposing moon
remains possible. It does not alter search payoffs or hidden-hand sampling.

Baseline: frozen `cautiousdynamic:128:12:none:0:32:0:0.6` (round three).
The candidate must show a positive paired 95% interval against mixed opponents
on 2,000 fresh deals starting at 180000. It must not show a clear loss (a wholly
negative 95% interval) against heuristic opponents (1,000 deals), full-strength
search (500 deals), or the fitted human model with legal claims (1,000 deals),
all starting at 180000. This is one selected candidate; no fallback will be
promoted from these same confirmation seeds if it fails.

Also evaluate all 255 held-out historical games against the fitted model with
claims, and check the other five charging rules on 100 deals each. A clear loss
in the historical check blocks promotion. Scope checks are exploratory; the
human corpus contains only Classic games. Inconclusive comparisons do not prove
equivalence. Other variants remain opt-in unless separately confirmed.
