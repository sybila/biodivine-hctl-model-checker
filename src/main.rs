mod tokenizer;
mod parser;
mod operation_enums;
mod implementation;
mod evaluator;
mod compute_scc;

use tokenizer::tokenize_recursive;
use parser::parse_hctl_formula;
use evaluator::{mark_duplicates, eval_node, minimize_number_of_state_vars};

use std::fs::read_to_string;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Write;
use std::time::SystemTime;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;


/* TODOs to implement in the whole project */
// TODO: IMPLEMENT CACHE FOR EVALUATOR
// TODO: SPECIAL CASES FOR EVALUATOR (attractors, stable states...)
// TODO: optims for evaluator
// TODO: safe version for labeled_by (does not ignore error)
// TODO: iterator for GraphColoredVertices sets - we only have for vertices (or something like that)
// TODO: maybe - exact set size for GraphColoredVertices, GraphColors, GraphVertices - idk
// TODO: more efficient operators on GraphColoredVertices (like imp, xor, equiv)?
// TODO: printer for all correct valuations in all three color/vertex sets

#[allow(dead_code)]
fn print_results_fast(results: &GraphColoredVertices) {
    println!("{} results in total", results.approx_cardinality());
    println!("{} colors in total", results.colors().approx_cardinality());
    println!("{} states in total", results.vertices().approx_cardinality());
}

#[allow(dead_code)]
fn print_results(
    graph: &SymbolicAsyncGraph,
    results: &GraphColoredVertices,
    show_names: bool
) -> () {
    // first print general info
    print_results_fast(results);

    let mut counter = 0;
    let network = graph.as_network();
    for valuation in results.vertices().materialize().iter() {
        // colored var names version
        if show_names {
            let mut i = 0;
            let variable_name_strings = network
                .variables()
                .map(|id| format!("\"{}\"", network.get_variable_name(id)));

            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            for var in variable_name_strings {
                if valuation.get(i) {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
                    write!(&mut stdout, "{} ", var).unwrap();
                } else {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();
                    write!(&mut stdout, "{} ", var).unwrap();
                }
                i += 1;
            }
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap();
            println!();
        }
        // just 0/1 valuation vector version
        else {
            let mut valuation_str = String::new();
            for j in 0..valuation.len() {
                valuation_str.push(if valuation.get(j) { '1' } else { '0' });
            }
            println!("{}", valuation_str.as_str());

        }
        counter += 1;
    }
    println!("{} result states found in total.", counter)
}

fn main() {
    let start = SystemTime::now();

    let formula : String = "!{var}: AG EF {var}".to_string();
    let filename : String = "models/[var27]__[id098]__[WG-SIGNALING-PATHWAY]/model.aeon".to_string();
    // let filename : String = "test_model.aeon".to_string();
    let tokens = match tokenize_recursive(&mut formula.chars().peekable(), true) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    // print_tokens(&tokens);

    match parse_hctl_formula(&tokens) {
        Ok(tree) => {
            println!("original formula: {}", tree.subform_str);
            let aeon_string = read_to_string(filename).unwrap();
            let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
            let graph = SymbolicAsyncGraph::new(bn).unwrap();

            println!("Graph creation time: {}ms", start.elapsed().unwrap().as_millis());

            let (new_tree, _) = minimize_number_of_state_vars(
                *tree, HashMap::new(), String::new(), 0);
            // println!("renamed formula: {}", new_tree.subform_str);

            let mut duplicates = mark_duplicates(&new_tree);
            let result = eval_node(new_tree, &graph, &mut duplicates);

            println!("Computation time: {}ms", start.elapsed().unwrap().as_millis());
            //print_results(&graph, &result, true);
            print_results_fast(&result);
        },
        Err(message) => println!("{}", message),
    }
}
