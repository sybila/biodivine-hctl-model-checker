mod tokenizer;
mod parser;
mod operation_enums;
mod implementation;
mod evaluator;

use tokenizer::tokenize_recursive;
use parser::parse_hctl_formula;
use evaluator::{mark_duplicates, eval_node};

use std::fs::read_to_string;
use std::convert::TryFrom;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;


// TODO: iterator for GraphColoredVertices sets - we only have one for vertices..
// TODO: maybe - exact set size for GraphColoredVertices, GraphColors, GraphVertices - idk
// TODO: better operators on GraphColoredVertices (like imp, xor, equiv)?
// TODO: printer for all correct valuations in all three color/vertex sets

fn main() {
    let formula : String = "~ EF coup_fti".to_string();
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
            println!("{}", tree.subform_str);
            match read_to_string(filename) {
                Ok(aeon_string) => {
                    match BooleanNetwork::try_from(aeon_string.as_str()) {
                        Ok(network) => {
                            match SymbolicAsyncGraph::new(network) {
                                Ok(graph) => {
                                    let mut duplicates = mark_duplicates(&*tree);
                                    let result = eval_node(*tree, &graph, &mut duplicates);
                                    // TODO - do something with the result
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
                                                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)));
                                                write!(&mut stdout, "{} ", var);
                                            }
                                            else {
                                                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)));
                                                write!(&mut stdout, "{} ", var);
                                            }
                                            i += 1;
                                        }
                                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
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
                                }
                                Err(message) => println!("{}", message),
                            }
                        }
                        Err(message) => println!("{}", message),
                    }
                }
                Err(message) => println!("{}", message),
            }

        },
        Err(message) => println!("{}", message),
    }
}
