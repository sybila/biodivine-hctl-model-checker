//! Contains the functionality to search for duplicate sub-formulae in several formulae. This is
//! highly useful for memoization during evaluation.

use crate::evaluation::canonization::{get_canonical, get_canonical_and_mapping};
use crate::preprocessing::node::{HctlTreeNode, NodeType};

use crate::preprocessing::operator_enums::Atomic;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Check if there are some duplicate subtrees in a given formula syntax tree.
/// This function uses canonization and thus recognizes duplicates with differently named
/// variables (e.g., `AX {y}` and `AX {z}`).
/// Return the CANONICAL versions of duplicate sub-formulae + the number of their appearances.
///
/// Note that except for wild-card properties, most of the terminal nodes (props, vars, constants)
/// are not considered.
pub fn mark_duplicates_canonized_multiple(root_nodes: &Vec<HctlTreeNode>) -> HashMap<String, i32> {
    // go through each tree from top, use height to compare only the nodes with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)

    // duplicates and their counters
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    // queue of the nodes to yet traverse
    let mut heap_queue: BinaryHeap<&HctlTreeNode> = BinaryHeap::new();
    // set of strings of the nodes (with the same height) to compare
    let mut same_height_canonical_strings: HashSet<String> = HashSet::new();

    // find the maximal root height, and push each root node to the queue
    let mut last_height = 0;
    for root_node in root_nodes {
        let height = root_node.height;
        if height > last_height {
            last_height = height;
        }
        heap_queue.push(root_node);
    }

    // because we are traversing trees, we dont care about cycles
    while let Some(current_node) = heap_queue.pop() {
        // if current node is terminal, process it only if it represents the `wild-card prop`
        // other kinds of terminals are not worth to be considered and cached during eval
        if let NodeType::TerminalNode(atom) = &current_node.node_type {
            if let Atomic::WildCardProp(_) = atom {
            } else {
                continue;
            }
        }

        let mut skip = false;
        let (current_subform_canonical, renaming) =
            get_canonical_and_mapping(current_node.subform_str.clone());

        // only mark duplicates with at max 1 variable (to not cause var name collisions during caching)
        // todo: extend this for any number of variables
        if (last_height == current_node.height) & (renaming.len() <= 1) {
            // if we have saved some nodes of the same height, compare them with the current one
            for other_canonical_string in same_height_canonical_strings.clone() {
                if other_canonical_string == current_subform_canonical {
                    if duplicates.contains_key(&current_subform_canonical) {
                        duplicates.insert(
                            current_subform_canonical.clone(),
                            duplicates[&current_subform_canonical] + 1,
                        );
                    } else {
                        duplicates.insert(current_subform_canonical.clone(), 1);
                    }
                    skip = true; // skip the descendants of the duplicate current_node
                    break;
                }
            }

            // do not traverse subtree of the duplicate later (whole node is cached during eval)
            if skip {
                continue;
            }
            same_height_canonical_strings.insert(current_subform_canonical);
        } else {
            // we continue with node from lower level, so we empty the set of nodes to compare
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

/// Wrapper for duplicate marking (`mark_duplicates_canonized_multiple`) for a single formula.
pub fn mark_duplicates_canonized_single(root_node: &HctlTreeNode) -> HashMap<String, i32> {
    mark_duplicates_canonized_multiple(&vec![root_node.clone()])
}

/*
/// DEPRECATED VERSION THAT DOES NOT UTILIZE CANONIZATION, USE THE VERSION ABOVE
/// Check if there are some duplicate subtrees in the given syntax tree
/// Save the (raw) duplicate sub-formulae + the number of their appearances
/// This version does not consider canonical forms! - only recognizes fully identical duplicates
/// Note that terminal nodes (props, vars, constants) are not considered
pub fn mark_duplicates_deprecated(root_node: &HctlTreeNode) -> HashMap<String, i32> {
    // go through the nodes from top, use height to compare only those with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    let mut heap_queue: BinaryHeap<&HctlTreeNode> = BinaryHeap::new();

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
    use crate::evaluation::mark_duplicate_subform::{
        mark_duplicates_canonized_multiple, mark_duplicates_canonized_single,
    };
    use crate::preprocessing::parser::{
        parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula,
    };
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    fn test_duplicates_single_simple() {
        let formula = "!{x}: 3{y}: (AX {x} & AX {y})";
        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let tree = parse_and_minimize_hctl_formula(&bn, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);

        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    fn test_duplicates_single_complex() {
        let formula = "(!{x}: 3{y}: ((AG EF {x} & AG EF {y}) & (EF {y}))) & (!{z}: EF {z})";
        let expected_duplicates = HashMap::from([
            ("(Ag (Ef {var0}))".to_string(), 1),
            ("(Ef {var0})".to_string(), 2),
        ]);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let tree = parse_and_minimize_hctl_formula(&bn, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    /// Use multiple input formulae.
    fn test_duplicates_multiple() {
        let formulae = vec![
            "!{x}: 3{y}: (AX {x} & AX {y})",
            "!{x}: (AX {x})",
            "!{z}: AX {z}",
        ];
        let expected_duplicates = HashMap::from([
            ("(Ax {var0})".to_string(), 2),
            ("(Bind {var0}: (Ax {var0}))".to_string(), 1),
        ]);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let mut trees = Vec::new();
        for formula in formulae {
            let tree = parse_and_minimize_hctl_formula(&bn, formula).unwrap();
            trees.push(tree);
        }
        let duplicates = mark_duplicates_canonized_multiple(&trees);

        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Test that wild-card propositions are also detected correctly (opposed to other terminals).
    fn test_duplicates_wild_cards() {
        // define a placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%) & v1 & v1";
        let expected_duplicates = HashMap::from([("%subst%".to_string(), 1)]);

        let tree = parse_and_minimize_extended_formula(&bn, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }
}
