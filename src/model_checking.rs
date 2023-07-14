//! High-level functionality regarding the whole model-checking process.

use crate::evaluation::algorithm::{compute_steady_states, eval_node};
use crate::evaluation::eval_info::EvalInfo;
use crate::mc_utils::*;
use crate::postprocessing::sanitizing::sanitize_colored_vertices;
use crate::preprocessing::node::HctlTreeNode;
use crate::preprocessing::parser::parse_and_minimize_hctl_formula;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::HashSet;

/// Perform the model checking for the list of HCTL syntax trees on GIVEN graph.
/// Return the list of resulting sets of colored vertices (in the same order as input formulae).
/// There MUST be enough symbolic variables to represent HCTL vars.
/// Does not sanitize the resulting BDDs.
pub fn model_check_multiple_trees_dirty(
    formula_trees: Vec<HctlTreeNode>,
    stg: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // find duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalInfo::from_multiple_trees(&formula_trees);
    // compute states with self-loops which will be needed, and add them to graph object
    let self_loop_states = compute_steady_states(stg);

    // evaluate the formulae (perform the actual model checking) and collect results
    let mut results: Vec<GraphColoredVertices> = Vec::new();
    for parse_tree in formula_trees {
        results.push(eval_node(
            parse_tree,
            stg,
            &mut eval_info,
            &self_loop_states,
        ));
    }
    Ok(results)
}

/// Perform the model checking for the list of HCTL syntax trees on GIVEN graph.
/// Return the list of resulting sets of colored vertices (in the same order as input formulae).
/// There MUST be enough symbolic variables to represent HCTL vars.
pub fn model_check_trees(
    formula_trees: Vec<HctlTreeNode>,
    stg: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // evaluate the formulae and collect results
    let results = model_check_multiple_trees_dirty(formula_trees, stg)?;

    // sanitize the results' bdds - get rid of additional bdd vars used for HCTL vars
    let sanitized_results: Vec<GraphColoredVertices> = results
        .iter()
        .map(|x| sanitize_colored_vertices(stg, x))
        .collect();
    Ok(sanitized_results)
}

/// Perform the model checking for a given HCTL syntax tree on GIVEN graph.
/// Return the resulting set of colored vertices.
/// There MUST be enough symbolic variables to represent all needed HCTL vars.
pub fn model_check_tree(
    formula_tree: HctlTreeNode,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_trees(vec![formula_tree], stg)?;
    Ok(result[0].clone())
}

/// Perform the model checking for the syntactic tree, but do not sanitize the results.
pub fn model_check_tree_dirty(
    formula_tree: HctlTreeNode,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_trees_dirty(vec![formula_tree], stg)?;
    Ok(result[0].clone())
}

fn parse_into_trees(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
) -> Result<Vec<HctlTreeNode>, String> {
    // first parse all the formulae and check that graph supports enough HCTL vars
    let mut parsed_trees = Vec::new();
    for formula in formulae {
        let tree = parse_and_minimize_hctl_formula(stg.as_network(), formula.as_str())?;

        // check that given extended symbolic graph supports enough stated variables
        let num_vars_formula = collect_unique_hctl_vars(tree.clone(), HashSet::new()).len();
        if !check_hctl_var_support(stg, num_vars_formula) {
            return Err("Graph does not support enough HCTL state variables".to_string());
        }

        parsed_trees.push(tree);
    }
    Ok(parsed_trees)
}

/// Perform the model checking for the list of formulae on GIVEN graph and return the list
/// of resulting sets of colored vertices (in the same order as input formulae).
/// Return Error if the given extended symbolic graph does not support enough extra BDD variables to
/// represent all needed HCTL state-variables or if some formula is badly formed.
pub fn model_check_multiple_formulae(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees
    let parsed_trees = parse_into_trees(formulae, stg)?;
    // run the main model-checking procedure on formulae trees
    model_check_trees(parsed_trees, stg)
}

/// Perform the model checking for the list of formulae, but do not sanitize the results.
pub fn model_check_multiple_formulae_dirty(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees
    let parsed_trees = parse_into_trees(formulae, stg)?;
    // run the main model-checking procedure on formulae trees
    model_check_multiple_trees_dirty(parsed_trees, stg)
}

/// Perform the model checking for given formula on GIVEN graph and return the resulting
/// set of colored vertices.
/// Return Error if the given extended symbolic graph does not support enough extra BDD variables
/// to represent all needed HCTL state-variables or if the formula is badly formed.
pub fn model_check_formula(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_formulae(vec![formula], stg)?;
    Ok(result[0].clone())
}

/// Perform the model checking for given formula, but do not sanitize the result.
pub fn model_check_formula_dirty(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_formulae_dirty(vec![formula], stg)?;
    Ok(result[0].clone())
}

#[allow(dead_code)]
/// Perform the model checking on GIVEN graph and return the resulting set of colored vertices.
/// Self-loops are not pre-computed, and thus are ignored in EX computation, which is fine for
/// some formulae, but incorrect for others - it is thus an UNSAFE optimisation - only use it
/// if you are sure everything will work fine.
/// This must NOT be used for formulae containing `!{x}:AX{x}` sub-formulae.
/// Also, does not sanitize results.
pub fn model_check_formula_unsafe_ex(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let tree = parse_and_minimize_hctl_formula(stg.as_network(), formula.as_str())?;
    // check that given extended symbolic graph supports enough stated variables
    let num_vars_formula = collect_unique_hctl_vars(tree.clone(), HashSet::new()).len();
    if !check_hctl_var_support(stg, num_vars_formula) {
        return Err("Graph does not support enough HCTL state variables".to_string());
    }

    let mut eval_info = EvalInfo::from_single_tree(&tree);

    // do not consider self-loops during EX computation (UNSAFE optimisation)
    let result = eval_node(tree, stg, &mut eval_info, &stg.mk_empty_vertices());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::mc_utils::{collect_unique_hctl_vars, get_extended_symbolic_graph};
    use crate::model_checking::{model_check_formula, model_check_formula_dirty};
    use crate::preprocessing::parser::parse_hctl_formula;
    use crate::preprocessing::utils::check_props_and_rename_vars;

    use biodivine_lib_param_bn::BooleanNetwork;

    use std::collections::{HashMap, HashSet};

    // model FISSION-YEAST-2008
    const MODEL_FISSION_YEAST: &str = r"
targets,factors
Cdc25, ((!Cdc2_Cdc13 & (Cdc25 & !PP)) | ((Cdc2_Cdc13 & (!Cdc25 & !PP)) | (Cdc2_Cdc13 & Cdc25)))
Cdc2_Cdc13, (!Ste9 & (!Rum1 & !Slp1))
Cdc2_Cdc13_A, (!Ste9 & (!Rum1 & (!Slp1 & (!Wee1_Mik1 & Cdc25))))
PP, Slp1
Rum1, ((!SK & (!Cdc2_Cdc13 & (!Rum1 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP))))))))
SK, Start
Slp1, Cdc2_Cdc13_A
Start, false
Ste9, ((!SK & (!Cdc2_Cdc13 & (!Ste9 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP))))))))
Wee1_Mik1, ((!Cdc2_Cdc13 & (!Wee1_Mik1 & PP)) | ((!Cdc2_Cdc13 & Wee1_Mik1) | (Cdc2_Cdc13 & (Wee1_Mik1 & PP))))
";

    // model MAMMALIAN-CELL-CYCLE-2006
    const MODEL_MAMMALIAN_CELL_CYCLE: &str = r"
targets,factors
v_Cdc20, v_CycB
v_Cdh1, ((v_Cdc20 | (v_p27 & !v_CycB)) | !(((v_p27 | v_CycB) | v_CycA) | v_Cdc20))
v_CycA, ((v_CycA & !(((v_Cdh1 & v_UbcH10) | v_Cdc20) | v_Rb)) | (v_E2F & !(((v_Cdh1 & v_UbcH10) | v_Cdc20) | v_Rb)))
v_CycB, !(v_Cdc20 | v_Cdh1)
v_CycE, (v_E2F & !v_Rb)
v_E2F, ((v_p27 & !(v_CycB | v_Rb)) | !(((v_p27 | v_Rb) | v_CycB) | v_CycA))
v_Rb, ((v_p27 & !(v_CycD | v_CycB)) | !((((v_CycE | v_p27) | v_CycB) | v_CycD) | v_CycA))
v_UbcH10, (((((v_UbcH10 & ((v_Cdh1 & ((v_CycB | v_Cdc20) | v_CycA)) | !v_Cdh1)) | (v_CycA & !v_Cdh1)) | (v_Cdc20 & !v_Cdh1)) | (v_CycB & !v_Cdh1)) | !((((v_UbcH10 | v_Cdh1) | v_CycB) | v_Cdc20) | v_CycA))
v_p27, ((v_p27 & !((v_CycD | (v_CycA & v_CycE)) | v_CycB)) | !((((v_CycE | v_p27) | v_CycB) | v_CycD) | v_CycA))
";

    // largely parametrized version of the model ASYMMETRIC-CELL-DIVISION-B
    const MODEL_ASYMMETRIC_CELL_DIVISION: &str = r"
DivJ -?? DivK
PleC -?? DivK
DivK -?? DivL
DivL -?? CckA
CckA -?? ChpT
ChpT -?? CpdR
CpdR -?? ClpXP_RcdA
ChpT -?? CtrAb
ClpXP_RcdA -?? CtrAb
DivK -?? DivJ
PleC -?? DivJ
DivK -?? PleC
$CckA: DivL
$ChpT: CckA
$DivK: (!PleC & DivJ)
";

    /// Run the evaluation tests for the set of given formulae on given model.
    /// Compare numbers of results with the expected numbers given.
    /// The `test_tuples` consist of tuples <formula, num_total, num_colors, num_states>.
    fn test_model_check_basic_formulae(
        test_tuples: Vec<(&str, f64, f64, f64)>,
        bn: BooleanNetwork,
    ) {
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
    fn test_model_check_basic_formulae_yeast() {
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
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        test_model_check_basic_formulae(test_tuples, bn);
    }

    #[test]
    /// Test evaluation of several important formulae on model MAMMALIAN-CELL-CYCLE-2006.
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON.
    fn test_model_check_basic_formulae_mammal() {
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
        let bn = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        test_model_check_basic_formulae(test_tuples, bn);
    }

    #[test]
    /// Test evaluation of several important formulae on model ASYMMETRIC-CELL-DIVISION-B.
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON.
    fn test_model_check_basic_formulae_cell_division() {
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
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        test_model_check_basic_formulae(test_tuples, bn);
    }

    /// Test evaluation of pairs of equivalent formulae on given BN model.
    /// Compare whether the results are the same.
    fn test_model_check_equivalences(bn: BooleanNetwork) {
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
        ];

        for (formula1, formula2) in equivalent_formulae_pairs {
            let result1 = model_check_formula(formula1.to_string(), &stg).unwrap();
            let result2 = model_check_formula(formula2.to_string(), &stg).unwrap();
            assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());

            let result1 = model_check_formula_dirty(formula1.to_string(), &stg).unwrap();
            let result2 = model_check_formula_dirty(formula2.to_string(), &stg).unwrap();
            assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());
        }
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model FISSION-YEAST-2008.
    fn test_model_check_equivalences_yeast() {
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model MAMMALIAN-CELL-CYCLE-2006.
    fn test_model_check_equivalences_mammal() {
        let bn = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test evaluation of pairs of equivalent formulae on model ASYMMETRIC-CELL-DIVISION-B.
    fn test_model_check_equivalences_cell_division() {
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        test_model_check_equivalences(bn);
    }

    #[test]
    /// Test that function errors correctly if graph does not support enough state variables.
    fn test_model_check_error_1() {
        // create symbolic graph supporting only one variable
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

        // define formula with two variables
        let formula = "!{x}: !{y}: (AX {x} & AX {y})".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains free variables.
    fn test_model_check_error_2() {
        // create placeholder symbolic graph
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // define formula that contains free variable
        let formula = "AX {x}".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains several times quantified vars.
    fn test_model_check_error_3() {
        // create placeholder symbolic graph
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // define formula with several times quantified var
        let formula = "!{x}: !{x}: AX {x}".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains invalid propositions.
    fn test_model_check_error_4() {
        // create placeholder symbolic graph
        let bn = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // define formula with invalid proposition
        let formula = "AX invalid_proposition".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test regarding collecting state vars from HCTL formulae.
    fn test_state_var_collecting() {
        // formula "FORKS1 & FORKS2" - both parts are semantically same, just use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))".to_string();
        let tree = parse_hctl_formula(formula.as_str()).unwrap();

        // test for original tree
        let expected_vars = vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "x1".to_string(),
            "y1".to_string(),
            "z1".to_string(),
        ];
        assert_eq!(
            collect_unique_hctl_vars(tree.clone(), HashSet::new()),
            HashSet::from_iter(expected_vars)
        );

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // and for tree with minimized number of renamed state vars
        let modified_tree =
            check_props_and_rename_vars(tree, HashMap::new(), String::new(), &bn).unwrap();
        let expected_vars = vec!["x".to_string(), "xx".to_string(), "xxx".to_string()];
        assert_eq!(
            collect_unique_hctl_vars(modified_tree, HashSet::new()),
            HashSet::from_iter(expected_vars)
        );
    }
}
