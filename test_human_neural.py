import unittest
import numpy as np
from train_human_neural import loss_gradient


class HumanNeuralTests(unittest.TestCase):
    def test_policy_and_centered_payoff_gradients(self):
        rng = np.random.default_rng(8)
        x = rng.normal(size=(3,18))
        ids = rng.integers(0,52,size=(3,13))
        a = rng.normal(size=(3,13,28))
        legal = np.arange(13)[None,:] < np.array([[3],[5],[8]])
        params = [rng.normal(scale=.1,size=(18,32)), np.ones(32)*.2,
                  rng.normal(scale=.1,size=(32,80)), np.zeros(80)]
        chosen = np.array([1,2,0])
        targets = np.where(legal,rng.normal(scale=.3,size=(3,13)),0)
        targets = np.where(legal,targets-targets.sum(1,keepdims=True)/legal.sum(1,keepdims=True),0)
        for target in [None,targets]:
            _, grads = loss_gradient(params,x,ids,a,legal,chosen,.001,target)
            for layer,index in [(0,(3,7)),(1,(4,)),(2,(7,19)),(3,(30,))]:
                plus,minus = [p.copy() for p in params],[p.copy() for p in params]
                plus[layer][index]+=1e-5;minus[layer][index]-=1e-5
                numeric=(loss_gradient(plus,x,ids,a,legal,chosen,.001,target)[0]-loss_gradient(minus,x,ids,a,legal,chosen,.001,target)[0])/2e-5
                self.assertAlmostEqual(grads[layer][index],numeric,places=7)
        shifted=[p.copy() for p in params];shifted[3][28:]+=3
        self.assertAlmostEqual(loss_gradient(params,x,ids,a,legal,chosen,targets=targets)[0],
                               loss_gradient(shifted,x,ids,a,legal,chosen,targets=targets)[0],places=12)


if __name__=='__main__': unittest.main()
