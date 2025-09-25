use crate::_test_model_checking::_test_util::make_random_boolean_trees;
use crate::_test_model_checking::{
    MODEL_CELL_CYCLE, MODEL_CELL_DIVISION, MODEL_YEAST, NUM_FUZZING_CASES,
};
use crate::evaluation::LabelToSetMap;
use crate::mc_utils::get_extended_symbolic_graph;
use crate::model_checking::{model_check_extended_formula, model_check_tree_dirty};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use std::collections::HashMap;

/// Test evaluation of pairs of equivalent pattern formulae on given BN model.
/// The patterns (wild-card propositions) are evaluated to raw sets specified by `context`.
fn evaluate_equivalent_formulae_in_context(stg: &SymbolicAsyncGraph, context_sets: LabelToSetMap) {
    let equivalent_pattern_pairs = vec![
        // AU equivalence
        (
            "(%p1%) AU (%p2%)",
            "~EG~(%p2%) & ~(~%p2% EU (~%p2% & ~%p1%))",
        ),
        // AX equivalence
        ("AX %p1%", "~EX ~%p1%"),
        // AF equivalence
        ("AF %p1%", "~EG ~%p1%"),
        // quantifiers equivalence
        ("~(3{x}: @{x}: %p1%)", "V{x}: @{x}: ~%p1%"),
        // binder and forall equivalence v1
        ("!{x}: %p1%", "V{x}: ({x} => %p1%)"),
    ];

    for (f1, f2) in equivalent_pattern_pairs {
        let result1 = model_check_extended_formula(f1, stg, &context_sets).unwrap();
        let result2 = model_check_extended_formula(f2, stg, &context_sets).unwrap();
        assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());
    }
}

#[test]
/// Test evaluation of pairs of equivalent pattern formulae on all 3 pre-defined models.
/// Create several versions of the pattern formulae (by substituting random expressions).
fn model_check_equivalent_patterns() {
    // the 3 predefined models
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();
    let bns = [bn1, bn2, bn3];

    for bn in bns {
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // make two sets of random boolean expression trees (we need context for 2 wild-cards)

        // for now, the number of tree is low (to make github action tests swift), but it was tested on larger set
        let num_test_trees = NUM_FUZZING_CASES;
        let height_test_trees = 4;
        let seed_1 = 100000;
        let seed_2 = 200000;
        let random_trees_1 =
            make_random_boolean_trees(num_test_trees, height_test_trees, &bn, seed_1);
        let random_trees_2 =
            make_random_boolean_trees(num_test_trees, height_test_trees, &bn, seed_2);

        for (tree_1, tree_2) in random_trees_1.into_iter().zip(random_trees_2) {
            // we must use 'dirty' version to avoid sanitation (BDDs must retain all symbolic vars)
            let raw_set_1 = model_check_tree_dirty(tree_1, &stg).unwrap();
            let raw_set_2 = model_check_tree_dirty(tree_2, &stg).unwrap();
            let context = HashMap::from([
                ("p1".to_string(), raw_set_1.clone()),
                ("p2".to_string(), raw_set_2.clone()),
            ]);
            evaluate_equivalent_formulae_in_context(&stg, context);
        }
    }
}
