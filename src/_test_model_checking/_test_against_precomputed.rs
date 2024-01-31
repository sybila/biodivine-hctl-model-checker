use crate::_test_model_checking::{MODEL_CELL_CYCLE, MODEL_CELL_DIVISION, MODEL_YEAST};
use crate::mc_utils::get_extended_symbolic_graph;
use crate::model_checking::{model_check_formula, model_check_formula_dirty};
use biodivine_lib_param_bn::BooleanNetwork;

/// Run the evaluation for the set of given formulae on a given model.
/// Compare the result numbers with the given expected numbers.
/// The `test_tuples` consist of tuples <formula, num_total, num_colors, num_states>.
fn compare_mc_results_with_expected(test_tuples: Vec<(&str, f64, f64, f64)>, bn: BooleanNetwork) {
    // test formulae use 3 HCTL vars at most
    let stg = get_extended_symbolic_graph(&bn, 3).unwrap();

    for (formula, num_total, num_colors, num_states) in test_tuples {
        let result = model_check_formula(formula.to_string(), &stg).unwrap();
        assert_eq!(num_total, result.approx_cardinality());
        assert_eq!(num_colors, result.colors().approx_cardinality());
        assert_eq!(num_states, result.vertices().approx_cardinality());

        let result = model_check_formula_dirty(formula.to_string(), &stg).unwrap();
        assert_eq!(num_total, result.approx_cardinality());
        assert_eq!(num_colors, result.colors().approx_cardinality());
        assert_eq!(num_states, result.vertices().approx_cardinality());
    }
}

#[test]
/// Test evaluation of several important formulae on model FISSION-YEAST-2008.
/// Compare numbers of results with the numbers acquired by Python model checker or AEON.
fn model_check_basic_yeast() {
    // tuples consisting of <formula, num_total, num_colors, num_states>
    // num_x are numbers of expected results
    let test_tuples = vec![
        ("!{x}: AG EF {x}", 12., 1., 12.),
        ("!{x}: AX {x}", 12., 1., 12.),
        ("!{x}: AX EF {x}", 68., 1., 68.),
        ("AF (!{x}: AX {x})", 60., 1., 60.),
        ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", 12., 1., 12.),
        ("3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))", 11., 1., 11.),
        ("!{x}: (AX (AF {x}))", 12., 1., 12.),
        ("AF (!{x}: (AX (~{x} & AF {x})))", 0., 0., 0.),
        ("AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))", 0., 0., 0.),
        // TODO: more tests regarding formulae for inference using concrete observations
    ];

    // model is in bnet format
    let bn = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();
    compare_mc_results_with_expected(test_tuples, bn);
}

#[test]
/// Test evaluation of several important formulae on model MAMMALIAN-CELL-CYCLE-2006.
/// Compare numbers of results with the numbers acquired by Python model checker or AEON.
fn model_check_basic_mammal() {
    // tuples consisting of <formula, num_total, num_colors, num_states>
    // num_x are numbers of expected results
    let test_tuples = vec![
        ("!{x}: AG EF {x}", 113., 2., 113.),
        ("!{x}: AX {x}", 1., 1., 1.),
        ("!{x}: AX EF {x}", 425., 2., 425.),
        ("AF (!{x}: AX {x})", 32., 1., 32.),
        ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", 0., 0., 0.),
        ("3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))", 0., 0., 0.),
        ("!{x}: (AX (AF {x}))", 1., 1., 1.),
        ("AF (!{x}: (AX (~{x} & AF {x})))", 0., 0., 0.),
        ("AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))", 0., 0., 0.),
        // TODO: more tests regarding formulae for inference using concrete observations
    ];

    // model is in bnet format
    let bn = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    compare_mc_results_with_expected(test_tuples, bn);
}

#[test]
/// Test evaluation of several important formulae on model ASYMMETRIC-CELL-DIVISION-B.
/// Compare numbers of results with the numbers acquired by Python model checker or AEON.
fn model_check_basic_cell_division() {
    // tuples consisting of <formula, num_total, num_colors, num_states>
    // num_x are numbers of expected results
    let test_tuples = vec![
        ("!{x}: AG EF {x}", 1097728., 65536., 512.),
        ("!{x}: AX {x}", 65536., 53248., 64.),
        ("!{x}: AX EF {x}", 1499136., 65536., 512.),
        ("AF (!{x}: AX {x})", 21430272., 53248., 512.),
        ("!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})", 24576., 12288., 64.),
        ("3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))", 24576., 12288., 48.),
        ("!{x}: (AX (AF {x}))", 84992., 59392., 112.),
        ("AF (!{x}: (AX (~{x} & AF {x})))", 49152., 6144., 128.),
        ("AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))", 28672., 3584., 128.),
        // TODO: more tests regarding formulae for inference using concrete observations
    ];

    // model is in aeon format
    let bn = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    compare_mc_results_with_expected(test_tuples, bn);
}
