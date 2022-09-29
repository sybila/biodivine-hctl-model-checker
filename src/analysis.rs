use crate::evaluator::eval_minimized_tree;
#[allow(unused_imports)]
use crate::io::{print_results, print_results_fast};
use crate::operation_enums::*;
use crate::parser::*;
#[allow(unused_imports)]
use crate::tokenizer::{print_tokens, tokenize_formula};

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::time::SystemTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrintOptions {
    NoPrint,
    ShortPrint,
    LongPrint,
}

/// renames vars to canonical form of "x", "xx", ...
/// works only FOR FORMULAS WITHOUT FREE VARIABLES
/// renames as many state-vars as possible to the identical names, without changing the formula
fn minimize_number_of_state_vars(
    orig_node: Node,
    mut mapping_dict: HashMap<String, String>,
    mut last_used_name: String,
) -> Node {
    // If we find hybrid node with bind or exist, we add new var-name to rename_dict and stack (x, xx, xxx...)
    // After we leave this binder/exist, we remove its var from rename_dict
    // When we find terminal with free var or jump node, we rename the var using rename-dict
    return match orig_node.node_type {
        // rename vars in terminal state-var nodes
        NodeType::TerminalNode(ref atom) => match atom {
            Atomic::Var(name) => {
                let renamed_var = mapping_dict.get(name.as_str()).unwrap();
                Node {
                    subform_str: format!("{{{}}}", renamed_var.to_string()),
                    height: 0,
                    node_type: NodeType::TerminalNode(Atomic::Var(renamed_var.to_string())),
                }
            }
            _ => return orig_node,
        },
        // just dive one level deeper for unary nodes, and rename string
        NodeType::UnaryNode(op, child) => {
            let node = minimize_number_of_state_vars(*child, mapping_dict, last_used_name.clone());
            create_unary(Box::new(node), op)
        }
        // just dive deeper for binary nodes, and rename string
        NodeType::BinaryNode(op, left, right) => {
            let node1 =
                minimize_number_of_state_vars(*left, mapping_dict.clone(), last_used_name.clone());
            let node2 = minimize_number_of_state_vars(*right, mapping_dict, last_used_name);
            create_binary(Box::new(node1), Box::new(node2), op)
        }
        // hybrid nodes are more complicated
        NodeType::HybridNode(op, var, child) => {
            // if we hit binder or exist, we are adding its new var name to dict & stack
            // no need to do this for jump, jump is not quantifier
            match op {
                HybridOp::Bind | HybridOp::Exist => {
                    last_used_name.push('x'); // this represents adding to stack
                    mapping_dict.insert(var.clone(), last_used_name.clone());
                }
                _ => {}
            }

            // dive deeper
            let node =
                minimize_number_of_state_vars(*child, mapping_dict.clone(), last_used_name.clone());

            // rename the variable in the node
            let renamed_var = mapping_dict.get(var.as_str()).unwrap();
            create_hybrid(Box::new(node), renamed_var.clone(), op)
        }
    };
}

/// Returns the set of all uniquely named HCTL variables in the formula tree
/// Variable names are collected from BIND and EXIST quantifiers
/// That is sufficient, since the formula have to be closed
fn collect_unique_hctl_vars(
    formula_tree: Node,
    mut seen_vars: HashSet<String>
) -> HashSet<String> {
    match formula_tree.node_type {
        NodeType::TerminalNode(_) => {}
        NodeType::UnaryNode(_, child) => {
            seen_vars.extend(collect_unique_hctl_vars(*child, seen_vars.clone()));
        }
        NodeType::BinaryNode(_, left, right) => {
            seen_vars.extend(collect_unique_hctl_vars(*left, seen_vars.clone()));
            seen_vars.extend(collect_unique_hctl_vars(*right, seen_vars.clone()));
        }
        // collect variables from exist and bind nodes
        NodeType::HybridNode(op, var_name, child) => {
            match op {
                HybridOp::Bind | HybridOp::Exist => {
                    seen_vars.insert(var_name); // we do not care whether insert is successful
                }
                _ => {}
            }
            seen_vars.extend(collect_unique_hctl_vars(*child, seen_vars.clone()));
        }
    }
    seen_vars
}

/// Performs the whole model checking process, including parsing of formula and model
/// Creates only as many HCTL vars as is needed
/// Prints selected amount of results (no prints / summary prints / all results printed)
pub fn analyse_formula(aeon_string: String, formula: String, print_option: PrintOptions) {
    let start = SystemTime::now();

    let tokens = match tokenize_formula(formula) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    //print_tokens(&tokens);

    match parse_hctl_formula(&tokens) {
        Ok(tree) => {
            println!("original formula: {}", tree.subform_str);
            let new_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());
            println!("modified formula: {}", new_tree.subform_str);

            // count the number of needed HCTL vars and instantiate graph with it
            let num_hctl_vars = collect_unique_hctl_vars(new_tree.clone(), HashSet::new()).len();
            let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
            let graph = SymbolicAsyncGraph::new(bn, num_hctl_vars as i16).unwrap();

            if print_option != PrintOptions::NoPrint {
                println!(
                    "Formula parse + graph build time: {}ms",
                    start.elapsed().unwrap().as_millis()
                );
            }

            let result = eval_minimized_tree(new_tree, &graph);
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

/// Just performs the model checking on GIVEN graph and returns result, no prints happen
/// UNSAFE - does not parse the graph from formula, assumes that graph was created correctly
/// Graph must have enough HCTL variables for the formula
pub fn model_check_formula_unsafe(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> GraphColoredVertices {
    let tokens = tokenize_formula(formula).unwrap();
    let tree = parse_hctl_formula(&tokens).unwrap();
    let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());
    eval_minimized_tree(modified_tree, stg)
}
