## Q-learning Pseudo Code

# have: set of decision variables D, set of stochastic variables S
# have: number of values each decision variable can take, v(x), x in D, and number of values each stochastic variable can take, v(s), s in S
# have: T - number of "time steps", N - number of Q iterations, M - number of stages
# have: epsilon, the "greedy" probability for updating x
# have: information about which variables are in which stage, L_D(m) = [subset of decision vars] L_S(m) = [subset of stochastic vars]
# have: set of constraints, C
# have: gamma, multiplier in the qhat update corresponding to how influential the previous Q is

D_combos = all possible combinations of decision vars values #assume it hase size (number of variables),(number of combos), in python I know there's a numpy function for this
S_combos = all possible combinations of stochastic vars values # similarly
Q0 = [rand( (size(D_combos)[1] , size(S_combos)[1]) )]*T # T copies of the initialization

def random_x(L_D(m)):
    x_random = []
    for x in L_D(m):
        sample i in categorical (number of categories given by v(x))
        x_random.append(x[i]) # i.e. pick i'th possible value x can take on as the random choice of x
    return x_random

def stage_argmax_x(Q0,D,L_D(m),S,L_S(m),s_t):
    # Q0 indicates the previous iterations' Q
    # need to identify Q0_m (the relevant parts of Q for this stage)
    Dm_combos = all possible combinations of active decision vars values # assume this is like above, each row is the value taken on by a decision variable, each column is a different combination of these values
    Sm_combos = all possible combinations of active stochastic vars values
    Qm_x = zeros( (size(Dm_combos)[1],size(S_combos)[1]) ) # Qm is initialized in these two steps in order to handle the summing that needs to happen. first step is an array with the number of columns matching the number of combinations of active decision vars, and the number of rows matching the number of rows of all stochastic vars
    Qm = zeros( (size(Dm_combos)[1],size(Sm_combos)[1]) ) # second step has number of columns same as combinations of active decision vars, and number of rows same as combinations of active stoch vars

    # find Qm 
    j = 0
    for combo in Dm_combos:
        search D_combos for all columns containing combo and return the indices as ind_x
        Qm_x[:,j] = sum of colums of Q0 given by ind_x # sum columns of Q given by ind_x and store in column j of Qm for this  
        j = j+1

    j = 0
    Sm_combos = all possible combinations of active stochastic vars values # likewise
    for combo in Sm_combos:
        search S_combos for all rows combo and return the indices as ind_s
        Qm[j,:] = sum of rows of Qm_x given by ind_s 
        j = j+1

    # compute argmax
    s_t_ind = find index in Sm_combos corresponding to the combination of sampled values of our active stochastic variables
    ind_max = find index of max value of Qm[s_t_ind,:] # search the row corresponding to s_t_ind for the max value

    return Dm_combos[ind_max] # returns the values of the decision vars corresponding to the max entry in Qm

def max_x(Q,S_combos,s):
    ind_s = find index in S_combos corresponding to the combination of sampled stochastic variable values in s
    q_max = max value of Q[ind_s,:]

    return q_max


while n <= N:
    sn = sample all stochastic variables T times
    while t <= T:
        x_t = empty array
        s_t = sn[t]
        while m <= M:
            sm_t = []
            for s in L_S(m):
                sm_t.append(s_t[s]) # poor notation, but basically says store sampled values of all active stochastic variables in sm_t

            Cm = set of constraints only dependent on variables of stage m (only containing x in L_D(m) and s in L_S(m))

            # Naomi's idea would fit in here

            # update x
            sample p in uniform(0,1)
            if p <= epsilon:
                xm_t = random_x(L_D(m))
            else:
                xm_t = stage_argmax_x(Q0,D,L_D(m),S,L_S(m),sm_t)

            x_t[active decision variables] = xm_t # store the decision variables for this stage
            m = m+1
        
        # now x_t is fully defined
        qhat_t = reward(x_t,s_t) + gamma * max_x(Q0,S_combos,sn[t+1])

        # update Q:
        decision_ind = find index in D_combos corresponding to combo of decision variable values in x_t
        stochastic_ind = find index in S_combos corresponding to combo of stochastic variable values in s_t
        Q1[decision_ind,stochastic_ind] = (1-alpha_n) * Q0[decision_ind,stochastic_ind] + alpha_n * qhat_t # here, Q(x_t,s_t) is the entry of Q corresponding to the combinations of decision and stochastic variable values that we have in x_t,s_t 

    update Q0 = Q1
    


