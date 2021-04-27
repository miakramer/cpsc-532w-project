## Q-learning Pseudo Code

# have: set of decision variables D, set of stochastic variables S
# have: number of values each decision variable can take, v(x), x in D, and number of values each stochastic variable can take, v(s), s in S
# have: T - number of "time steps", N - number of Q iterations, M - number of stages
# have: epsilon, the "greedy" probability for updating x
# have: information about which variables are in which stage, L_D(m) = [subset of decision vars] L_S(m) = [subset of stochastic vars]
# have: set of constraints, C

D_combos = all possible combinations of decision vars values #assume it hase size (number of variables),(number of combos), in python I know there's a numpy function for this
S_combos = all possible combinations of stochastic vars values # similarly
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
    Dm_combos = all possible combinations of active decision vars values # assume this is like above, each row is the value taken on by a decision variable, each column is a different combination of these values
    Sm_combos = all possible combinations of active stochastic vars values
    Qm_x = zeros( (size(Dm_combos)[1],size(S_combos)[1]) ) # Qm is initialized in these two steps in order to handle the summing that needs to happen. first step is an array with the number of columns matching the number of combinations of active decision vars, and the number of rows matching the number of rows of all stochastic vars
    Qm = zeros( (size(Dm_combos)[1],size(Sm_combos)[1]) ) # second step has number of columns same as combinations of active decision vars, and number of rows same as combinations of active stoch vars

    j = 0
    for combo in Dm_combos:
        search D_combos for all columns containing combo and return the indices as ind_x
        Qm_x[:,j] = sum of colums of Q given by ind_x # sum columns of Q given by ind_x and store in column j of Qm for this  
        j = j+1

    j = 0
    Sm_combos = all possible combinations of active stochastic vars values # likewise
    for combo in Sm_combos:
        search S_combos for all rows combo and return the indices as ind_s
        Qm[j,:] = sum of rows of Qm_x given by ind_s 
        j = j+1

    

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



