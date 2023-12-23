//! Contains the functionality to search for duplicate sub-formulae in several formulae. This is
//! highly useful for memoization during evaluation.

use crate::evaluation::canonization::{get_canonical, get_canonical_and_mapping};
use crate::preprocessing::node::{HctlTreeNode, NodeType};
use std::cmp::Ordering;

use crate::preprocessing::operator_enums::Atomic;
use std::collections::{BTreeMap, BinaryHeap, HashMap, HashSet};

/// Structure that holds a combination of a subtree and information about domains for all the free
/// HCTL variables in that subtree (if they are specified).
///
/// This structure enables to compare the trees (by their height) and is made to put the trees to
/// a HashSet (HashMap). The variable domains are needed if we are to compare two subtrees with
/// free variables for equivalence.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct NodeWithDomains<'a> {
    subtree: &'a HctlTreeNode,
    domains: BTreeMap<String, String>,
}

impl NodeWithDomains<'_> {
    pub fn new(subtree: &HctlTreeNode, domains: BTreeMap<String, String>) -> NodeWithDomains {
        NodeWithDomains { subtree, domains }
    }

    pub fn new_empty_doms(subtree: &HctlTreeNode) -> NodeWithDomains {
        NodeWithDomains {
            subtree,
            domains: BTreeMap::new(),
        }
    }
}

/// `NodeWithDomains` objects are ordered by the height of their subtrees.
impl PartialOrd for NodeWithDomains<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.subtree.height.cmp(&other.subtree.height))
    }
    fn lt(&self, other: &Self) -> bool {
        self.subtree.height.lt(&other.subtree.height)
    }
    fn le(&self, other: &Self) -> bool {
        self.subtree.height.le(&other.subtree.height)
    }
    fn gt(&self, other: &Self) -> bool {
        self.subtree.height.gt(&other.subtree.height)
    }
    fn ge(&self, other: &Self) -> bool {
        self.subtree.height.ge(&other.subtree.height)
    }
}

/// Nodes are ordered by their height.
impl Ord for NodeWithDomains<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.subtree.height.cmp(&other.subtree.height)
    }
}

/// Check if there are some duplicate subtrees in a given formula syntax tree.
/// This function uses canonization and thus recognizes duplicates with differently named
/// variables (e.g., `AX {y}` and `AX {z}`).
/// Return the CANONICAL versions of duplicate sub-formulae + the number of their appearances.
///
/// Note that except for wild-card properties, the terminal nodes (props, vars, constants)
/// are not considered.
pub fn mark_duplicates_canonized_multiple(root_nodes: &Vec<HctlTreeNode>) -> HashMap<String, i32> {
    // TODO: check and test if addition of restricted var domains does not break things
    // TODO: we must check that duplicates share var domains (in case that they contain free vars) - done?

    // go through each tree from top, use height to compare only the nodes with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)

    // duplicates and their counters
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    // queue of the nodes to yet traverse
    let mut heap_queue: BinaryHeap<NodeWithDomains> = BinaryHeap::new();
    // set with all relevant info for simple comparison of sub-formulas of the same height
    //  1) a canonical string of the sub-formula
    //  2) a mapping from each free variable to its corresponding domain
    let mut same_height_formulae: HashSet<(String, BTreeMap<String, String>)> = HashSet::new();

    // find the maximal root height, and push each root node to the queue
    let mut last_height = 0;
    for root_node in root_nodes {
        let height = root_node.height;
        if height > last_height {
            last_height = height;
        }
        heap_queue.push(NodeWithDomains::new_empty_doms(root_node));
    }

    // because we are traversing tree structures, we dont have to check for loops
    while let Some(current_node) = heap_queue.pop() {
        // if the node is terminal, process it only if it represents the `wild-card proposition`
        // other kinds of terminals are not worth to be considered and cached during evaluation
        if let NodeType::TerminalNode(atom) = &current_node.subtree.node_type {
            if let Atomic::WildCardProp(_) = atom {
            } else {
                continue;
            }
        }

        let mut skip_sub_tree = false; // we will skip traversing of duplicate sub-trees
                                       // get canonical substring and corresponding variable renaming map
        let (current_formula, renaming) =
            get_canonical_and_mapping(current_node.subtree.subform_str.clone());

        // we only mark duplicate formulae with at max 1 variable (to not cause var name collisions during caching)
        // todo: extend this for any number of variables
        if (last_height == current_node.subtree.height) & (renaming.len() <= 1) {
            // if we have saved some nodes of the same height, compare them with the current one
            for (other_formula, other_domains) in same_height_formulae.clone() {
                if other_formula == current_formula && other_domains == current_node.domains {
                    // increment the duplicate counter or add a new duplicate
                    if duplicates.contains_key(&current_formula) {
                        duplicates
                            .insert(current_formula.clone(), duplicates[&current_formula] + 1);
                    } else {
                        duplicates.insert(current_formula.clone(), 1);
                    }
                    skip_sub_tree = true; // skip the descendants of the duplicate current_node
                    break;
                }
            }

            // do not traverse subtree of the duplicate later (whole node is cached during eval)
            if skip_sub_tree {
                continue;
            }
            same_height_formulae.insert((current_formula, current_node.domains.clone()));
        } else {
            // we have a node from lower level, so we empty the current set of nodes to compare to
            last_height = current_node.subtree.height;
            same_height_formulae.clear();
            same_height_formulae.insert((
                get_canonical(current_node.subtree.subform_str.clone()),
                current_node.domains.clone(),
            ));
        }

        // add children of current node to the heap_queue
        match &current_node.subtree.node_type {
            NodeType::TerminalNode(_) => {}
            NodeType::UnaryNode(_, child) => {
                heap_queue.push(NodeWithDomains::new(child, current_node.domains.clone()));
            }
            NodeType::BinaryNode(_, left, right) => {
                heap_queue.push(NodeWithDomains::new(left, current_node.domains.clone()));
                heap_queue.push(NodeWithDomains::new(right, current_node.domains.clone()));
            }
            NodeType::HybridNode(_, variable, opt_domain, child) => {
                let mut child_w_domains = NodeWithDomains::new(child, current_node.domains.clone());
                // if the variable of the hybrid node has domain specified, add it
                if opt_domain.is_some() {
                    child_w_domains
                        .domains
                        .insert(variable.clone(), opt_domain.clone().unwrap());
                }
                heap_queue.push(child_w_domains);
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
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    fn test_duplicates_single_simple() {
        let formula = "!{x}: 3{y}: (AX {x} & AX {y})";
        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        let tree = parse_and_minimize_hctl_formula(&ctx, formula).unwrap();
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
        let ctx = SymbolicContext::new(&bn).unwrap();

        let tree = parse_and_minimize_hctl_formula(&ctx, formula).unwrap();
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
        let ctx = SymbolicContext::new(&bn).unwrap();

        let mut trees = Vec::new();
        for formula in formulae {
            let tree = parse_and_minimize_hctl_formula(&ctx, formula).unwrap();
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
        let ctx = SymbolicContext::new(&bn).unwrap();

        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%) & v1 & v1";
        let expected_duplicates = HashMap::from([("%subst%".to_string(), 1)]);

        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Test that duplicates with same duplicates are detected, but with different domains not.
    fn test_duplicates_domains() {
        // define a placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        // example to recognize the whole quantified sub-formulas with the same domains as duplicate
        let formula = "(!{x} in %d1%: AX {x}) & AX (!{x} in %d1%: AX {x}) & v1";
        let expected_duplicates =
            HashMap::from([("(Bind {var0} in %d1%: (Ax {var0}))".to_string(), 1)]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);

        // example to recognize the sub-formulas with free vars with the same domains as duplicate
        let formula = "(!{x} in %d1%: v1 & AX {x}) & AX (!{x} in %d1%: AX {x})";
        let expected_duplicates = HashMap::from([("(Ax {var0})".to_string(), 1)]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);

        // example with sub-formulae that cannot be duplicate due to domains
        let formula = "(!{x} in %d1%: AG EF {x}) & AX (!{x} in %d2%: AG EF {x}) & v1";
        let expected_duplicates = HashMap::new();
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }
}
