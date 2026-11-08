import unittest
import numpy as np
from train_human_policy import loss_gradient
from scripts.export_human_games import sanitize


class HumanPolicyTests(unittest.TestCase):
    def test_mixture_gradient_matches_finite_differences(self):
        rng = np.random.default_rng(123)
        x, w = rng.normal(size=(7, 8)), rng.normal(size=(5, 8))
        likelihood = rng.uniform(size=(7, 5))
        likelihood[0, 1:] = 0
        _, gradient = loss_gradient(x, likelihood, w)
        for idx in np.ndindex(w.shape):
            plus, minus = w.copy(), w.copy()
            plus[idx] += 1e-5
            minus[idx] -= 1e-5
            numeric = (loss_gradient(x, likelihood, plus)[0]-loss_gradient(x, likelihood, minus)[0])/2e-5
            self.assertAlmostEqual(gradient[idx], numeric, places=7)

    def test_export_removes_identity_chat_and_original_seed(self):
        sit = {"type": "sit", "seed": {"type": "chosen", "value": "private"}}
        for s in ["north", "east", "south", "west"]:
            sit[s] = {"type": "human", "user_id": "private"}
        events = sanitize([sit, {"type": "chat", "message": "private"}])
        self.assertNotIn("private", str(events))
        self.assertEqual(len(events), 1)
        self.assertEqual(sit["north"]["user_id"], "private")


if __name__ == "__main__":
    unittest.main()
