use crate::formula_evaluation::canonization::*;
use crate::formula_preprocessing::parser::{Node, NodeType};

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;

/// Checks if there are some duplicate subtrees in the given syntax tree
/// Marks canonized duplicate sub-formulae + the number of their appearances
/// Due to the canonization, things like EX{x} and EX{y} recognized as duplicates
/// Terminal nodes (props, vars, constants) are not considered - not worth caching
/// Only considers sub-formulae with max 1 variable (to not cause rename collisions during caching)
pub fn mark_duplicates_canonized(root_node: &Node) -> HashMap<String, i32> {
    // go through the nodes from top, use height to compare only those with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    let mut heap_queue: BinaryHeap<&Node> = BinaryHeap::new();

    let mut last_height = root_node.height.clone();
    let mut same_height_canonical_strings: HashSet<String> = HashSet::new();
    heap_queue.push(root_node);

    // because we are traversing a tree, we dont care about cycles
    while let Some(current_node) = heap_queue.pop() {
        // lets stop the process when we hit the first terminal node
        // terminals are not worth to mark as duplicates and use them for caching
        if current_node.height == 0 {
            break;
        }

        let mut skip = false;
        let (current_subform_canonical, renaming) =
            get_canonical_and_mapping(current_node.subform_str.clone());

        // only mark duplicates with at max 1 variable (to not cause var name collisions during caching)
        if (last_height == current_node.height) & (renaming.len() <= 1) {
            // if we have saved some nodes of the same height, compare them with the current one
            for other_canonical_string in same_height_canonical_strings.clone() {
                if other_canonical_string == current_subform_canonical {
                    if duplicates.contains_key(&current_subform_canonical) {
                        duplicates.insert(
                            current_subform_canonical.clone(),
                            duplicates[&current_subform_canonical] + 1
                        );
                    } else {
                        duplicates.insert(current_subform_canonical.clone(), 1);
                    }
                    skip = true; // skip the descendants of the duplicate current_node
                    break;
                }
            }

            // do not traverse subtree of the duplicate later (will be cached during eval)
            if skip {
                continue;
            }
            same_height_canonical_strings.insert(current_subform_canonical);
        } else {
            // we got node from lower level, so we empty the set of nodes to compare
            last_height = current_node.height;
            same_height_canonical_strings.clear();
            same_height_canonical_strings.insert(get_canonical(current_node.subform_str.clone()));
        }

        // add children of current node to the heap_queue
        match &current_node.node_type {
            NodeType::TerminalNode(_) => {}
            NodeType::UnaryNode(_, child) => {
                heap_queue.push(child);
            }
            NodeType::BinaryNode(_, left, right) => {
                heap_queue.push(left);
                heap_queue.push(right);
            }
            NodeType::HybridNode(_, _, child) => {
                heap_queue.push(child);
            }
        }
    }
    duplicates
}

/*
/// DEPRECATED VERSION THAT DOES NOT UTILIZE CANONIZATION, USE THE VERSION ABOVE
/// Checks if there are some duplicate subtrees in the given syntax tree
/// Marks (raw) duplicate sub-formulae + the number of their appearances
/// This version does not consider canonical forms! - only recognizes identical duplicates
/// Note that terminal nodes (props, vars, constants) are not considered - not worth
pub fn mark_duplicates_deprecated(root_node: &Node) -> HashMap<String, i32> {
    // go through the nodes from top, use height to compare only those with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    let mut heap_queue: BinaryHeap<&Node> = BinaryHeap::new();

    let mut last_height = root_node.height.clone();
    let mut same_height_node_strings: HashSet<String> = HashSet::new();
    heap_queue.push(root_node);

    // because we are traversing a tree, we dont care about cycles
    while let Some(current_node) = heap_queue.pop() {
        //println!("{}", current_node.subform_str.as_str());

        // lets stop the process when we hit the first terminal node
        // terminals are not worth to mark as duplicates and use them for caching
        if current_node.height == 0 {
            break;
        }

        let mut skip = false;
        if last_height == current_node.height {
            // if we have saved some nodes of the same height, compare them with the current one
            for other_node_string in same_height_node_strings.clone() {
                if other_node_string == current_node.subform_str.as_str() {
                    if duplicates.contains_key(current_node.subform_str.as_str()) {
                        duplicates
                            .insert(current_node.subform_str.clone(), duplicates[&current_node.subform_str] + 1);
                    } else {
                        duplicates.insert(current_node.subform_str.clone(), 1);
                    }
                    skip = true; // skip the descendants of the duplicate current_node
                    break;
                }
            }

            // do not traverse subtree of the duplicate later (will be cached during eval)
            if skip {
                continue;
            }
            same_height_node_strings.insert(current_node.subform_str.clone());
        } else {
            // we got node from lower level, so we empty the set of nodes to compare
            last_height = current_node.height;
            same_height_node_strings.clear();
            same_height_node_strings.insert(current_node.subform_str.clone());
        }

        // add children of current_node to the heap_queue
        match &current_node.node_type {
            NodeType::TerminalNode(_) => {}
            NodeType::UnaryNode(_, child) => {
                heap_queue.push(child);
            }
            NodeType::BinaryNode(_, left, right) => {
                heap_queue.push(left);
                heap_queue.push(right);
            }
            NodeType::HybridNode(_, _, child) => {
                heap_queue.push(child);
            }
        }
    }
    duplicates
}
 */

#[cfg(test)]
mod tests {
    use crate::formula_evaluation::mark_duplicate_subform::mark_duplicates_canonized;
    use crate::formula_preprocessing::parser::parse_hctl_formula;
    use crate::formula_preprocessing::vars_props_manipulation::check_props_and_rename_vars;
    use crate::formula_preprocessing::tokenizer::tokenize_formula;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones
    fn test_duplicates_simple() {
        let formula = "!{x}: 3{y}: (AX {x} & AX {y})".to_string();
        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();
        let modified_tree = check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).unwrap();
        let duplicates = mark_duplicates_canonized(&modified_tree);

        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones
    fn test_duplicates_complex() {
        let formula = "(!{x}: 3{y}: ((AG EF {x} & AG EF {y}) & (EF {y}))) & (!{z}: EF {z})".to_string();
        let expected_duplicates = HashMap::from([
            ("(Ag (Ef {var0}))".to_string(), 1),
            ("(Ef {var0})".to_string(), 2),
        ]);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();
        let modified_tree = check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).unwrap();
        let duplicates = mark_duplicates_canonized(&modified_tree);

        assert_eq!(duplicates, expected_duplicates);
    }
}
