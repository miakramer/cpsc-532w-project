## Q-learning Pseudo Code

# have: set of decision variables D, set of stochastic variables S
# have: number of values each decision variable can take, v(x), x in D, and number of values each stochastic variable can take, v(s), s in S
# have: T - number of "time steps", N - number of Q iterations, M - number of stages
# have: epsilon, the "greedy" probability for updating x
# have: information about which variables are in which stage, L_D(m) = [subset of decision vars] L_S(m) = [subset of stochastic vars]
# have: set of constraints, C

D_combos = all possible combinations of decision vars #assume it hase size (number of variables),(number of combos)
S_combos = all possible combinations of stochastic vars # similarly
Q0 = [rand( (size(D_combos)[1] , size(S_combos)[1]) )]*T # T copies of the initialization

def random_x(L_D(m)):
    x_random = []
    for x in L_D(m):
        sample i in categorical (number of categories given by v(x))
        x_random.append(x[i]) # i.e. pick i'th possible value x can take on as the random choice of x
    return x_random

def argmax_x(Q0,D,L_D(m)):
    # Q0 indicates the previous iterations' Q
    # need to identify Q0_m (the relevant parts of Q for this stage)
    
    return

while n <= N:
    while t <= T:
        while m <= M:
            for s in L_S(m):
                s_t = sample(s)

            Cm = set of constraints only dependent on variables of stage m (only containing x in L_D(m) and s in L_S(m))

            # Naomi's idea would fit in here

            # update x
            sample p in uniform(0,1)
            if p <= epsilon:
                xm_t = random_x(L_D(m))
            else:
                xm_t = 



