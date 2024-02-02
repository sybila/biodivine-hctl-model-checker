use crate::_test_model_checking::_test_util::make_random_boolean_trees;
use crate::_test_model_checking::{
    MODEL_CELL_CYCLE, MODEL_CELL_DIVISION, MODEL_YEAST, NUM_FUZZING_CASES,
};
use crate::mc_utils::get_extended_symbolic_graph;
use crate::model_checking::{
    model_check_extended_formula, model_check_formula, model_check_tree_dirty,
};
use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;

#[test]
/// Test evaluation of extended HCTL formulae with restricted domains of quantified variables.
/// Consider basic equivalences of formulae with one domain. Use all 3 pre-defined models.
fn model_check_with_domains() {
    // bn for each of the 3 predefined models
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();
    let bns = [bn1, bn2, bn3];

    for bn in bns {
        let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

        // make a set of random boolean expression trees (will be used to get 'random' set of states for the domain)

        let num_test_trees = NUM_FUZZING_CASES;
        let height_test_trees = 4;
        let seed = 0;
        let random_boolean_trees =
            make_random_boolean_trees(num_test_trees, height_test_trees, &bn, seed);

        for tree in random_boolean_trees {
            // we must use 'dirty' version to avoid sanitation (BDDs must retain all symbolic vars)
            let raw_set = model_check_tree_dirty(tree, &stg).unwrap();

            // pairs of equivalent pattern formulae, one without using domain, the other with domain specified
            let formulae_pairs = [
                ("3{x}:@{x}: %s% & AG EF{x}", "3{x} in %s%:@{x}:AG EF{x}"),
                ("V{x}:@{x}: %s% => AG EF{x}", "V{x} in %s%:@{x}:AG EF{x}"),
                ("!{x}: %s% & (AG EF {x})", "!{x} in %s%: AG EF {x}"),
            ];

            for (f, f_with_domain) in formulae_pairs {
                // eval the variant without domain first (contains wild-card prop)
                let ctx_sets = HashMap::from([("s".to_string(), raw_set.clone())]);
                let res = model_check_extended_formula(f.to_string(), &stg, &ctx_sets).unwrap();

                // eval the variant with a domain
                let ctx_sets = HashMap::from([("s".to_string(), raw_set.clone())]);
                let res_v2 =
                    model_check_extended_formula(f_with_domain.to_string(), &stg, &ctx_sets)
                        .unwrap();
                assert!(res.as_bdd().iff(res_v2.as_bdd()).is_true());
            }
        }
    }
}

#[test]
/// Test evaluation of extended HCTL formulae, where quantified vars are given an empty domain.
/// This is an edge-case worth testing (explicitly handled during the evaluation).
fn model_check_with_empty_domain() {
    let bn = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

    // pairs of equivalent formulae, one without using domain, the other with (empty) domain specified
    let formulae_pairs = [
        ("3{x}: @{x}: false & (AX {x})", "3{x} in %s%: @{x}: AX {x}"),
        ("V{x}: @{x}: false => (AX {x})", "V{x} in %s%: @{x}: AX {x}"),
        ("!{x}: false & (AX {x})", "!{x} in %s%: (AX {x})"),
    ];

    let empty_set = stg.mk_empty_colored_vertices();
    let context = HashMap::from([("s".to_string(), empty_set)]);

    for (f, domain_f) in formulae_pairs {
        // eval the variant without domain first
        let res = model_check_formula(f.to_string(), &stg).unwrap();

        // and now the variant with empty domain
        let res_v2 = model_check_extended_formula(domain_f.to_string(), &stg, &context).unwrap();

        assert!(res.as_bdd().iff(res_v2.as_bdd()).is_true());
    }
}

#[test]
/// Test evaluation of extended HCTL formulae with restricted domains of quantified variables.
/// Consider cases with multiple domains, use all 3 pre-defined models.
fn model_check_with_domains_complex() {
    // bn for each of the 3 predefined models
    let bn1 = BooleanNetwork::try_from(MODEL_CELL_DIVISION).unwrap();
    let bn2 = BooleanNetwork::try_from_bnet(MODEL_CELL_CYCLE).unwrap();
    let bn3 = BooleanNetwork::try_from_bnet(MODEL_YEAST).unwrap();
    let bns = [bn1, bn2, bn3];

    for bn in bns {
        let stg = get_extended_symbolic_graph(&bn, 3).unwrap();

        // make two sets of random boolean expression trees (we need context for 2 wild-cards)

        // for now, the number of tree is low (to make github action tests swift), but it was tested on larger set
        let num_test_trees = 2;
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

            // pairs of equivalent pattern formulae, one without using domain, the other with domain specified
            let formulae_pairs = [
                (
                    "!{x} in %s1%: 3{y} in %s2%: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})",
                    "!{x}: 3{y}: (@{x}: %s1% & ~{y} & AX {x}) & (@{y}: %s2% & AX {y})",
                ),
                (
                    "3{x} in %s1%: 3{y} in %s2%: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})",
                    "3{x}: 3{y}: (@{x}: %s1% & ~{y} & AX {x}) & (@{y}: %s2% & AX {y})",
                ),
                (
                    "!{x} in %s1%: 3{y} in %s2%: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))",
                    "!{x}: 3{y}: (@{x}: %s1% & ~{y} & AX {x}) & (@{y}: %s2% & AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))",
                ),
                (
                    "3{x} in %s1%: 3{y} in %s2%: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))",
                    "3{x}: 3{y}: (@{x}: %s1% & ~{y} & AX {x}) & (@{y}: %s2% & AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))",
                ),
            ];

            for (f1, f2) in formulae_pairs {
                // eval the variant with a domain
                let context = HashMap::from([
                    ("s1".to_string(), raw_set_1.clone()),
                    ("s2".to_string(), raw_set_2.clone()),
                ]);
                let res_v2 = model_check_extended_formula(f1.to_string(), &stg, &context).unwrap();

                // eval the variant without domain (contains wild-card prop)
                let context = HashMap::from([
                    ("s1".to_string(), raw_set_1.clone()),
                    ("s2".to_string(), raw_set_2.clone()),
                ]);
                let res = model_check_extended_formula(f2.to_string(), &stg, &context).unwrap();

                assert!(res.as_bdd().iff(res_v2.as_bdd()).is_true());
            }
        }
    }
}
