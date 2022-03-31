#[allow(unused_imports)]
use hctl_model_checker::analysis::{analyse_formula, model_check_formula, PrintOptions};

use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

#[allow(dead_code)]
/// Optimised version - first evaluates formula for specific attractor existence, then (if we want
/// forbid all additional attractors) evaluates the formula for the attractor prohibition, this
/// time only on graph with colors restricted to those from the first part
fn perform_inference_with_attractors(
    data_set: Vec<String>,
    aeon_string: String,
    forbid_extra_attr: bool,
) {
    let start = SystemTime::now();
    let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    let graph = SymbolicAsyncGraph::new(bn).unwrap();

    // first the part that ensures attractor existence
    let mut formula_part1 = String::new();
    for attractor_state in data_set.clone() {
        if attractor_state.is_empty() {
            continue;
        }
        formula_part1.push_str(
            format!("(3{{x}}: (@{{x}}: {} & (AG EF ({})))) & ", attractor_state, attractor_state).as_str()
        )
    }
    formula_part1.push_str("true"); // just so we dont end with "&"
    let results_formula1 = model_check_formula(formula_part1, &graph);

    if forbid_extra_attr {
        // we will compute the part of formula which forbids additional attractors
        // we will use graph with colors RESTRICTED to those from formula 1
        let colors_formula1 = results_formula1.colors();
        let restricted_graph =
            SymbolicAsyncGraph::new_restrict_colors_from_existing(graph, &colors_formula1);

        // appendix for the formula which forbids additional attractors
        let mut formula_part2 = String::new();
        formula_part2.push_str("~(3{x}: (@{x}: ~(AG EF (");
        for attractor_state in data_set {
            if attractor_state.is_empty() {
                continue;
            }
            formula_part2.push_str(format!("({}) | ", attractor_state).as_str());
        }
        formula_part2.push_str("false ))))");

        let results_formula2 = model_check_formula(formula_part2, &restricted_graph);

        println!("{} colors (networks) found", results_formula2.colors().approx_cardinality())
    } else {
        println!("{} colors (networks) found", results_formula1.colors().approx_cardinality())
    }
    println!("Elapsed time: {}ms", start.elapsed().unwrap().as_millis());
}

/// Creates the formula describing specific attractor existence and if we want to forbid all
/// additional attractors, also forbids these.
/// Creates formula in the way so that AEON can be used for attractor computation
/// *SHOULD NOT BE USED* - use the function above
#[allow(dead_code)]
fn create_restricted_attractor_formula_aeon(data_set: Vec<String>) -> String {
    // basic version without forbidding additional attractors
    let mut formula = String::new();
    for attractor_state in data_set.clone() {
        if attractor_state.is_empty() {
            continue;
        }

        // We can create formula in more efficient ways - but if we want to use aeon, we must
        // compute formula "!y: AG EF y" anyway, and it can then be just cached
        formula.push_str(
            format!("(3{{x}}: (@{{x}}: {} & (!{{y}}: AG EF {{y}}))) & ", attractor_state).as_str()
        );
    }
    // appendix for the formula which forbids additional attractors
    formula.push_str(" & ~(3{x}: (@{x}: ");
    for attractor_state in data_set {
        if attractor_state.is_empty() {
            continue;
        }
        formula.push_str(format!("~(AG EF ( {} ))  & ", attractor_state).as_str());
    }
    formula.push_str("(!{y}: AG EF {y})))");

    formula
}

/// Creates the formula describing specific steady-states existence and if we want to forbid all
/// additional steady-states, also forbids these.
#[allow(dead_code)]
fn create_steady_state_formula(data_set: Vec<String>, forbid_extra_attr: bool) -> String {
    // basic version without forbidding additional attractors
    let mut formula = String::new();
    for attractor_state in data_set.clone() {
        if attractor_state.is_empty() {
            continue;
        }
        formula.push_str(format!("(3{{x}}: (@{{x}}: {} & (!{{y}}: AX ({{y}})))) & ", attractor_state).as_str())
    }
    formula.push_str("true"); // just so we dont end with "&"

    // (optional) appendix for the formula which forbids additional attractors
    if forbid_extra_attr {
        formula.push_str(" & ~(3{x}: (@{x}: ");
        for attractor_state in data_set {
            if attractor_state.is_empty() {
                continue;
            }
            formula.push_str(format!("~( {} )  & ", attractor_state).as_str())
        }
        formula.push_str("(!{y}: AX {y})))")
    }

    formula
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("3 arguments expected, got {}", args.len() - 1);
        println!("Usage: ./infer_networks model_file attractor_data forbid_extra_attrs");
        return;
    }
    if !(args[3].as_str() == "false" ||  args[3].as_str() == "true") {
        println!("Invalid argument \"{}\", it must be either \"true\" or \"false\"", args[3]);
        println!("Usage: ./infer_networks model_file attractor_data (true | false)");
        return;
    }
    let forbid_extra_attrs = match args[3].as_str() {
        "false" => false,
        _ => true  // we need match to be exhaustive
    };

    let data_file = File::open(Path::new(args[2].as_str())).unwrap();
    let reader = BufReader::new(&data_file);
    let data: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let aeon_string = read_to_string(args[1].clone()).unwrap();

    perform_inference_with_attractors(data, aeon_string, forbid_extra_attrs);

    /*
    // older version:
    //let formula = create_attractor_formula(data.clone(), forbid_extra_attrs);
    //let formula = create_steady_state_formula(data, forbid_extra_attrs);
    //analyse_formula(aeon_string, formula, PrintOptions::ShortPrint);
    //println!("original formula: {}", formula.clone());
    // result should have 2^(number of vars) states - basically all states
     */
}
