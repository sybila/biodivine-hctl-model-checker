# Symbolic model checker for logic HCTL written in RUST

Work in progress.

This repository contains the Rust implementation of the symbolic model checker for hybrid logic HCTL. The tool is focused on the analysis of (partially specified) Boolean networks. In particular, it allows to check for any behavioural hypotheses expressible in HCTL on large, non-trivial networks. This includes properties like stability, bi-stability, attractors, or oscillatory behaviour.

For a given Boolean network (with inputs) and HCTL formula (representing the property we want to check), it computes all the states of the network (and corresponding colours) that satisfy the formula. Depending on the mode, either prints the numbers of satisfying states and colours, or prints all the satisfying assignments.

Currently, there is only a command-line interface, with a GUI soon to be implemented.
To directly invoke the model checker, compile the code using
```
cargo build --release
```
and then run the binary:
```
.\target\release\model-check <MODEL_PATH> <FORMULA> [-p <PRINT_OPTION>]
```

- `MODEL_PATH` is a path to a file with BN model in aeon format
- `FORMULA` is a valid HCTL formula without free variables in correct format
- `PRINT_OPTION` is one of none/short/full and defines the output mode


## Models

The tool takes BN models in `aeon` format as its input, with many example models present in the `benchmark_models` directory.
Many benchmark models are also present in bnet format, but this is just for the convenience of the user, and those can't be used as inputs for now.


## HCTL formulae

Some interesting formulae can be found in the ```benchmark_formulae.txt``` file.

To create custom formulae, use the following syntax rules (&phi; and &psi; can be arbitrary sub-formulae):

| HCTL        | Our syntax         |
|:------------|:-------------------|
| True        | true               |
| False       | false              |
| negation	   | ~&phi;             |
| Conjunction | &phi; & &psi;      |
| Disjunction | &phi; &#124; &psi; |
| Implication | &phi; => &psi;     |
| Equivalence | &phi; <=> &psi;    |
| Xor         | &phi; ^ &psi;      |
| AX          | AX &phi;           |
| EX          | EX &phi;           |
| AF          | AF &phi;           |
| EF          | EF &phi;           |
| AG          | AG &phi;           |
| EG          | EG &phi;           |
| AU          | &phi; AU &psi;     |
| EU          | &phi; EU &psi;     |
| AW          | &phi; AW &psi;     |
| EW          | &phi; EW &psi;     |
| bind x      | !{x}: &phi;        |
| jump x      | @{x}: &phi;        |
| exists x    | 3{x}: &phi;        |

We are using following operator precedence:
* unary operators (negation + temporal): 1
* binary temporal operators: 2
* boolean binary operators: and=3, xor=4, or=5, imp=6, eq=7
* hybrid operators: 8

Note that propositions can contain alphanumeric characters and underscores. Variables use the same characters, but enclosed in "{}".
* proposition example: p_1
* variable example: {x_1}
