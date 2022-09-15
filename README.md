# Symbolic model checker for logic HCTL written in RUST

Work in progress.

This repository contains the Rust implementation of the symbolic model checker for hybrid logic HCTL. The tool is focused on the analysis of (partially specified) Boolean networks. In particular, it allows to check for any behavioural hypotheses expressible in HCTL on large, non-trivial networks. This includes properties like stability, bi-stability, attractors, or oscillatory behaviour.

For a given Boolean network (with inputs) and HCTL formula (representing the property we want to check), it computes all the states of the network (and corresponding colours) that satisfy the formula. Depending on the mode, either prints the numbers of satisfying states and colours, or prints all the satisfying assignments.

To directly invoke the model checker, compile the code using
```
cargo build --release
```
and then run the binary:
```
.\target\release\model-check model_file hctl_formula
```

- `model_file` is a path to a file with BN model in bnet format
- `hctl_formula` is a valid HCTL formula in correct format

