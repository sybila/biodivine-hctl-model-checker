//! Symbolic classifier for BN models based on dynamic properties.
//!
//! Takes an input path to a partially defined BN model (format may be specified, aeon format is
//! default) and a 2 paths to sets of HCTL formulae. These are 2 kinds of formulae - assertions
//! that must be satisfied, and properties that are used for classification.
//!
//! First, conjunction of assertions is model-checked, and then the set of remaining colors is
//! decomposed into categories based on properties.
//!

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

use biodivine_hctl_model_checker::formula_preprocessing::node::HctlTreeNode;
use biodivine_hctl_model_checker::formula_preprocessing::parser::parse_hctl_formula;
use biodivine_hctl_model_checker::formula_preprocessing::read_inputs::load_formulae;
use biodivine_hctl_model_checker::formula_preprocessing::tokenizer::try_tokenize_formula;
use biodivine_hctl_model_checker::formula_preprocessing::vars_props_manipulation::check_props_and_rename_vars;
use biodivine_hctl_model_checker::model_checking::{
    collect_unique_hctl_vars, get_extended_symbolic_graph, model_check_trees,
};

use clap::Parser;

use biodivine_hctl_model_checker::result_print::write_class_report_and_dump_bdds;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::path::Path;

/// Structure to collect CLI arguments
#[derive(Parser)]
#[clap(
    author = "Ondřej Huvar",
    version,
    about = "Symbolic classifier for BN models based on dynamic properties."
)]
struct Arguments {
    /// Path to a file with BN model file in one of supported formats.
    model_path: String,

    /// Path to a file with assertion formulae that must be satisfied.
    assertion_formulae_path: String,

    /// Path to a file with property formulae that are used for classification.
    property_formulae_path: String,

    /// Path to an existing directory to which report and BDD results will be dumped.
    result_dir: String,
}

/// Parse formulae into syntax trees, and count maximal number of HCTL variables in a formula
fn parse_formulae_and_count_vars(
    bn: &BooleanNetwork,
    formulae: Vec<String>,
) -> (Vec<HctlTreeNode>, usize) {
    // TODO: make it return "Result"

    let mut parsed_trees = Vec::new();
    let mut max_num_hctl_vars = 0;
    for formula in formulae {
        let tokens = try_tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        let modified_tree =
            check_props_and_rename_vars(*tree, HashMap::new(), String::new(), bn).unwrap();
        let num_hctl_vars = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();

        parsed_trees.push(modified_tree);
        if num_hctl_vars > max_num_hctl_vars {
            max_num_hctl_vars = num_hctl_vars;
        }
    }
    (parsed_trees, max_num_hctl_vars)
}

/// Combine all assertions into one conjunction formula
/// Empty strings are transformed into "true" literal
fn combine_assertions(formulae: Vec<String>) -> String {
    let mut conjunction = String::new();
    for formula in formulae {
        conjunction.push_str(format!("({}) & ", formula).as_str());
    }

    // this ensures that formula does not end with "&"
    // moreover, even if there are no assertions, resulting formula will not be empty
    conjunction.push_str("true");

    conjunction
}

/// Wrapper function to invoke the model checker, works with CLI arguments.
fn main() {
    // TODO: change existential semantics to universal (regarding colors in results) - doesnt matter if formula begins 3x.@x., but still
    // TODO: caching between assertions and properties somehow (and adjusting results when using them)

    let args = Arguments::parse();

    // check if given paths are valid
    if !Path::new(args.assertion_formulae_path.as_str()).is_file() {
        println!("{} is not valid file", args.assertion_formulae_path);
        return;
    }
    if !Path::new(args.property_formulae_path.as_str()).is_file() {
        println!("{} is not valid file", args.property_formulae_path);
        return;
    }
    if !Path::new(args.model_path.as_str()).is_file() {
        println!("{} is not valid file", args.model_path);
        return;
    }
    if !Path::new(args.result_dir.as_str()).is_dir() {
        println!("{} is not valid directory", args.result_dir);
        return;
    }

    // read the model and formulae
    let assertion_formulae = load_formulae(args.assertion_formulae_path);
    let property_formulae = load_formulae(args.property_formulae_path);
    let model_string = read_to_string(args.model_path).unwrap();
    let bn = BooleanNetwork::try_from(model_string.as_str()).unwrap();

    // combine all assertions into one formula
    let single_assertion = combine_assertions(assertion_formulae.clone());

    // preproc all formulae at once - parse, compute max num of vars
    // it is crucial to include all formulae when computing number of HCTL vars needed
    let mut all_formulae = property_formulae.clone();
    all_formulae.push(single_assertion);
    let (mut all_trees, num_hctl_vars) = parse_formulae_and_count_vars(&bn, all_formulae);

    // instantiate extended STG with enough variables to evaluate all formulae
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars as u16);

    // compute the satisfying colors for assertion formula
    let assertion_tree = all_trees.pop().unwrap();
    let result_assertions = model_check_trees(vec![assertion_tree], &graph).unwrap();
    let valid_colors: GraphColors = result_assertions.get(0).unwrap().colors();

    // restrict the colors on the symbolic graph
    let graph = SymbolicAsyncGraph::with_custom_context(
        bn,
        graph.symbolic_context().clone(),
        valid_colors.as_bdd().clone(),
    )
    .unwrap();

    // model check all properties on restricted graph
    let results_properties = model_check_trees(all_trees, &graph).unwrap();
    let colors_properties: Vec<GraphColors> = results_properties
        .iter()
        .map(|colored_vertices| colored_vertices.colors())
        .collect();

    // print the classification report and dump resulting BDDs
    write_class_report_and_dump_bdds(
        &assertion_formulae,
        valid_colors,
        &property_formulae,
        &colors_properties,
        args.result_dir.as_str(),
    );
}

#[cfg(test)]
mod tests {
    use crate::combine_assertions;

    #[test]
    /// Test combining of assertion formulae into one conjunction formula.
    fn test_assertion_formulae_merge() {
        let formula1 = "3{x}: @{x}: AX {x}".to_string();
        let formula2 = "false".to_string();
        let formula3 = "a & b".to_string();

        // empty vector should result in true constant
        assert_eq!(combine_assertions(Vec::new()), "true".to_string());

        // otherwise, result should be a conjunction ending with `& true`
        assert_eq!(
            combine_assertions(vec![formula1.clone(), formula2.clone()]),
            "(3{x}: @{x}: AX {x}) & (false) & true".to_string(),
        );
        assert_eq!(
            combine_assertions(vec![formula1, formula2, formula3]),
            "(3{x}: @{x}: AX {x}) & (false) & (a & b) & true".to_string(),
        )
    }
}
