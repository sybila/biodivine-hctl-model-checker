mod tokenizer;
mod parser;
mod operation_enums;
mod implementation;
mod evaluator;

use tokenizer::{tokenize_recursive, print_tokens};
use parser::parse_hctl_formula;
use evaluator::{mark_duplicates, eval_node};

use std::env;
use std::fs::read_to_string;
use std::convert::TryFrom;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

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
                                    eval_node(*tree, &graph, &mut duplicates);
                                    // TODO - do something with it
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
