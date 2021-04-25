The code is split into three parts:

- `compiler`, which compiles from the FOPPL-ish language to an RL form
- `evaluator`, which consumes the RL form and produces a policy
- `common`, with code common to the two

Compiler has the following steps:

- Parse the FOPPL-ish language
- Desugar it
- Partially evaluate it to graph intermediate form
- Compile to the RL intermediate form
- Serialize to a storage format


For the partial evaluation, we do the following steps:

- Gather all stochastic & decision variables and their possible values / distributions, respectively
- Gather dependency graph
- Gather constraints
