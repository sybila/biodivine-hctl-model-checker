mod tokenizer;
mod parser;
mod operation_enums;
mod implementation;
mod evaluator;
mod compute_scc;
mod io;
mod infer_networks;

#[allow(unused_imports)]
use crate::io::{print_results_fast, print_results};
#[allow(unused_imports)]
use crate::tokenizer::{tokenize_recursive, print_tokens};
#[allow(unused_imports)]
use crate::compute_scc::write_attractors_to_file;
use crate::parser::parse_hctl_formula;
use crate::evaluator::eval_tree;
use crate::infer_networks::parse_and_infer;

use std::fs::read_to_string;
use std::convert::TryFrom;
use std::time::SystemTime;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

/* TODOs to implement for the model checking part */
// TODO: USE PROPER DUPLICATE MARKING AND IMPLEMENT PROPER CACHE FOR EVALUATOR
// TODO: SPECIAL CASES FOR EVALUATOR (attractors, stable states...)
// TODO: optims for evaluator
// TODO: separate function
// TODO: safe version for labeled_by (does not ignore error)
// TODO: iterator for GraphColoredVertices sets - we only have for vertices (or something like that)
// TODO: maybe - exact set size for GraphColoredVertices, GraphColors, GraphVertices - idk
// TODO: more efficient operators on GraphColoredVertices (like imp, xor, equiv)?
// TODO: printer for all correct valuations in all three color/vertex sets

/* BUGs to fix */
// TODO: FIX TOKENIZING - NAMES STARTING WITH A/E DOES NOT PARSE (because of AG, EF...)
// TODO: "!{var}: AG EF {var} & & !{var}: AG EF {var}" DOES NOT CAUSE ERROR
// TODO: "!{var}: AG EF {var} & !{var}: AG EF {var}" DOES NOT PARSE CORRECTLY
// TODO: check that w dont have formula like "!x (EF (!x x)) - same var quantified more times

/* TODOs to implement for the inference part */
// TODO: parse attractors from file
// TODO: implement both approaches (model-checking vs component-wise)
// TODO: create separate binaries
// TODO: printing satisfying BNs? or do something with the resulting colors

fn main() {
    let start = SystemTime::now();

    let formula = "!{var}: AG EF {var}".to_string();
    //let formula = "(3{x}: (@{x}: (aGO1 & ~aGO10 & ~aGO7 & aNT & aRF4 & ~aS1 & ~aS2 & eTT & FIL & KaN1 & miR165 & miR390 & ~ReV & ~TaS3siRNa & aGO1_miR165 & ~aGO7_miR390 & ~aS1_aS2 & aUXINh & ~CKh & ~GTe6 & ~IPT5 & (!{y}: AG EF {y})))) & (3{x}: (@{x}: (~aGO1 & aGO10 & aGO7 & aNT & ~aRF4 & aS1 & aS2 & ~eTT & ~FIL & ~KaN1 & ~miR165 & miR390 & ReV & TaS3siRNa & ~aGO1_miR165 & aGO7_miR390 & aS1_aS2 & aUXINh & CKh & GTe6 & IPT5 & (!{y}: AG EF {y}))))".to_string();
    let model_file = "test_model.aeon".to_string();
    let tokens = match tokenize_recursive(&mut formula.chars().peekable(), true) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    print_tokens(&tokens);

    match parse_hctl_formula(&tokens) {
        Ok(tree) => {
            println!("original formula: {}", tree.subform_str);
            let aeon_string = read_to_string(model_file).unwrap();
            let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
            let graph = SymbolicAsyncGraph::new(bn).unwrap();

            println!("Graph creation time: {}ms", start.elapsed().unwrap().as_millis());

            let result = eval_tree(tree, &graph);
            //write_attractors_to_file(&graph, "attractor_output.txt");

            println!("Computation time: {}ms", start.elapsed().unwrap().as_millis());
            println!("{} vars in network", graph.as_network().num_vars());
            //print_results(&graph, &result, true);
            print_results_fast(&result);
        },
        Err(message) => println!("{}", message),
    }
}
