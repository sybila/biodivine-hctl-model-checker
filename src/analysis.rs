//! Model-checking analysis from start to finish, with progress output and result prints.

use crate::evaluation::algorithm::{compute_steady_states, eval_node};
use crate::evaluation::eval_context::EvalContext;
use crate::mc_utils::{collect_unique_hctl_vars, get_extended_symbolic_graph};
use crate::preprocessing::parser::parse_hctl_formula;
use crate::preprocessing::utils::validate_props_and_rename_vars;
use crate::result_print::*;

use biodivine_lib_param_bn::BooleanNetwork;

use crate::generate_output::build_result_archive;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicContext};
use std::collections::HashMap;
use std::time::SystemTime;

/// Perform the whole model checking analysis regarding several (individual) formulae. This
/// comprises the complete process from the parsing to summarizing results).
///
/// Print the selected amount of result info (no prints / summary / detailed summary / exhaustive)
/// for each formula.
///
/// If provided, dump the resulting BDDs to an zip archive.
pub fn analyse_formulae(
    bn: &BooleanNetwork,
    formulae: Vec<String>,
    print_opt: PrintOptions,
    result_zip: Option<String>,
) -> Result<(), String> {
    let start = SystemTime::now();
    print_if_allowed(
        "============ INITIAL PHASE ============".to_string(),
        print_opt,
    );

    // first parse all the formulae and count max number of HCTL variables
    let mut parsed_trees = Vec::new();
    let mut max_num_hctl_vars = 0;
    print_if_allowed(format!("Read {} HCTL formulae.", formulae.len()), print_opt);
    print_if_allowed("-----".to_string(), print_opt);

    let plain_context = SymbolicContext::new(bn).unwrap();
    for (i, formula) in formulae.iter().enumerate() {
        print_if_allowed(
            format!("Original formula n.{}: {formula}", i + 1),
            print_opt,
        );
        let tree = parse_hctl_formula(formula.as_str())?;
        print_if_allowed(format!("Parsed version:       {tree}"), print_opt);

        let modified_tree =
            validate_props_and_rename_vars(tree, HashMap::new(), String::new(), &plain_context)?;
        let num_hctl_vars = collect_unique_hctl_vars(modified_tree.clone()).len();
        print_if_allowed(format!("Modified version:     {modified_tree}"), print_opt);
        print_if_allowed("-----".to_string(), print_opt);

        parsed_trees.push(modified_tree);
        if num_hctl_vars > max_num_hctl_vars {
            max_num_hctl_vars = num_hctl_vars;
        }
    }

    // instantiate one extended STG with enough variables to evaluate all formulae
    let graph = get_extended_symbolic_graph(bn, max_num_hctl_vars as u16)?;
    print_if_allowed(
        format!(
            "Loaded BN model with {} components and {} parameters.",
            graph.num_vars(),
            graph.symbolic_context().num_parameter_variables()
        ),
        print_opt,
    );
    print_if_allowed(
        format!(
            "Built STG that admits {:.0} states and {:.0} colors.",
            graph
                .unit_colored_vertices()
                .vertices()
                .approx_cardinality(),
            graph.unit_colors().approx_cardinality(),
        ),
        print_opt,
    );
    print_if_allowed(
        format!(
            "Time to parse all formulae + build STG: {}ms.",
            start.elapsed().unwrap().as_millis()
        ),
        print_opt,
    );
    print_if_allowed("-----".to_string(), print_opt);

    // find duplicate sub-formulae throughout all formulae + initiate caching structures
    let mut eval_info = EvalContext::from_multiple_trees(&parsed_trees);
    print_if_allowed(
        format!(
            "Found following duplicate sub-formulae (canonized): {:?}",
            eval_info.get_duplicates()
        ),
        print_opt,
    );
    print_if_allowed("-----".to_string(), print_opt);

    // pre-compute states with self-loops which will be needed
    let self_loop_states = compute_steady_states(&graph);
    print_if_allowed(
        "Self-loops successfully pre-computed.\n".to_string(),
        print_opt,
    );

    print_if_allowed(
        "============= EVALUATION PHASE =============".to_string(),
        print_opt,
    );

    // evaluate the formulae (perform the actual model checking) and summarize results
    let mut results: HashMap<String, GraphColoredVertices> = HashMap::new();
    for (i, parse_tree) in parsed_trees.iter().enumerate() {
        let formula = formulae[i].clone();
        print_if_allowed(format!("Evaluating formula {}...", i + 1), print_opt);
        let curr_comp_start = SystemTime::now();
        let result = eval_node(
            parse_tree.clone(),
            &graph,
            &mut eval_info,
            &self_loop_states,
        );

        match print_opt {
            PrintOptions::Exhaustive => {
                print_results_full(formula, &graph, &result, curr_comp_start, true)
            }
            PrintOptions::WithProgress => summarize_results(formula, &result, curr_comp_start),
            PrintOptions::JustSummary => summarize_results(formula, &result, curr_comp_start),
            PrintOptions::NoPrint => {}
        }
        results.insert(format!("formula-{i}"), result);
    }

    // create the archive for the results (for now, there'll be just the model string)
    if let Some(zip_path) = result_zip {
        print_if_allowed(format!("Writing the results to {zip_path}.\n"), print_opt);
        build_result_archive(
            results,
            zip_path.as_str(),
            bn.to_string().as_str(),
            formulae,
        )
        .map_err(|e| e.to_string())?;
        print_if_allowed("Results successfully written.\n".to_string(), print_opt);
    }

    print_if_allowed(
        format!(
            "Total computation time: {}ms",
            start.elapsed().unwrap().as_millis()
        ),
        print_opt,
    );
    Ok(())
}

#[allow(dead_code)]
/// Perform the whole model checking analysis for a single formula (complete process from
/// the parsing to summarizing results).
///
/// Print the selected amount of result info (no prints / summary / detailed summary / exhaustive).
///
/// If provided, dump the resulting BDDs to an zip archive.
pub fn analyse_formula(
    bn: &BooleanNetwork,
    formula: String,
    print_opt: PrintOptions,
    result_zip: Option<String>,
) -> Result<(), String> {
    analyse_formulae(bn, vec![formula], print_opt, result_zip)
}

#[cfg(test)]
mod tests {
    use crate::analysis::{analyse_formula, analyse_formulae};
    use crate::result_print::PrintOptions;
    use biodivine_lib_param_bn::BooleanNetwork;

    #[test]
    /// Simple test to check whether the whole analysis runs without an error.
    fn test_analysis_run() {
        let model = r"
                $frs2:(!erk & fgfr)
                fgfr -> frs2
                erk -| frs2
                $fgfr:(fgfr & !frs2)
                frs2 -| fgfr
                fgfr -> fgfr
                $erk:(frs2 | shc)
                frs2 -> erk
                shc -> erk
                $shc:fgfr
                fgfr -> shc
        ";
        let bn = BooleanNetwork::try_from(model).unwrap();

        // try both versions with exhaustive results and without them (they execute different code)

        let formulae = vec!["!{x}: AG EF {x}".to_string(), "!{x}: AF {x}".to_string()];
        assert!(analyse_formulae(&bn, formulae, PrintOptions::WithProgress, None).is_ok());

        let formula = "erk & fgfr & ~shc".to_string(); // simple to avoid long prints
        assert!(analyse_formula(&bn, formula, PrintOptions::Exhaustive, None).is_ok());
    }
}
