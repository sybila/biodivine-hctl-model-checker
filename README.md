[![Crates.io](https://img.shields.io/crates/v/biodivine-hctl-model-checker?style=flat-square)](https://crates.io/crates/biodivine-hctl-model-checker)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/biodivine-hctl-model-checker/)
[![Continuous integration](https://img.shields.io/github/actions/workflow/status/sybila/biodivine-hctl-model-checker/build.yml?branch=master&style=flat-square)](https://github.com/sybila/biodivine-hctl-model-checker/actions?query=workflow%3Abuild)
[![Coverage](https://img.shields.io/codecov/c/github/sybila/biodivine-hctl-model-checker?style=flat-square)](https://app.codecov.io/gh/sybila/biodivine-hctl-model-checker)
[![GitHub last commit](https://img.shields.io/github/last-commit/sybila/biodivine-hctl-model-checker?style=flat-square)](https://github.com/sybila/biodivine-hctl-model-checker/commits/master)
[![Crates.io](https://img.shields.io/crates/l/biodivine-hctl-model-checker?style=flat-square)](https://github.com/sybila/biodivine-hctl-model-checker/blob/master/LICENSE)


# Symbolic model checker for logic HCTL written in RUST

This repository contains the Rust implementation of the symbolic model checker for hybrid logic HCTL. 
The method is focused on the analysis of (partially specified) Boolean networks. 
In particular, it allows to check for any behavioural hypotheses expressible in HCTL on large, non-trivial networks. 
This includes properties like stability, bi-stability, attractors, or oscillatory behaviour.

## Prerequisites

To run the model checker, you will need the Rust compiler.
We recommend following the instructions on [rustlang.org](https://www.rust-lang.org/learn/get-started).

If you are not familiar with Rust, there are also Python bindings for most of the important functionality in [AEON.py](https://github.com/sybila/biodivine-aeon-py).

## Functionality

This repository encompasses the CLI model-checking tool, and the model-checking library.

### Model-checking tool

Given a (partially defined) Boolean network and HCTL formulae (encoding properties we want to check), the tool computes all the states of the network (and corresponding parametrizations) that satisfy the formula.
Currently, there is only a command-line interface, with a GUI soon to be implemented.
Depending on the mode, the program can either print the numbers of satisfying states and colours, or print all the satisfying assignments.

To directly invoke the model checker, compile the code using
```
cargo build --release
```
and then run the binary:
```
.\target\release\hctl-model-checker <MODEL_PATH> <FORMULAE_PATH> [-m <MODEL_FORMAT>] [-p <PRINT_OPTION>] [-h]
```

- `MODEL_PATH` is a path to a file with BN model in selected format (see below, `aeon` is default)
- `FORMULAE_PATH` is path to a file with a set of valid HCTL formulae (one per line)
- `PRINT_OPTION` is one of `no-print`/`summary`/`with-progress`/`exhaustive` and defines the amount of information on the output (`summary` is a default mode)
- `MODEL_FORMAT` is one of `aeon`/`bnet`/`smbl` and defines the input format (`aeon` is default)

For more help, use option `-h` or `--help`.

### Library

This package also offers an API for utilizing the model-checking functionality.
The most relevant high-level functionality can be found in modules `analysis` and `model_checking`.
Further, useful functionality and structures regarding parsing (parser, tokenizer, syntactic trees) is in `preprocessing` module.

## Model formats

The model checker takes BN models in `aeon` format as its default input, with many example models present in the `benchmark_models` directory.
You can also use `sbml` and `bnet` models by specifying the format as a CLI option (see above).

## HCTL formulae

All formulae used must not contain free variables.
In the input file, there has to be one formula in a correct format per line.

Several interesting formulae are listed in the ```benchmark_formulae.txt``` file.

To create custom formulae, you can use any HCTL operators and many derived ones.
We use the following syntax:
* constants: `true`/`True`/`1`, `false`/`False`/`0`
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

#### Wild-card properties

The library also provides functions to model check extended formulae that contain so called "wild-card propositions".
These special propositions are evaluated as an arbitrary (coloured) set given by the user.
This allows the re-use of already pre-computed results in subsequent computations. 
In formulae, the syntax of these propositions is `%property_name%`.