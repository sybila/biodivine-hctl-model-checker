mod tokenizer;
mod parser;
mod operation_enums;
mod implementation;
mod evaluator;

use tokenizer::tokenize_recursive;
use parser::parse_hctl_formula;
use evaluator::{mark_duplicates, eval_node, minimize_number_of_state_vars};

use std::fs::read_to_string;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

// TODO: safe version for labeled_by (does not ignore error)
// TODO: iterator for GraphColoredVertices sets - we only have one for vertices..
// TODO: maybe - exact set size for GraphColoredVertices, GraphColors, GraphVertices - idk
// TODO: better operators on GraphColoredVertices (like imp, xor, equiv)?
// TODO: printer for all correct valuations in all three color/vertex sets

fn main() {
    let formula : String = "!{var}: EF {var}".to_string();
    let filename : String = "models/[var5]__[id007]__[CORTICAL-AREA-DEVELOPMENT]/model.aeon".to_string();
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

            let (new_tree, _) = minimize_number_of_state_vars(
                *tree, HashMap::new(), String::new(), 0);
            println!("formula with renamed vars: {}", new_tree.subform_str);

            let mut duplicates = mark_duplicates(&new_tree);
            let result = eval_node(new_tree, &graph, &mut duplicates);

            let network = graph.as_network();

            let mut counter = 0;
            for valuation in result.vertices().materialize().iter() {
                // colored var names version
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

                /*
                // just 0/1 valuation vector version
                let mut valuation_str = String::new();
                for i in 0..valuation.len() {
                    valuation_str.push(if valuation.get(i) { '1' } else { '0' });
                }
                println!("{}", valuation_str.as_str());
                 */
                counter += 1;
            }
            println!("{} result states found in total.", counter)
        },
        Err(message) => println!("{}", message),
    }
}
