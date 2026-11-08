import unittest
import numpy as np
from train_preplay import FEATURES, HIDDEN, loss_gradient, validation_mask
from train_preplay import forward
from train_preplay_additive import CONTEXT, CHARGES, export_parameters


class PreplayTests(unittest.TestCase):
    def test_centered_money_gradient_and_offset_invariance(self):
        rng = np.random.default_rng(98)
        x = rng.normal(size=(3, 5, FEATURES))
        legal = np.arange(5)[None, :] < np.array([[2], [4], [5]])
        y = rng.normal(size=(3, 5)) * legal
        y = (y-y.sum(1, keepdims=True)/legal.sum(1, keepdims=True)) * legal
        params = [rng.normal(scale=.03, size=(FEATURES, HIDDEN)), np.ones(HIDDEN)*.1,
                  rng.normal(scale=.03, size=HIDDEN), np.zeros(1)]
        loss, grads = loss_gradient(params, x, y, legal, .001)
        for layer, index in [(0, (4, 8)), (1, (7,)), (2, (3,)), (3, (0,))]:
            a, b = [p.copy() for p in params], [p.copy() for p in params]
            a[layer][index] += 1e-5
            b[layer][index] -= 1e-5
            numeric = (loss_gradient(a, x, y, legal, .001)[0]-loss_gradient(b, x, y, legal, .001)[0])/2e-5
            self.assertAlmostEqual(grads[layer][index], numeric, places=7)
        shifted = [p.copy() for p in params]
        shifted[3] += 4
        self.assertAlmostEqual(loss, loss_gradient(shifted, x, y, legal, .001)[0], places=12)

    def test_whole_deal_split_covers_all_seats_directions_and_rules(self):
        ids = np.arange(1800000, 1806000)
        chosen = ids[validation_mask(ids)]
        self.assertEqual(set(chosen % 4), set(range(4)))
        self.assertEqual(set(chosen // 4 % 4), set(range(4)))
        self.assertEqual(set(chosen // 16 % 6), set(range(6)))

    def test_additive_charge_export_gates_each_charge_independently(self):
        rng = np.random.default_rng(28)
        coefficients = rng.normal(0, .05, (4, len(CONTEXT)))
        x = rng.random((16, FEATURES)).astype('f4')
        selected = (np.arange(16)[:, None] & (1 << np.arange(4))) != 0
        x[:, CHARGES] = selected
        expected = ((x[:, CONTEXT] @ coefficients.T) * selected).sum(1)
        actual = forward(export_parameters(coefficients), x)[1]
        np.testing.assert_allclose(actual, expected, atol=1e-5)


if __name__ == '__main__':
    unittest.main()
