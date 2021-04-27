## Q-learning Pseudo Code

# have: set of decision variables D, set of stochastic variables S
# have: number of values each decision variable can take, v(x), x in D, and number of values each stochastic variable can take, v(s), s in S 
# have: N - number of Q iterations, M - number of stages (time steps)
# have: epsilon, the "greedy" probability for updating x
# have: information about which variables are in which stage, L_D(m) = [subset of decision vars] L_S(m) = [subset of stochastic vars]
# have: set of constraints, C
# have: gamma, multiplier in the qhat update corresponding to how influential the previous Q is
# have: objective function, Obj - if no objective function is given, set it by default to 1. Note, for the reward function and Q updates to work properly, objective functions need to be maximisations (if theyr'e minimisation in the input, modify it to an equivalent maximisation)

D_combos = all possible combinations of decision vars values # a list of tuples containing all combinations of values
S_combos = all possible combinations of stochastic vars values # similarly
num_S = number of combinations in S_combos
Q0 = [zeros( (size(D_combos)[1] , size(S_combos)[1]) )] # T copies of the initialization

def random_x(L_D(m)):
    x_random = []
    for x in L_D(m):
        sample i in categorical (number of categories given by v(x))
        x_random.append(x[i]) # i.e. pick i'th possible value x can take on as the random choice of x
    return x_random

def stage_argmax_x(Q0,D_combos,L_D(m),S_combos,L_S(m),s):
    """ returns the combination of decision variable values of the active decision variables corresponding to the maximum Q0 value given the sampling of stochastic variables s_t """
    # Q0 indicates the previous iterations' Q
    # need to identify Q0_m (the relevant parts of Q for this stage)
    Dm_combos = all possible combinations of active decision vars values # a list of tuples containing all combinations of values 
    num_Dm = number of combinations in Dm_combos
    Sm_combos = all possible combinations of active stochastic vars values
    num_Sm = number of combinations in Sm_combos

    Qm_x = zeros( (num_Dm,num_S) ) # Qm is initialized in these two steps in order to handle the summing that needs to happen. first step is an array with the number of columns matching the number of combinations of active decision vars, and the number of rows matching the number of rows of all stochastic vars
    Qm = zeros( (num_Dm,num_Sm) ) # second step has number of columns same as combinations of active decision vars, and number of rows same as combinations of active stoch vars

    # find Qm 
    j = 0
    for combo in Dm_combos:
        search D_combos for all columns containing combo and return the indices as ind_x
        Qm_x[:,j] = sum of colums of Q0 given by ind_x # sum columns of Q given by ind_x and store in column j of Qm for this  
        j = j+1

    j = 0
    for combo in Sm_combos:
        search S_combos for all rows containing combo and return the indices as ind_s
        Qm[j,:] = sum of rows of Qm_x given by ind_s 
        j = j+1

    # compute argmax
    s_ind = find index in Sm_combos corresponding to the combination of sampled values of our active stochastic variables s
    ind_max = find index of max value of Qm[s_ind,:] # search the row corresponding to s_t_ind for the max value

    return Dm_combos[ind_max] # returns the values of the decision vars corresponding to the max entry in Qm

def max_x(Q,D_combos,L_D(m+1),S_combos,L_S(m),s):
    """ returns the maximum value over the decision variable combinations of the row in Q corresponding to sampled stochastic variable combination s """
    
    Dm2_combos = all possible combinations of active decision vars values of stage m+1 # a list of tuples containing all combinations of values 
    num_Dm2 = number of combinations in Dm2_combos
    Sm_combos = all possible combinations of active stochastic vars values
    num_Sm = number of combinations in Sm_combos

    Qm2_x = zeros( (num_Dm2,num_S) ) # Qm is initialized in these two steps in order to handle the summing that needs to happen. first step is an array with the number of columns matching the number of combinations of active decision vars, and the number of rows matching the number of rows of all stochastic vars
    Qm2 = zeros( (num_Dm2,num_Sm) ) # second step has number of columns same as combinations of active decision vars, and number of rows same as combinations of active stoch vars

    # find Qm 2
    j = 0
    for combo in Dm_combos:
        search D_combos for all columns containing combo and return the indices as ind_x
        Qm2_x[:,j] = sum of colums of Q0 given by ind_x # sum columns of Q given by ind_x and store in column j of Qm for this  
        j = j+1

    j = 0
    for combo in Sm_combos:
        search S_combos for all rows containing combo and return the indices as ind_s
        Qm2[j,:] = sum of rows of Qm2_x given by ind_s 
        j = j+1

    ind = find index in S_combos corresponding to the combination of sampled stochastic variable values in s
    q_max = max value of Qm2[ind_s,:]

    return q_max

def reward(C,x,s):
    """ returns 0 if at least one constraint is NOT satisfied, else if ALL constraints are satisfied it returns the value of the objective function """
    for c in C: #loop through all constraints in C
        if c(x,s) not satisfied: 
            return 0 # returns 0 if at least one constraint isn't satisfied
    return 1 # returns 1 if all the stage-relevant constraints are satisfied

while n <= N:
    qhat = 0
    while m <= M:
        Cm = subset of the constraints C that only depend on variables from stage m or earlier (list?)
        sm = []
        for s in L_S(m):
            sm.append(sample(s)) # poor notation, but basically says store sampled values of all active stochastic variables in sm_t

        # update x
        sample p in uniform_continuous(0,1)
        if p <= epsilon:
            xm = random_x(L_D(m))
        else:
            xm = stage_argmax_x(Q0,D_combos,L_D(m),S_combos,L_S(m),sm)

        x[active decision variables] = xm # store the decision variables for this stage
        s[active stochastic variables] = sm # store the stochastic variables for this stage
        
        # now x_t is fully defined
        qhat = qhat + reward(Cm,xm,sm) + gamma * max_x(Q0,D_combos,L_D(m+1),S_combos,L_S(m),sm))
        m = m+1

    # update qhat to contain information about objective from final assignment of decision and stochastic variables
    qhat = qhat + Obj(x,s)

    # update Q:
    decision_ind = find index in D_combos corresponding to combo of decision variable values in x
    stochastic_ind = find index in S_combos corresponding to combo of stochastic variable values in s
    Q1[decision_ind,stochastic_ind] = (1-alpha_n) * Q0[decision_ind,stochastic_ind] + alpha_n * qhat 

    update Q0 = Q1
    n = n+1


