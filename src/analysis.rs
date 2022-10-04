use crate::evaluator::eval_minimized_tree;
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

/// Renames hctl vars in the formula tree to canonical form - "x", "xx", ...
/// Works only for formulae without free variables
/// Renames as many state-vars as possible to identical names, without changing the semantics
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
/// (That is sufficient, since the formula has to be closed to be evaluated)
fn collect_unique_hctl_vars(formula_tree: Node, mut seen_vars: HashSet<String>) -> HashSet<String> {
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

/// Performs the whole model checking process, including parsing at the beginning
/// Prints selected amount of result info (no prints / summary / all results printed)
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
            println!("parsed formula:   {}", tree.subform_str);
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

/// Performs the model checking on GIVEN graph and returns result, no prints happen
/// UNSAFE - does not modify the graph based on formula (number of hctl vars, etc.),
/// Assumes that graph was created correctly (meaning graph's BDD must have enough HCTL variables)
/// Only use this function for testing and internal operations
pub fn model_check_formula_unsafe(
    formula: String,
    stg: &SymbolicAsyncGraph,
) -> GraphColoredVertices {
    let tokens = tokenize_formula(formula).unwrap();
    let tree = parse_hctl_formula(&tokens).unwrap();
    let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());
    eval_minimized_tree(modified_tree, stg)
}

#[cfg(test)]
mod tests {
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
    use biodivine_lib_param_bn::BooleanNetwork;
    use crate::analysis::model_check_formula_unsafe;

    // model FISSION-YEAST-2008
    const BNET_MODEL: &str = r"
targets,factors
Cdc25, ((!Cdc2_Cdc13 & (Cdc25 & !PP)) | ((Cdc2_Cdc13 & (!Cdc25 & !PP)) | (Cdc2_Cdc13 & Cdc25)))
Cdc2_Cdc13, (!Ste9 & (!Rum1 & !Slp1))
Cdc2_Cdc13_A, (!Ste9 & (!Rum1 & (!Slp1 & (!Wee1_Mik1 & Cdc25))))
PP, Slp1
Rum1, ((!SK & (!Cdc2_Cdc13 & (!Rum1 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP))))))))
SK, Start
Slp1, Cdc2_Cdc13_A
Start, 0
Ste9, ((!SK & (!Cdc2_Cdc13 & (!Ste9 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP))))))))
Wee1_Mik1, ((!Cdc2_Cdc13 & (!Wee1_Mik1 & PP)) | ((!Cdc2_Cdc13 & Wee1_Mik1) | (Cdc2_Cdc13 & (Wee1_Mik1 & PP))))
";

    #[test]
    /// Test evaluation of several important formulae on model FISSION-YEAST-2008
    /// Compare numbers of results with the numbers acquired by Python model checker or AEON
    fn test_model_check_basic_formulae() {
        let bn = BooleanNetwork::try_from_bnet(BNET_MODEL).unwrap();
        // test formulae use 3 HCTL vars at most
        let stg = SymbolicAsyncGraph::new(bn, 3).unwrap();

        let mut result = model_check_formula_unsafe("!{x}: AG EF {x}".to_string(), &stg);
        assert_eq!(76., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(76., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("!{x}: AX {x}".to_string(), &stg);
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("!{x}: AX EF {x}".to_string(), &stg);
        assert_eq!(132., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(132., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("AF (!{x}: AX {x})".to_string(), &stg);
        assert_eq!(60., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(60., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe(
            "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string(),
            &stg,
        );
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe(
            "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))".to_string(),
            &stg,
        );
        assert_eq!(11., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(11., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("!{x}: (AX (AF {x}))".to_string(), &stg);
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe("AF (!{x}: (AX (~{x} & AF {x})))".to_string(), &stg);
        assert_eq!(0., result.approx_cardinality());
        assert_eq!(0., result.colors().approx_cardinality());
        assert_eq!(0., result.vertices().approx_cardinality());

        result = model_check_formula_unsafe(
            "AF (!{x}: ((AX (~{x} & AF {x})) & (EF (!{y}: EX ~AF {y}))))".to_string(),
            &stg,
        );
        assert_eq!(0., result.approx_cardinality());
        assert_eq!(0., result.colors().approx_cardinality());
        assert_eq!(0., result.vertices().approx_cardinality());
    }
}