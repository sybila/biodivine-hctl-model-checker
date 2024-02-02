use crate::_test_model_checking::{MODEL_CELL_CYCLE, MODEL_CELL_DIVISION, MODEL_YEAST};
use crate::mc_utils::get_extended_symbolic_graph;
use crate::model_checking::{
    model_check_extended_formula, model_check_extended_formula_dirty, model_check_formula,
    model_check_formula_dirty,
};
use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

#[test]
/// Test evaluation of (very simple) extended formulae, where special propositions are
/// evaluated as various simple pre-computed sets.
fn model_check_extended_simple() {
    let bn = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

    // 1) first test, only proposition substituted
    let formula_v1 = "PleC & EF PleC";
    let sub_formula = "PleC";
    let formula_v2 = "%s% & EF %s%";

    let result_v1 = model_check_formula(formula_v1, &stg).unwrap();
    // use 'dirty' version to avoid sanitation (for BDD to retain all symbolic vars)
    let result_sub = model_check_formula_dirty(sub_formula, &stg).unwrap();
    let context_sets = HashMap::from([("s".to_string(), result_sub)]);
    let result_v2 = model_check_extended_formula(formula_v2, &stg, &context_sets).unwrap();
    assert!(result_v1.as_bdd().iff(result_v2.as_bdd()).is_true());

    // 2) second test, disjunction substituted
    let formula_v1 = "EX (PleC | DivK)";
    let sub_formula = "PleC | DivK";
    let formula_v2 = "EX %s%";

    let result_v1 = model_check_formula(formula_v1, &stg).unwrap();
    // use 'dirty' version to avoid sanitation (for BDD to retain all symbolic vars)
    let result_sub = model_check_formula_dirty(sub_formula, &stg).unwrap();
    let context_sets = HashMap::from([("s".to_string(), result_sub)]);
    let result_v2 = model_check_extended_formula(formula_v2, &stg, &context_sets).unwrap();
    assert!(result_v1.as_bdd().iff(result_v2.as_bdd()).is_true());
}

/// Evaluate extended HCTL formulae, in which `wild-card properties` can represent already
/// pre-computed results. Compare with the equivalent computation that does the whole computation
/// in one step (without semantic substitution).
fn model_check_extended_complex_on_bn(bn: BooleanNetwork) {
    let stg = get_extended_symbolic_graph(&bn, 3).unwrap();

    // the test is conducted on two different formulae

    // first define and evaluate the two formulae normally in one step
    let formula1 = "!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))";
    let formula2 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))";
    let result1 = model_check_formula(formula1, &stg).unwrap();
    let result2 = model_check_formula(formula2, &stg).unwrap();

    // now precompute part of the formula, and then substitute it as `wild-card proposition`
    let substitution_formula = "(!{z}: AX {z})";
    // we must use 'dirty' version to avoid sanitation (BDDs must retain all symbolic vars)
    let raw_set = model_check_formula_dirty(substitution_formula, &stg).unwrap();
    let context_sets = HashMap::from([("subst".to_string(), raw_set)]);

    let formula1_v2 = "!{x}: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%)";
    let formula2_v2 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & %subst%) & EF ({y} & %subst%) & AX (EF ({x} & %subst%) ^ EF ({y} & %subst%))";
    let result1_v2 = model_check_extended_formula(formula1_v2, &stg, &context_sets).unwrap();
    let result2_v2 = model_check_extended_formula(formula2_v2, &stg, &context_sets).unwrap();

    assert!(result1.as_bdd().iff(result1_v2.as_bdd()).is_true());
    assert!(result2.as_bdd().iff(result2_v2.as_bdd()).is_true());

    // also double check that running "extended" evaluation on the original formula (without
    // wild-card propositions) is the same as running the standard variant
    let empty_context = HashMap::new();
    let result1_v2 = model_check_extended_formula(formula1, &stg, &empty_context).unwrap();
    let result2_v2 = model_check_extended_formula(formula2, &stg, &empty_context).unwrap();
    assert!(result1.as_bdd().iff(result1_v2.as_bdd()).is_true());
    assert!(result2.as_bdd().iff(result2_v2.as_bdd()).is_true());
}

#[test]
/// Test evaluation of extended HCTL formulae, in which `wild-card properties` can
/// represent already pre-computed results. Use all 3 pre-defined models.
fn model_check_extended_complex() {
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();

    model_check_extended_complex_on_bn(bn1);
    model_check_extended_complex_on_bn(bn2);
    model_check_extended_complex_on_bn(bn3);
}

/// Test checking tautology or contradiction formulae that contain both wild-card properties
/// and variable domains.
fn model_check_extended_tautologies_on_bn(bn: BooleanNetwork) {
    let formula = "!{x}: AG EF {x}";
    let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

    // we must use 'dirty' version to avoid sanitation (BDDs must retain all symbolic vars)
    let raw_set = model_check_formula_dirty(formula, &stg).unwrap();

    let formulas = [
        "V{x} in %1%: @{x}: (%1% & %2%)",
        "V{x} in %1%: @{x}: (%2% & %3%)",
        "V{x} in %2%: @{x}: (%2% & %3%)",
    ];

    // context with three entries of the same set
    let context = HashMap::from([
        ("1".to_string(), raw_set.clone()),
        ("2".to_string(), raw_set.clone()),
        ("3".to_string(), raw_set.clone()),
    ]);

    for f in formulas {
        let result = model_check_extended_formula_dirty(f, &stg, &context).unwrap();
        assert!(result
            .as_bdd()
            .iff(stg.unit_colored_vertices().as_bdd())
            .is_true());
    }
}

#[test]
/// Test checking tautology or contradiction formulae that contain both wild-card properties
/// and variable domains. Use all 3 pre-defined models.
fn model_check_extended_tautologies() {
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();

    model_check_extended_tautologies_on_bn(bn1);
    model_check_extended_tautologies_on_bn(bn2);
    model_check_extended_tautologies_on_bn(bn3);
}
