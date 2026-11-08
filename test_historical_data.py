import copy
import unittest
import numpy as np

from scripts.export_human_games import sanitize
from scripts.queen_charge_statistics import statistics, SEATS
from train_charge_policy import examples, fit
from scripts.rank_human_games import rank
from train_human_action import loss_gradient as action_loss_gradient


class HistoricalDataTests(unittest.TestCase):
    def test_successful_cohort_never_uses_later_outcomes(self):
        training=[{"players":[0,1,2,3],"humans":[True]*4,"split":0,
                   "hand_money":[[30,-10,-10,-10]]*4} for _ in range(3)]
        future={"players":[0,1,2,3],"humans":[True]*4,"split":2,
                "hand_money":[[-999,2997,-999,-999]]*4}
        a,ratings_a=rank(training,minimum=2)
        b,ratings_b=rank(training+[future],minimum=2)
        self.assertEqual(a,b)
        self.assertEqual(ratings_a,ratings_b)
        self.assertIn(0,a)

    def test_direct_action_gradient_preserves_decision_groups(self):
        rng=np.random.default_rng(4)
        rows=np.zeros((5,49));rows[:,7:]=rng.normal(size=(5,42))
        rows[[1,4],4]=1
        starts=np.array([0,3]);weights=rng.normal(scale=.1,size=(18,24))
        _,gradient=action_loss_gradient(rows,starts,weights,.001)
        for i,j in [(0,0),(3,7),(12,18),(17,23)]:
            a,b=weights.copy(),weights.copy();a[i,j]+=1e-5;b[i,j]-=1e-5
            numeric=(action_loss_gradient(rows,starts,a,.001)[0]-action_loss_gradient(rows,starts,b,.001)[0])/2e-5
            self.assertAlmostEqual(gradient[i,j],numeric,places=7)

    def test_charge_training_uses_first_opportunity_and_excludes_keeper(self):
        deal = {"type": "deal", "pass": "left", **{seat: [] for seat in SEATS}}
        deal["north"] = ["QS", "AS", "9S", "8S", "7S", "6S"]
        events = [{"type": "sit", "rules": "classic"}, deal,
                  {"type": "charge", "seat": "north", "cards": []},
                  {"type": "charge", "seat": "north", "cards": ["QS"]},
                  dict(deal, **{"pass": "keeper"}),
                  {"type": "charge", "seat": "north", "cards": ["QS"]}]
        game = {"id": 5, "split": 0, "humans": [True, False, False, False], "events": events}
        rows = examples([game])
        self.assertEqual(rows.tolist(), [[5, 0, 0, 6, 0]])
        probabilities, _, counts = fit(rows, 64)
        self.assertEqual(counts[0, 3].tolist(), [1, 0])
        self.assertTrue(((probabilities >= .02) & (probabilities <= .98)).all())

    def test_sanitization_removes_legacy_identity_metadata_without_mutating_source(self):
        sit = {"type": "sit", "rules": "classic", "created_by": "private-id",
               "created_time": 123, "seed": {"type": "chosen", "value": "private-seed"}}
        for seat in SEATS:
            sit[seat] = {"type": "bot", "strategy": "neural_net", "user_id": seat,
                         "name": "private-name"}
        source = [sit, {"type": "chat", "message": "private-chat"},
                  {"type": "play", "seat": "north", "card": "2C"}]
        before = copy.deepcopy(source)
        result = sanitize(source)
        self.assertEqual(source, before)
        self.assertEqual(len(result), 2)
        self.assertEqual(set(result[0]), {"type", "rules", "seed", *SEATS})
        self.assertEqual(result[0]["north"], {
            "type": "bot", "strategy": "neural_net",
            "user_id": "00000000-0000-0000-0000-000000000001"})
        self.assertNotIn("private", str(result))

    def test_charge_counts_use_post_pass_human_holdings_once_per_hand(self):
        deal = {"type": "deal", **{seat: [] for seat in SEATS}}
        deal["north"] = ["QS", "AS", "9S", "8S", "7S", "6S"]
        events = [{"type": "sit", "rules": "classic"}, deal,
                  {"type": "send_pass", "from": "north", "cards": ["QS", "AS", "9S"]},
                  {"type": "recv_pass", "to": "east", "cards": ["QS", "AS", "9S"]},
                  {"type": "charge", "seat": "east", "cards": ["QS"]},
                  {"type": "play", "seat": "south", "card": "2C"},
                  {"type": "play", "seat": "west", "card": "3C"},
                  deal, {"type": "play", "seat": "south", "card": "2C"}]
        human = {"humans": [True, True, False, False], "events": events}
        bots = {"humans": [False] * 4, "events": events}
        blind = copy.deepcopy(human)
        blind["events"][0]["rules"] = "blind"
        result = statistics([human, bots, blind])
        self.assertEqual(result["held"]["hands"], 2)
        self.assertEqual(result["held"]["spades_at_least_6"], 1)
        self.assertEqual(result["charged"]["hands"], 1)
        self.assertEqual(result["charged"]["spades_at_least_5"], 0)
        self.assertEqual(result["charged"]["ace_and_nine_spades"], 1)


if __name__ == "__main__":
    unittest.main()
