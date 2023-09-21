//! High-level functionality regarding the whole model-checking process.
//! Several variants of the model-checking procedure are provided:
//!  - variants for both single or multiple formulae
//!  - variants for formulae given by a string or a syntactic tree
//!  - `dirty` variants that do not sanitize the resulting BDDs (and thus, the BDDs retain additional vars)
//!  - variants allowing `extended` HCTL with special propositions referencing raw sets
//!  - variants using potentially unsafe optimizations, targeted for specific use cases

use crate::evaluation::algorithm::{compute_steady_states, eval_node};
use crate::evaluation::eval_context::EvalContext;
use crate::mc_utils::*;
use crate::postprocessing::sanitizing::sanitize_colored_vertices;
use crate::preprocessing::node::HctlTreeNode;
use crate::preprocessing::parser::{
    parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula,
};

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::HashMap;

/// Perform the model checking for the list of HCTL syntax trees on a given transition `graph`.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
/// Return the list of resulting sets of colored vertices (in the same order as input formulae).
///
/// This version does not sanitize the resulting BDDs (`model_check_multiple_trees` does).
pub fn model_check_multiple_trees_dirty(
    formula_trees: Vec<HctlTreeNode>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // find duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalContext::from_multiple_trees(&formula_trees);
    // pre-compute states with self-loops which will be needed during eval
    let self_loop_states = compute_steady_states(graph);

    // evaluate the formulae (perform the actual model checking) and collect results
    let mut results: Vec<GraphColoredVertices> = Vec::new();
    for parse_tree in formula_trees {
        results.push(eval_node(
            parse_tree,
            graph,
            &mut eval_info,
            &self_loop_states,
        ));
    }
    Ok(results)
}

/// Perform the model checking for the syntactic tree, but do not sanitize the results.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
pub fn model_check_tree_dirty(
    formula_tree: HctlTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_trees_dirty(vec![formula_tree], graph)?;
    Ok(result[0].clone())
}

/// Perform the model checking for the list of HCTL syntax trees on a given transition `graph`.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
/// Return the list of resulting sets of colored vertices (in the same order as input formulae).
pub fn model_check_multiple_trees(
    formula_trees: Vec<HctlTreeNode>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // evaluate the formulae and collect results
    let results = model_check_multiple_trees_dirty(formula_trees, graph)?;

    // sanitize the results' bdds - get rid of additional bdd vars used for HCTL vars
    let sanitized_results: Vec<GraphColoredVertices> = results
        .iter()
        .map(|x| sanitize_colored_vertices(graph, x))
        .collect();
    Ok(sanitized_results)
}

/// Perform the model checking for a given HCTL formula's syntax tree on a given transition `graph`.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
/// Return the resulting set of colored vertices.
pub fn model_check_tree(
    formula_tree: HctlTreeNode,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_trees(vec![formula_tree], graph)?;
    Ok(result[0].clone())
}

/// Parse given HCTL formulae into syntactic trees and perform compatibility check with
/// the provided `graph` (i.e., check if `graph` object supports enough symbolic variables).
fn parse_hctl_and_validate(
    formulae: Vec<String>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<HctlTreeNode>, String> {
    // parse all the formulae and check that graph supports enough HCTL vars
    let mut parsed_trees = Vec::new();
    for formula in formulae {
        let tree = parse_and_minimize_hctl_formula(graph.as_network(), formula.as_str())?;
        // check that given extended symbolic graph supports enough stated variables
        if !check_hctl_var_support(graph, tree.clone()) {
            return Err("Graph does not support enough HCTL state variables".to_string());
        }
        parsed_trees.push(tree);
    }
    Ok(parsed_trees)
}

/// Perform the model checking for the list of HCTL formulae on a given transition `graph`.
/// Return the resulting sets of colored vertices (in the same order as input formulae).
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
pub fn model_check_multiple_formulae(
    formulae: Vec<String>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees
    let parsed_trees = parse_hctl_and_validate(formulae, graph)?;
    // run the main model-checking procedure on formulae trees
    model_check_multiple_trees(parsed_trees, graph)
}

/// Perform the model checking for the list of formulae, but do not sanitize the results.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
pub fn model_check_multiple_formulae_dirty(
    formulae: Vec<String>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees
    let parsed_trees = parse_hctl_and_validate(formulae, graph)?;
    // run the main model-checking procedure on formulae trees
    model_check_multiple_trees_dirty(parsed_trees, graph)
}

/// Perform the model checking for a given HCTL formula on a given transition `graph`.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
/// Return the resulting set of colored vertices.
pub fn model_check_formula(
    formula: String,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_formulae(vec![formula], graph)?;
    Ok(result[0].clone())
}

/// Perform the model checking for given formula, but do not sanitize the result.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
pub fn model_check_formula_dirty(
    formula: String,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_formulae_dirty(vec![formula], graph)?;
    Ok(result[0].clone())
}

/// Parse given extended HCTL formulae into syntactic trees and perform compatibility check with
/// the provided `graph` (i.e., check if `graph` object supports enough symbolic variables).
fn parse_extended_and_validate(
    formulae: Vec<String>,
    graph: &SymbolicAsyncGraph,
    substitution_context: &HashMap<String, GraphColoredVertices>,
) -> Result<Vec<HctlTreeNode>, String> {
    // parse all the formulae and check that graph supports enough HCTL vars
    let mut parsed_trees = Vec::new();
    for formula in formulae {
        let tree = parse_and_minimize_extended_formula(graph.as_network(), formula.as_str())?;

        // check that given extended symbolic graph supports enough stated variables
        if !check_hctl_var_support(graph, tree.clone()) {
            return Err("Graph does not support enough HCTL state variables".to_string());
        }
        // check that all occurring wild-card propositions are present in `substitution_context`
        for wild_card_prop in collect_unique_wild_card_props(tree.clone()) {
            if !substitution_context.contains_key(wild_card_prop.as_str()) {
                return Err(format!(
                    "Wild-card proposition `{}` lacks evaluation context.",
                    wild_card_prop
                ));
            }
        }
        parsed_trees.push(tree);
    }
    Ok(parsed_trees)
}

/// Perform the model checking for list of `extended` HCTL formulae on a given transition `graph`.
/// Return the resulting sets of colored vertices (in the same order as input formulae).
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
///
/// The `substitution context` is a mapping determining how `wild-card propositions` are evaluated.
pub fn model_check_multiple_extended_formulae(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
    substitution_context: HashMap<String, GraphColoredVertices>,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees and check compatibility with graph
    let parsed_trees = parse_extended_and_validate(formulae, stg, &substitution_context)?;

    // prepare the extended evaluation context

    // 1) find normal duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalContext::from_multiple_trees(&parsed_trees);
    // 2) extended the cache with given substitution context for wild-card nodes
    eval_info.extend_context_with_wild_cards(substitution_context);
    // 3) pre-compute compute states with self-loops which will be needed during eval
    let self_loop_states = compute_steady_states(stg);

    // evaluate the formulae (perform the actual model checking) and collect results
    let mut results: Vec<GraphColoredVertices> = Vec::new();
    for parse_tree in parsed_trees {
        results.push(eval_node(
            parse_tree,
            stg,
            &mut eval_info,
            &self_loop_states,
        ));
    }

    // sanitize the results' bdds - get rid of additional bdd vars used for HCTL vars
    let sanitized_results: Vec<GraphColoredVertices> = results
        .iter()
        .map(|x| sanitize_colored_vertices(stg, x))
        .collect();
    Ok(sanitized_results)
}

/// Perform the model checking for a given `extended` HCTL formula on a given transition `graph`.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
///
/// The `substitution context` is a mapping determining how `wild-card propositions` are evaluated.
pub fn model_check_extended_formula(
    formula: String,
    stg: &SymbolicAsyncGraph,
    substitution_context: HashMap<String, GraphColoredVertices>,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_extended_formulae(vec![formula], stg, substitution_context)?;
    Ok(result[0].clone())
}

/// Model check HCTL `formula` on a given transition `graph`.
/// This version does not compute with self-loops. They are thus ignored in EX computation, which
/// might fine for some formulae, but can be incorrect for others. It is an UNSAFE optimisation,
/// only use it if you are sure everything will work fine.
/// This function must NOT be used for formulae containing `!{x}:AX{x}` sub-formulae.
///
/// Also, this does not sanitize results.
pub fn model_check_formula_unsafe_ex(
    formula: String,
    graph: &SymbolicAsyncGraph,
) -> Result<GraphColoredVertices, String> {
    let tree = parse_hctl_and_validate(vec![formula], graph)?[0].clone();

    let mut eval_info = EvalContext::from_single_tree(&tree);
    // do not consider self-loops during EX computation (UNSAFE optimisation)
    let result = eval_node(tree, graph, &mut eval_info, &graph.mk_empty_vertices());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::mc_utils::get_extended_symbolic_graph;
    use crate::model_checking::{
        model_check_extended_formula, model_check_formula, model_check_formula_dirty,
        model_check_formula_unsafe_ex, parse_extended_and_validate,
    };
    use std::collections::HashMap;

    use crate::postprocessing::sanitizing::sanitize_colored_vertices;
    use biodivine_lib_param_bn::BooleanNetwork;

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
    /// Test evaluation of (very simple) extended formulae, where special propositions are
    /// evaluated as various simple pre-computed sets.
    fn test_model_check_extended_formulae_simple() {
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

        // 1) first test, only proposition substituted
        let formula_v1 = "PleC & EF PleC".to_string();
        let sub_formula = "PleC".to_string();
        let formula_v2 = "%s% & EF %s%".to_string();

        let result_v1 = model_check_formula(formula_v1, &stg).unwrap();
        // use 'dirty' version to avoid sanitation (for BDD to retain all symbolic vars)
        let result_sub = model_check_formula_dirty(sub_formula, &stg).unwrap();
        let context = HashMap::from([("s".to_string(), result_sub)]);
        let result_v2 = model_check_extended_formula(formula_v2, &stg, context).unwrap();
        assert!(result_v1.as_bdd().iff(result_v2.as_bdd()).is_true());

        // 2) second test, disjunction substituted
        let formula_v1 = "EX (PleC | DivK)".to_string();
        let sub_formula = "PleC | DivK".to_string();
        let formula_v2 = "EX %s%".to_string();

        let result_v1 = model_check_formula(formula_v1, &stg).unwrap();
        // use 'dirty' version to avoid sanitation (for BDD to retain all symbolic vars)
        let result_sub = model_check_formula_dirty(sub_formula, &stg).unwrap();
        let context = HashMap::from([("s".to_string(), result_sub)]);
        let result_v2 = model_check_extended_formula(formula_v2, &stg, context).unwrap();
        assert!(result_v1.as_bdd().iff(result_v2.as_bdd()).is_true());
    }

    /// Test evaluation of extended HCTL formulae, in which `wild-card properties` can
    /// represent already pre-computed results.
    fn test_model_check_extended_formulae(bn: BooleanNetwork) {
        // test formulae use 3 HCTL vars at most
        let stg = get_extended_symbolic_graph(&bn, 3).unwrap();

        // the test is conducted on two different formulae

        // first define and evaluate the two formulae normally in one step
        let formula1 = "!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))";
        let formula2 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))";
        let result1 = model_check_formula(formula1.to_string(), &stg).unwrap();
        let result2 = model_check_formula(formula2.to_string(), &stg).unwrap();

        // now precompute part of the formula, and then substitute it as `wild-card proposition`
        let substitution_formula = "(!{z}: AX {z})";
        // we must use 'dirty' version to avoid sanitation (BDDs must retain all symbolic vars)
        let raw_set = model_check_formula_dirty(substitution_formula.to_string(), &stg).unwrap();
        let context = HashMap::from([("subst".to_string(), raw_set)]);

        let formula1_v2 = "!{x}: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%)";
        let formula2_v2 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & %subst%) & EF ({y} & %subst%) & AX (EF ({x} & %subst%) ^ EF ({y} & %subst%))";
        let result1_v2 =
            model_check_extended_formula(formula1_v2.to_string(), &stg, context.clone()).unwrap();
        let result2_v2 =
            model_check_extended_formula(formula2_v2.to_string(), &stg, context).unwrap();

        assert!(result1.as_bdd().iff(result1_v2.as_bdd()).is_true());
        assert!(result2.as_bdd().iff(result2_v2.as_bdd()).is_true());

        // also double check that running "extended" evaluation on the original formula (without
        // wild-card propositions) is the same as running the standard variant
        let result1_v2 =
            model_check_extended_formula(formula1.to_string(), &stg, HashMap::new()).unwrap();
        let result2_v2 =
            model_check_extended_formula(formula2.to_string(), &stg, HashMap::new()).unwrap();
        assert!(result1.as_bdd().iff(result1_v2.as_bdd()).is_true());
        assert!(result2.as_bdd().iff(result2_v2.as_bdd()).is_true());
    }

    #[test]
    /// Test evaluation of extended HCTL formulae, in which `wild-card properties` can
    /// represent already pre-computed results. Use all 3 pre-defined models.
    fn test_model_check_extended_all_models() {
        let bn1 = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        let bn2 = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        let bn3 = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();

        test_model_check_extended_formulae(bn1);
        test_model_check_extended_formulae(bn2);
        test_model_check_extended_formulae(bn3);
    }

    #[test]
    /// Test that the (potentially unsafe) optimization works on several formulae where it
    /// should be applicable without any loss of information.
    fn test_model_check_unsafe_version() {
        let bn1 = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        let bn2 = BooleanNetwork::try_from_bnet(MODEL_MAMMALIAN_CELL_CYCLE).unwrap();
        let bn3 = BooleanNetwork::try_from_bnet(MODEL_FISSION_YEAST).unwrap();
        let bns = vec![bn1, bn2, bn3];

        for bn in bns {
            let stg = get_extended_symbolic_graph(&bn, 3).unwrap();
            // use formula for attractors that won't be recognized as the "attractor pattern"
            let formula = "!{x}: AG EF ({x} & {x})".to_string();
            let result1 = model_check_formula(formula.clone(), &stg).unwrap();
            // result of the unsafe eval must be sanitized
            let result2 = sanitize_colored_vertices(
                &stg,
                &model_check_formula_unsafe_ex(formula.clone(), &stg).unwrap(),
            );
            assert!(result1.as_bdd().iff(result2.as_bdd()).is_true());
        }
    }

    #[test]
    /// Test that the helper function for parsing and validating extended formulae properly
    /// discovers errors.
    fn test_validation_extended_context() {
        let bn = BooleanNetwork::try_from(MODEL_ASYMMETRIC_CELL_DIVISION).unwrap();
        let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

        // test situation where one substitution is missing
        let sub_context = HashMap::from([("s".to_string(), stg.mk_empty_vertices())]);
        let formula = "%s% & EF %t%".to_string();
        let res = parse_extended_and_validate(vec![formula], &stg, &sub_context);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap(),
            "Wild-card proposition `t` lacks evaluation context.".to_string()
        );
    }
}
