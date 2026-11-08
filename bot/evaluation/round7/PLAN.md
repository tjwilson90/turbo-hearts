# Successful human players

User priority: improve against humans, especially consistently successful ones.
Synthetic search-opponent uncertainty is reported, but is not an automatic veto
as in round six. No offline simulation is described as a live-human win-rate gain.

1. Replay complete games through the engine. Fit opponent-adjusted player ratings
   from training-game money per hand only, with ridge 30. Among humans with at
   least 30 training games, freeze the top third. Include their losses as well
   as wins. Validation/test outcomes never determine membership.
2. Fit a richer contextual opponent policy on that cohort's training actions.
   Select on validation likelihood and report later-game likelihood separately.
   The existing all-human model is the comparator. No player identifiers enter
   the runtime model.
3. Audit human/bot disagreements at positions sampled independently of outcomes.
   Use actual hands only for labeled counterfactual diagnostics, with responsive
   continuations under more than one policy. Do not force recorded future moves
   or assign the final observed score as causal credit for an individual move.
4. Screen individual changes on validation starts and fresh synthetic deals.
   Confirm a selected change on the test starts and fresh seeds. Emphasize the
   learned successful-player responder plus a search/learned hybrid, with
   ordinary-human, shooting, and search populations as cross-checks. Keep
   production and every research baseline reproducible if promoting a change.

The existing historical test split has been used in earlier research. Report it
as later-game evaluation, not as an entirely untouched corpus. The four recent
games in th2.db are diagnostics and cannot establish player skill on their own.

Final candidate frozen before the later-game match confirmation: `skilledmodel`,
100% successful-human contextual-mixture continuations for seats publicly marked
human, with round-four play otherwise unchanged. It had positive point estimates
against all four validation responders; the more accurate direct action predictor
is retained as an independent responder, not automatically promoted. Confirm on
all 244 later games containing a cohort member with `actionclaim`, `actionhybrid`,
`skilledclaim`, and `skilledhybrid`, using round4 as the frozen baseline.

After confirmation, the fitted-only model improved both imitation responders but
was uncertain against hybrids (-2.95 with the direct hybrid, +3.30 with the
mixture hybrid). Code inspection found that fitted policies replace even a
confident inferred shooting style. Screen two explicit fallback variants on
100 validation games: `skilledfallback` retains inferred duck/shooter styles;
`skilledadaptive` also retains sampled early-intent evidence. Both otherwise
use the fitted model for human seats. This is follow-up development after seeing
the later block; do not describe subsequent historical results as untouched.
The original finalist's fresh-deal checks continue unchanged.

Final follow-up selection: `skilledfallback`. On 100 validation games it gains
+16.75 against actionclaim and is essentially tied against actionhybrid. The
early-intent addition gives +15.78 and +0.01 respectively, so keep the simpler
fallback without the extra signal. Freeze this choice before new-seed matches
at 1400000 (1000 deals each against actionclaim and actionhybrid). Repeat the
later-game diagnostic against actionclaim, actionhybrid, skilledhybrid, and
general humanclaim. Do not tune further using those results. Promotion is
based on human-focused evidence; uncertain synthetic hybrid results are reported,
not automatically interpreted as either superiority or a veto.
