//! Model-checking analysis from start to finish, with progress output and result prints.

use crate::evaluation::algorithm::{compute_steady_states, eval_node};
use crate::evaluation::eval_info::EvalInfo;
use crate::model_checking::{collect_unique_hctl_vars, get_extended_symbolic_graph};
use crate::preprocessing::parser::parse_hctl_formula;
use crate::preprocessing::tokenizer::try_tokenize_formula;
use crate::preprocessing::utils::check_props_and_rename_vars;
use crate::result_print::*;

use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

/// Perform the whole model checking analysis regarding several formulae (includes the complete
/// process from the parsing to summarizing results).
/// Print the selected amount of result info (no prints / summary / detailed summary / exhaustive)
/// for each formula.
pub fn analyse_formulae(
    bn: &BooleanNetwork,
    formulae: Vec<String>,
    print_op: PrintOptions,
) -> Result<(), String> {
    let start = SystemTime::now();
    print_if_allowed("=========\nPARSE INFO:\n=========\n".to_string(), print_op);

    // first parse all the formulae and count max number of HCTL variables
    let mut parsed_trees = Vec::new();
    let mut max_num_hctl_vars = 0;
    for formula in formulae {
        print_if_allowed(format!("Formula: {formula}"), print_op);
        let tokens = try_tokenize_formula(formula)?;
        let tree = parse_hctl_formula(&tokens)?;
        print_if_allowed(format!("Parsed formula:   {}", tree.subform_str), print_op);

        let modified_tree = check_props_and_rename_vars(*tree, HashMap::new(), String::new(), bn)?;
        let num_hctl_vars = collect_unique_hctl_vars(modified_tree.clone(), HashSet::new()).len();
        print_if_allowed(
            format!("Modified formula: {}", modified_tree.subform_str),
            print_op,
        );
        print_if_allowed("-----".to_string(), print_op);

        parsed_trees.push(modified_tree);
        if num_hctl_vars > max_num_hctl_vars {
            max_num_hctl_vars = num_hctl_vars;
        }
    }

    // instantiate one extended STG with enough variables to evaluate all formulae
    let graph = get_extended_symbolic_graph(bn, max_num_hctl_vars as u16);
    print_if_allowed(
        format!(
            "Loaded BN with {} components and {} parameters",
            graph.as_network().num_vars(),
            graph.symbolic_context().num_parameter_variables()
        ),
        print_op,
    );
    print_if_allowed(
        format!(
            "Time to parse all formulae + build STG: {}ms\n",
            start.elapsed().unwrap().as_millis()
        ),
        print_op,
    );

    // find duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalInfo::from_multiple_trees(&parsed_trees);
    print_if_allowed(
        format!(
            "Duplicate sub-formulae (canonized): {:?}",
            eval_info.get_duplicates()
        ),
        print_op,
    );
    // compute states with self-loops which will be needed, and add them to graph object
    let self_loop_states = compute_steady_states(&graph);
    print_if_allowed("self-loops computed".to_string(), print_op);

    print_if_allowed("=========\nEVAL INFO:\n=========\n".to_string(), print_op);

    // evaluate the formulae (perform the actual model checking) and summarize results
    for parse_tree in parsed_trees {
        let curr_comp_start = SystemTime::now();
        let result = eval_node(parse_tree, &graph, &mut eval_info, &self_loop_states);

        match print_op {
            PrintOptions::FullPrint => print_results_full(&graph, &result, curr_comp_start, true),
            PrintOptions::MediumPrint => summarize_results(&result, curr_comp_start),
            PrintOptions::ShortPrint => summarize_results(&result, curr_comp_start),
            PrintOptions::NoPrint => {}
        }
    }

    print_if_allowed(
        format!(
            "Total computation time: {}ms",
            start.elapsed().unwrap().as_millis()
        ),
        print_op,
    );
    Ok(())
}

#[allow(dead_code)]
/// Perform the whole model checking analysis for a single formula (includes the complete process
/// from the parsing to summarizing results).
/// Print the selected amount of result info (no prints / summary / detailed summary / exhaustive).
pub fn analyse_formula(
    bn: &BooleanNetwork,
    formula: String,
    print_option: PrintOptions,
) -> Result<(), String> {
    analyse_formulae(bn, vec![formula], print_option)
}
