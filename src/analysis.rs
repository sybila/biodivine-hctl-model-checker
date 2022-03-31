use crate::evaluator::eval_tree;
#[allow(unused_imports)]
use crate::io::{print_results, print_results_fast};
use crate::parser::parse_hctl_formula;
#[allow(unused_imports)]
use crate::tokenizer::{print_tokens, tokenize_recursive};

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

use std::convert::TryFrom;
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintOptions {
    NoPrint,
    ShortPrint,
    LongPrint
}

/// Performs the whole model checking process, including parsing of formula and model
pub fn model_check_property(
    aeon_string: String,
    formula: String,
    print_option: PrintOptions
) -> () {
    let start = SystemTime::now();

    let tokens = match tokenize_recursive(&mut formula.chars().peekable(), true) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    //print_tokens(&tokens);

    match parse_hctl_formula(&tokens) {
        Ok(tree) => {
            //println!("original formula: {}", tree.subform_str);
            let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
            let graph = SymbolicAsyncGraph::new(bn).unwrap();

            if print_option != PrintOptions::NoPrint {
                println!(
                    "Graph build time: {}ms",
                    start.elapsed().unwrap().as_millis()
                );
            }

            let result = eval_tree(tree, &graph);
            //write_attractors_to_file(&graph, "attractor_output.txt");

            if print_option != PrintOptions::NoPrint {
                println!("Eval time: {}ms", start.elapsed().unwrap().as_millis());
                println!("{} vars in network", graph.as_network().num_vars());
            }

            match print_option {
                PrintOptions::LongPrint => print_results(&graph, &result, true),
                PrintOptions::ShortPrint => print_results_fast(&result),
                _ => {}
            }
        }
        Err(message) => println!("{}", message),
    }
}