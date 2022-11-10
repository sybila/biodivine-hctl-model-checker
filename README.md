# Symbolic model checker for logic HCTL written in RUST

This repository contains the Rust implementation of the symbolic model checker for hybrid logic HCTL. 
The tool is focused on the analysis of (partially specified) Boolean networks. 
In particular, it allows to check for any behavioural hypotheses expressible in HCTL on large, non-trivial networks. 
This includes properties like stability, bi-stability, attractors, or oscillatory behaviour.

For a given (partially defined) Boolean network and a HCTL formula (representing the property we want to check), it computes all the states of the network (and corresponding colours) that satisfy the formula. 
Currently, there is only a command-line interface, with a GUI soon to be implemented. 
Depending on the mode, the program can either print the numbers of satisfying states and colours, or print all the satisfying assignments. 

To directly invoke the model checker, compile the code using
```
cargo build --release
```
and then run the binary:
```
.\target\release\model-check <MODEL_PATH> <FORMULA> [-m <MODEL_FORMAT>] [-p <PRINT_OPTION>]
```

- `MODEL_PATH` is a path to a file with BN model in aeon format
- `FORMULA` is a valid HCTL formula without free variables in correct format
- `PRINT_OPTION` is one of none/short/full and defines the output mode (short is default)
- `MODEL_FORMAT` is one of aeon/bnet/smbl and defines the input format (aeon is default)

For more help, run:
```
.\target\release\model-check --help
```

## Models

The tool takes BN models in `aeon` format as its default input, with many example models present in the `benchmark_models` directory.
You can also use `sbml` and `bnet` models by specifying the format as a CLI option (see above).


## HCTL formulae

Several interesting formulae are listed in the ```benchmark_formulae.txt``` file.

To create custom formulae, you can use any HCTL operators and many derived ones.
We use the following syntax:
* constants: `true`, `false`
* propositions: `alphanumeric characters and underscores` (e.g. `p_1`)
* variables: `alphanumeric characters and underscores enclosed in "{}"` (e.g. `{x_1}`)
* negation: `~`
* boolean binary operators: `&`, `|`, `=>`, `<=>`, `^`
* temporal unary operators: `AX`, `EX`, `AF`, `EF`, `AG`, `EG`
* temporal binary operators: `AU`, `EU`, `AW`, `EW`
* hybrid operators
  * bind x: `!{x}:`
  * jump x: `@{x}:`
  * exists x: `3{x}:`
  * forall x: `V{x}:`
* parentheses: `(`, `)`

The operator precedence is following (the lower, the stronger):
* unary operators (negation + temporal): 1
* binary temporal operators: 2
* boolean binary operators: and=3, xor=4, or=5, imp=6, eq=7
* hybrid operators: 8

However, it is strongly recommended to use parentheses wherever possible to prevent any parsing issues.
