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
use crate::preprocessing::hctl_tree::HctlTreeNode;
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
fn parse_and_validate(
    formulae: Vec<String>,
    graph: &SymbolicAsyncGraph,
) -> Result<Vec<HctlTreeNode>, String> {
    // parse all the formulae and check that graph supports enough HCTL vars
    let mut parsed_trees = Vec::new();
    for formula in formulae {
        let tree = parse_and_minimize_hctl_formula(graph.symbolic_context(), formula.as_str())?;
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
    let parsed_trees = parse_and_validate(formulae, graph)?;
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
    let parsed_trees = parse_and_validate(formulae, graph)?;
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
fn parse_and_validate_extended(
    formulae: Vec<String>,
    graph: &SymbolicAsyncGraph,
    subst_context_props: &HashMap<String, GraphColoredVertices>,
    subst_context_domains: &HashMap<String, GraphColoredVertices>,
) -> Result<Vec<HctlTreeNode>, String> {
    // parse all the formulae and check that graph supports enough HCTL vars
    let mut parsed_trees = Vec::new();
    for formula in formulae {
        let tree = parse_and_minimize_extended_formula(graph.symbolic_context(), formula.as_str())?;

        // check that given extended symbolic graph supports enough stated variables
        if !check_hctl_var_support(graph, tree.clone()) {
            return Err("Graph does not support enough HCTL state variables".to_string());
        }

        let (wild_card_props, var_domains) = collect_unique_wild_cards(tree.clone());
        // check that all occurring wild-card props are present in `substitution_context`
        for wild_card in wild_card_props {
            if !subst_context_props.contains_key(wild_card.as_str()) {
                return Err(format!(
                    "Wild-card prop `{}` lacks evaluation context.",
                    wild_card
                ));
            }
        }
        // check that all occurring wild-card props are present in `substitution_context`
        for var_domain in var_domains {
            if !subst_context_domains.contains_key(var_domain.as_str()) {
                return Err(format!(
                    "Var domain `{}` lacks evaluation context.",
                    var_domain
                ));
            }
        }
        parsed_trees.push(tree);
    }
    Ok(parsed_trees)
}

/// Perform the model checking for list of `extended` HCTL formulae on a given transition `graph`,
/// but do not sanitize the results.
/// Return the resulting sets of colored vertices (in the same order as input formulae).
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
///
/// The `subst_context_props` is a mapping determining how `wild-card propositions` are evaluated.
/// The `subst_context_domains` is a mapping determining how `var domains` are evaluated.
/// Bdds of both must only depend on BN variables and colours, not on their symbolic variables.
pub fn model_check_multiple_extended_formulae_dirty(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
    subst_context_props: &HashMap<String, GraphColoredVertices>,
    subst_context_domains: &HashMap<String, GraphColoredVertices>,
) -> Result<Vec<GraphColoredVertices>, String> {
    // get the abstract syntactic trees and check compatibility with graph
    let parsed_trees =
        parse_and_validate_extended(formulae, stg, subst_context_props, subst_context_domains)?;

    // prepare the extended evaluation context

    // 1) find normal duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalContext::from_multiple_trees(&parsed_trees);
    // 2) extended the cache with given substitution context for wild-card nodes
    eval_info.extend_context_with_wild_cards(subst_context_props, subst_context_domains);
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
    Ok(results)
}

/// Perform the model checking for list of `extended` HCTL formulae on a given transition `graph`.
/// Return the resulting sets of colored vertices (in the same order as input formulae).
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
///
/// The `subst_context_props` is a mapping determining how `wild-card propositions` are evaluated.
/// The `subst_context_domains` is a mapping determining how `var domains` are evaluated.
/// Bdds of both must only depend on BN variables and colours, not on their symbolic variables.
pub fn model_check_multiple_extended_formulae(
    formulae: Vec<String>,
    stg: &SymbolicAsyncGraph,
    subst_context_props: &HashMap<String, GraphColoredVertices>,
    subst_context_domains: &HashMap<String, GraphColoredVertices>,
) -> Result<Vec<GraphColoredVertices>, String> {
    let results = model_check_multiple_extended_formulae_dirty(
        formulae,
        stg,
        subst_context_props,
        subst_context_domains,
    )?;

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
/// The `subst_context_props` is a mapping determining how `wild-card propositions` are evaluated.
/// The `subst_context_domains` is a mapping determining how `var domains` are evaluated.
/// Bdds of both must only depend on BN variables and colours, not on their symbolic variables.
pub fn model_check_extended_formula(
    formula: String,
    stg: &SymbolicAsyncGraph,
    subst_context_props: &HashMap<String, GraphColoredVertices>,
    subst_context_domains: &HashMap<String, GraphColoredVertices>,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_extended_formulae(
        vec![formula],
        stg,
        subst_context_props,
        subst_context_domains,
    )?;
    Ok(result[0].clone())
}

/// Perform the model checking for a given `extended` HCTL formula on a given transition `graph`,
/// but do not sanitize the results.
/// The `graph` object MUST support enough symbolic variables to represent all occurring HCTL vars.
///
/// The `subst_context_props` is a mapping determining how `wild-card propositions` are evaluated.
/// The `subst_context_domains` is a mapping determining how `var domains` are evaluated.
/// Bdds of both must only depend on BN variables and colours, not on their symbolic variables.
pub fn model_check_extended_formula_dirty(
    formula: String,
    stg: &SymbolicAsyncGraph,
    subst_context_props: &HashMap<String, GraphColoredVertices>,
    subst_context_domains: &HashMap<String, GraphColoredVertices>,
) -> Result<GraphColoredVertices, String> {
    let result = model_check_multiple_extended_formulae_dirty(
        vec![formula],
        stg,
        subst_context_props,
        subst_context_domains,
    )?;
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
    let tree = parse_and_validate(vec![formula], graph)?[0].clone();

    let mut eval_info = EvalContext::from_single_tree(&tree);
    // do not consider self-loops during EX computation (UNSAFE optimisation)
    let result = eval_node(
        tree,
        graph,
        &mut eval_info,
        &graph.mk_empty_colored_vertices(),
    );
    Ok(result)
}

#[cfg(test)]
/// Some basic tests for the model-checking procedure and corresponding utilities. Note that larger tests
/// involving complex models and formulae are in module `_test_model_checking`.
mod tests {

    use crate::mc_utils::get_extended_symbolic_graph;
    use crate::model_checking::{model_check_formula, parse_and_validate_extended};
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    #[test]
    /// Test that function errors correctly if graph object does not support enough state variables.
    fn model_check_not_enough_symbolic_vars() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        // create symbolic graph supporting only one variable
        let stg = get_extended_symbolic_graph(&bn, 1).unwrap();

        // define formula with two variables
        let formula = "!{x}: !{y}: (AX {x} & AX {y})".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains free variables.
    fn model_check_invalid_free_var() {
        // create placeholder bn and symbolic graph
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // define formula that contains free variable
        let formula = "AX {x}".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if some variable is quantified more than once in a sub-formula.
    fn model_check_invalid_quantification() {
        // create placeholder bn and symbolic graph
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // define formula with several times quantified var
        let formula = "!{x}: !{x}: AX {x}".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains invalid propositions (not present in the BN).
    fn model_check_invalid_proposition() {
        // create placeholder bn with a single var `v1`, and corresponding symbolic graph
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // define formula with an invalid proposition
        let formula = "AX invalid_proposition".to_string();
        assert!(model_check_formula(formula, &stg).is_err());
    }

    #[test]
    /// Test that the utility function for parsing and validating extended formulae can properly
    /// discover errors (such as missing context for wild-cards or domains).
    fn validate_extended_context() {
        // create placeholder bn and symbolic graph
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // test situation where one substitution is missing
        let sub_context_props = HashMap::from([("s".to_string(), stg.mk_empty_colored_vertices())]);
        let sub_context_domains = HashMap::new();
        let formula = "%s% & EF %t%".to_string();
        let res = parse_and_validate_extended(
            vec![formula],
            &stg,
            &sub_context_props,
            &sub_context_domains,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap(),
            "Wild-card prop `t` lacks evaluation context.".to_string()
        );

        // test situation where one domain is missing
        let sub_context_props = HashMap::new();
        let sub_context_domains =
            HashMap::from([("a".to_string(), stg.mk_empty_colored_vertices())]);
        let formula = "!{x} in %a%: !{y} in %b%: AX {x}".to_string();
        let res = parse_and_validate_extended(
            vec![formula],
            &stg,
            &sub_context_props,
            &sub_context_domains,
        );
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap(),
            "Var domain `b` lacks evaluation context.".to_string()
        );
    }
}
