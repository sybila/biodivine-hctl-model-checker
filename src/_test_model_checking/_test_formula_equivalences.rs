use crate::_test_model_checking::{MODEL_CELL_CYCLE, MODEL_CELL_DIVISION, MODEL_YEAST};
use crate::mc_utils::get_extended_symbolic_graph;
use crate::model_checking::{model_check_formula, model_check_formula_dirty};
use biodivine_lib_param_bn::BooleanNetwork;

/// Evaluate pairs of equivalent formulae on given BN model.
/// Compare whether the results are the same.
fn evaluate_equivalent_formulae(bn: BooleanNetwork) {
    // test formulae use 3 HCTL vars at most
    let stg = get_extended_symbolic_graph(&bn, 3).unwrap();

    let equivalent_formulae_pairs = vec![
        // constants
        ("true", "~ false"),
        // AU equivalence (where phi1 are attractors, and phi2 the rest)
        (
            "(~(!{x}:AG EF{x})) AU (!{x}:AG EF{x})",
            "~EG~(!{x}:AG EF{x}) & ~(~(!{x}:AG EF{x}) EU (~(!{x}:AG EF{x}) & (!{x}:AG EF{x})))",
        ),
        // AU equivalence (where phi1 are fixed points, and phi2 the rest)
        (
            "(~(!{x}:AX{x})) AU (!{x}:AX{x})",
            "~EG~(!{x}:AX{x}) & ~(~(!{x}:AX{x}) EU (~(!{x}:AX{x}) & (!{x}:AX{x})))",
        ),
        // formulae for attractors, one is evaluated directly through optimisation
        ("!{x}: AG EF {x}", "!{x}: AG EF ({x} & {x})"),
        // formulae for fixed-points, one is evaluated directly through optimisation
        ("!{x}: AX {x}", "!{x}: AX ({x} & {x})"),
        // formulae for fixed-points, but differently named variables
        ("!{x}: AX {x}", "!{y}: AX {y}"),
        // computation for one of these involves canonization and basic caching
        ("!{x}: AX {x}", "(!{x}: AX {x}) & (!{y}: AX {y})"),
        // AX equivalence
        ("!{x}: AX {x}", "!{x}: ~EX ~{x}"),
        // computation for one of these involves basic caching
        ("!{x}: ((AG EF {x}) & (AG EF {x}))", "!{x}: AG EF {x}"),
        // computation for one of these involves advanced canonized caching
        ("!{x}: !{y}: ((AG EF {x}) & (AG EF {y}))", "!{x}: AG EF {x}"),
        // different order of quantifiers
        (
            "3{x}: !{y}: ((AG EF {x}) & (AG EF {y}))",
            "!{x}: 3{y}: ((AG EF {y}) & (AG EF {x}))",
        ),
        // AF equivalence
        ("!{x}: AX AF {x}", "!{x}: AX ~EG ~{x}"),
        // steady-states in bi-stable dynamics expressed in different ways (2 vs 3 variables)
        (
            "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})",
            "!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))",
        ),
        // quantifiers
        (
            "~(3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}))",
            "V{x}: V{y}: (@{x}: {y} | ~(!{z}: AX {z})) | (@{y}: ~(!{z}: AX {z}))",
        ),
        // binder and forall equivalence v1
        ("!{x}: AG EF {x}", "V{x}: ({x} => (AG EF {x}))"),
        // binder and forall equivalence v2
        ("!{x}: AX {x}", "V{x}: ({x} => (AX {x}))"),
        // binder and forall equivalence v3
        ("!{x}: AF {x}", "V{x}: ({x} => (AF {x}))"),
    ];

    // check that the results for the two formulae are equivalent in both sanitized and
    // non-sanitized version of model checking
    for (formula1, formula2) in equivalent_formulae_pairs {
        let result1 = model_check_formula(formula1, &stg).unwrap();
        let result2 = model_check_formula(formula2, &stg).unwrap();
        assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());

        let result1 = model_check_formula_dirty(formula1, &stg).unwrap();
        let result2 = model_check_formula_dirty(formula2, &stg).unwrap();
        assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());
    }
}

#[test]
/// Test evaluation of pairs of equivalent formulae on model FISSION-YEAST-2008.
fn model_check_equivalent_formulae() {
    // bn for each of the 3 predefined models
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();
    let bns = [bn1, bn2, bn3];

    for bn in bns {
        evaluate_equivalent_formulae(bn);
    }
}
