"""Small numerical checks for the training objective and checkpoint contract."""
import tempfile
import unittest
from pathlib import Path

import numpy as np
from neural_net import FEATURES, HIDDEN, export, forward, load
from train_self_play import policy_loss_gradient


class TrainingTests(unittest.TestCase):
    def test_policy_gradient_matches_finite_differences_including_clipping(self):
        logits = np.array([[0.1, -0.3, 0.2], [0.8, -0.5, 0.0], [-0.4, 0.6, -0.2]], dtype=np.float64)
        chosen = np.eye(3)
        mask = np.array([[True, True, False], [True, True, True], [True, True, True]])
        old = np.array([[0.55, 0.45, 0], [0.1, 0.8, 0.1], [0.3, 0.6, 0.1]])
        for advantage in [np.array([1.5, -0.8, 0.7]), np.array([-1.5, 0.8, -0.7])]:
            _, gradient = policy_loss_gradient(logits, chosen, old, mask, advantage)
            for i, j in np.ndindex(logits.shape):
                plus, minus = logits.copy(), logits.copy()
                plus[i, j] += 1e-5
                minus[i, j] -= 1e-5
                numerical = (policy_loss_gradient(plus, chosen, old, mask, advantage)[0]
                             - policy_loss_gradient(minus, chosen, old, mask, advantage)[0]) / 2e-5
                self.assertAlmostEqual(gradient[i, j], numerical, places=7)
            np.testing.assert_allclose(gradient.sum(axis=1), 0, atol=1e-12)
            self.assertEqual(gradient[0, 2], 0)

    def test_checkpoint_preserves_predictions_and_rejects_corruption(self):
        rng = np.random.default_rng(55)
        params = [rng.normal(size=s).astype(np.float32) for s in [(FEATURES, HIDDEN), (HIDDEN,), (HIDDEN, 1), (1,)]]
        x = rng.normal(size=(4, FEATURES)).astype(np.float32)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "network.thp"
            export(path, params)
            np.testing.assert_array_equal(forward(x, params)[1], forward(x, load(path))[1])
            path.write_bytes(path.read_bytes()[:-1])
            with self.assertRaises(ValueError):
                load(path)
            params[0][0, 0] = np.nan
            export(path, params)
            with self.assertRaises(ValueError):
                load(path)


if __name__ == "__main__":
    unittest.main()
