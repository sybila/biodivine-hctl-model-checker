//! Components regarding the BN classification based on HCTL properties

pub mod generate_output;

use crate::bn_classification::generate_output::write_class_report_and_dump_bdds;
use crate::model_checking::{
    collect_unique_hctl_vars, get_extended_symbolic_graph, model_check_trees,
};
use crate::preprocessing::node::HctlTreeNode;
use crate::preprocessing::parser::parse_hctl_formula;
use crate::preprocessing::read_inputs::load_formulae;
use crate::preprocessing::tokenizer::try_tokenize_formula;
use crate::preprocessing::utils::check_props_and_rename_vars;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};
use std::fs::{read_dir, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

/// Parse formulae into syntax trees, and count maximal number of HCTL variables in a formula
fn parse_formulae_and_count_vars(
    bn: &BooleanNetwork,
    formulae: Vec<String>,
) -> Result<(Vec<HctlTreeNode>, usize), String> {
    let mut parsed_trees = Vec::new();
    let mut max_num_hctl_vars = 0;
    for formula in formulae {
        let tokens = try_tokenize_formula(formula)?;
        let tree = parse_hctl_formula(&tokens)?;

        let modified_tree = check_props_and_rename_vars(*tree, HashMap::new(), String::new(), bn)?;
        let num_hctl_vars = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();

        parsed_trees.push(modified_tree);
        if num_hctl_vars > max_num_hctl_vars {
            max_num_hctl_vars = num_hctl_vars;
        }
    }
    Ok((parsed_trees, max_num_hctl_vars))
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

/// Perform the classification of boolean networks based on given properties.
/// Takes a path to a partially defined BN model and paths to 2 sets of HCTL formulae. Assertions
/// are formulae that must be satisfied, and properties are formulae used for classification.
///
/// First, colors satisfying all assertions are computed, and then the set of remaining colors is
/// decomposed into categories based on properties. Report and BDDs representing resulting classes
/// are generated into `output_dir`.
pub fn classify(
    output_dir: &str,
    model_path: &str,
    assertion_formulae_path: &str,
    property_formulae_path: &str,
) -> Result<(), String> {
    // TODO: change existential semantics to universal (regarding colors in results) - doesnt matter if formula begins 3x.@x., but still
    // TODO: caching between assertions and properties somehow (and adjusting results when using them)

    // file to put some computation metadata
    let metadata_file_path = PathBuf::from(output_dir).join("metadata.txt");
    let mut metadata_file = File::create(metadata_file_path).unwrap();

    // read the model and formulae
    let assertion_formulae = load_formulae(assertion_formulae_path);
    let property_formulae = load_formulae(property_formulae_path);
    let model_string = read_to_string(model_path).unwrap();
    let bn = BooleanNetwork::try_from(model_string.as_str()).unwrap();
    println!("Loaded all inputs.");

    println!("Evaluating assertions...");
    // combine all assertions into one formula
    let single_assertion = combine_assertions(assertion_formulae.clone());

    // preproc all formulae at once - parse, compute max num of vars
    // it is crucial to include all formulae when computing number of HCTL vars needed
    let mut all_formulae = property_formulae.clone();
    all_formulae.push(single_assertion);
    let (mut all_trees, num_hctl_vars) = parse_formulae_and_count_vars(&bn, all_formulae)?;
    // save number of variables for future use
    write!(metadata_file, "{}", num_hctl_vars).unwrap();

    // instantiate extended STG with enough variables to evaluate all formulae
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars as u16);

    // compute the satisfying colors for assertion formula
    let assertion_tree = all_trees.pop().unwrap();
    let result_assertions = model_check_trees(vec![assertion_tree], &graph)?;
    let valid_colors: GraphColors = result_assertions.get(0).unwrap().colors();
    println!("Assertions evaluated.");

    // restrict the colors on the symbolic graph
    let graph = SymbolicAsyncGraph::with_custom_context(
        bn,
        graph.symbolic_context().clone(),
        valid_colors.as_bdd().clone(),
    )?;

    println!("Evaluating properties...");
    // model check all properties on restricted graph
    let results_properties = model_check_trees(all_trees, &graph)?;
    let colors_properties: Vec<GraphColors> = results_properties
        .iter()
        .map(|colored_vertices| colored_vertices.colors())
        .collect();
    println!("Classifying based on model-checking results...");

    // print the classification report and dump resulting BDDs
    write_class_report_and_dump_bdds(
        &assertion_formulae,
        valid_colors,
        &property_formulae,
        &colors_properties,
        output_dir,
    );
    println!("Output finished.");

    Ok(())
}

/// Collect the results of classification, which are color sets encoded as BDDs.
///
/// Each BDD is dumped in a file in `results_dir`. Moreover, excluding these BDD files, the dir
/// contains a report and a metadata file. Metadata file contains information regarding the number
/// of extended symbolic HCTL variables supported by the BDDs.
/// The file at `model_path` contains the original parametrized model that was used for the
/// classification.
pub fn load_classifier_output(results_dir: &str, model_path: &str) -> Vec<(String, GraphColors)> {
    // load number of HCTL variables from computation metadata
    let metadata_file_path = PathBuf::from(results_dir).join("metadata.txt");
    let num_hctl_vars: u16 = read_to_string(metadata_file_path)
        .unwrap()
        .parse::<u16>()
        .unwrap();

    // load the BN model and generate extended symbolic graph
    let model_string = read_to_string(model_path).unwrap();
    let bn = BooleanNetwork::try_from(model_string.as_str()).unwrap();
    let graph = get_extended_symbolic_graph(&bn, num_hctl_vars);

    // collect the colored sets from the BDD dumps together with their "names"
    let mut named_color_sets = Vec::new();

    // expects only BDD dumps (individual files) and a report&metadata in the directory (for now)
    let files = read_dir(results_dir).unwrap();
    for file in files {
        let path = file.unwrap().path().clone();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if file_name == "report.txt" || file_name == "metadata.txt" {
            continue;
        }
        let mut file = File::open(path.clone()).unwrap();

        // read the raw BDD
        let bdd = biodivine_lib_bdd::Bdd::read_as_string(&mut file).unwrap();

        let color_set = GraphColors::new(bdd, graph.symbolic_context());
        named_color_sets.push((file_name.to_string(), color_set));
    }
    named_color_sets
}

#[cfg(test)]
mod tests {
    use crate::bn_classification::combine_assertions;

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
