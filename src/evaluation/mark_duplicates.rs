//! Contains the functionality to search for duplicate sub-formulae in several formulae. This is
//! highly useful for memoization during evaluation.

use crate::evaluation::canonization::get_canonical_and_renaming;
use crate::evaluation::{FormulaWithDomains, VarDomainMap};
use crate::preprocessing::hctl_tree::{HctlTreeNode, NodeType};
use crate::preprocessing::operator_enums::Atomic;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Structure that holds a combination of a subtree and information about domains for all the free
/// HCTL variables in that subtree (if they are specified).
///
/// This structure enables to compare the trees (by their height) and is made to put the tree references
/// to a HashSet (HashMap). The variable domains are needed if we are to compare two subtrees with
/// free variables for equivalence.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct NodeWithDomains<'a> {
    subtree: &'a HctlTreeNode,
    domains: VarDomainMap,
}

impl NodeWithDomains<'_> {
    pub fn new(subtree: &HctlTreeNode, domains: VarDomainMap) -> NodeWithDomains {
        NodeWithDomains { subtree, domains }
    }

    pub fn new_empty_doms(subtree: &HctlTreeNode) -> NodeWithDomains {
        NodeWithDomains {
            subtree,
            domains: VarDomainMap::new(),
        }
    }
}

/// Nodes are ordered by their height, with atomic propositions being the "smallest".
///
/// Note that while this sort is "total" in the sense that every pair of nodes can be compared,
/// there are many "semantically equivalent" nodes that have the same height.
impl Ord for NodeWithDomains<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.subtree.height.cmp(&other.subtree.height)
    }
}

/// Nodes are ordered by their height, with atomic propositions being the "smallest".
impl PartialOrd for NodeWithDomains<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Check if there are some duplicate subtrees in a given formula syntax tree.
/// This function uses canonization and thus recognizes duplicates with differently named
/// variables (e.g., `AX {y}` and `AX {z}`).
/// Return the CANONICAL versions of duplicate sub-formulae + the number of their appearances.
///
/// Note that except for wild-card properties, the terminal nodes (props, vars, constants)
/// are not considered.
pub fn mark_duplicates_canonized_multiple(
    root_nodes: &Vec<HctlTreeNode>,
) -> HashMap<FormulaWithDomains, i32> {
    // go through each tree from top, use height to compare only the nodes with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)

    // duplicate (canonical) formulae with their free (canonical) variable's domains, and their counters
    let mut duplicates: HashMap<FormulaWithDomains, i32> = HashMap::new();
    // queue of the nodes to yet traverse
    let mut heap_queue: BinaryHeap<NodeWithDomains> = BinaryHeap::new();
    // set with all relevant info for simple comparison of sub-formulas of the same height
    //  1) a canonical string of the sub-formula
    //  2) a mapping from each (canonical) free variable in the sub-formula to its corresponding domain
    let mut same_height_formulae: HashSet<FormulaWithDomains> = HashSet::new();

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
        if let NodeType::Terminal(atom) = &current_node.subtree.node_type {
            if let Atomic::WildCardProp(_) = atom {
            } else {
                continue;
            }
        }

        // get canonical version of the current formula and corresponding variable renaming map
        let (current_formula, renaming) =
            get_canonical_and_renaming(current_node.subtree.to_string());

        // rename the variables in the domain map to their canonical form (duplicate formulae are always canonical)
        // only include the FREE canonical variables that are actually contained in the sub-formula
        // example: given "!{x}:!{y}: (AX {y})", its sub-formula "AX {x}" would have one "None" domain for "var0"
        let mut canonical_domains: VarDomainMap = VarDomainMap::new();
        for (variable, domain) in &current_node.domains {
            if renaming.contains_key(variable) {
                canonical_domains.insert(renaming.get(variable).unwrap().clone(), domain.clone());
            }
        }

        // canonical version of the current formula and canonized mappings of its domains
        let current_formula_with_domains = (current_formula.clone(), canonical_domains.clone());

        let mut skip_sub_tree = false; // we will skip traversing of duplicate sub-trees
        if last_height == current_node.subtree.height {
            // if we have the node with the same height as all the saved nodes, we can compare them

            // we only mark duplicate formulae with at max 1 variable (to not cause var name collisions during caching)
            // todo: extend this for any number of variables
            if renaming.len() <= 1 {
                // if we have saved some nodes of the same height, compare them with the current one
                for other_formula_with_domains in same_height_formulae.clone() {
                    if other_formula_with_domains == current_formula_with_domains {
                        // increment the duplicate counter or add a new duplicate
                        if duplicates.contains_key(&current_formula_with_domains) {
                            duplicates.insert(
                                current_formula_with_domains.clone(),
                                duplicates[&current_formula_with_domains] + 1,
                            );
                        } else {
                            duplicates.insert(current_formula_with_domains.clone(), 1);
                        }
                        skip_sub_tree = true; // skip the descendants of the duplicate current_node
                        break;
                    }
                }
            }

            // do not traverse subtree of the duplicate later (whole node is cached during eval)
            if skip_sub_tree {
                continue;
            }
            same_height_formulae.insert(current_formula_with_domains);
        } else {
            // we have a node from lower level, so we empty the current set of nodes to compare to
            last_height = current_node.subtree.height;
            same_height_formulae.clear();
            same_height_formulae.insert(current_formula_with_domains);
        }

        // add children of current node to the heap_queue
        match &current_node.subtree.node_type {
            NodeType::Terminal(_) => {}
            NodeType::Unary(_, child) => {
                heap_queue.push(NodeWithDomains::new(child, current_node.domains.clone()));
            }
            NodeType::Binary(_, left, right) => {
                heap_queue.push(NodeWithDomains::new(left, current_node.domains.clone()));
                heap_queue.push(NodeWithDomains::new(right, current_node.domains.clone()));
            }
            NodeType::Hybrid(_, variable, domain, child) => {
                let mut child_w_domains = NodeWithDomains::new(child, current_node.domains.clone());
                // add the domain of the new quantified variable to the domain list
                child_w_domains
                    .domains
                    .insert(variable.clone(), domain.clone());
                heap_queue.push(child_w_domains);
            }
        }
    }
    duplicates
}

/// Wrapper for duplicate marking for a single formula.
///
/// See [mark_duplicates_canonized_multiple] for more details.
pub fn mark_duplicates_canonized_single(
    root_node: &HctlTreeNode,
) -> HashMap<FormulaWithDomains, i32> {
    mark_duplicates_canonized_multiple(&vec![root_node.clone()])
}

#[cfg(test)]
mod tests {
    use crate::evaluation::mark_duplicates::{
        mark_duplicates_canonized_multiple, mark_duplicates_canonized_single,
    };
    use crate::evaluation::VarDomainMap;
    use crate::preprocessing::parser::{
        parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula,
    };
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    fn duplicates_single_simple() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        let formula = "!{x}: 3{y}: (AX {x} & AX {y})";
        let domains = VarDomainMap::from([("var0".to_string(), None)]);
        let expected_duplicates = HashMap::from([(("(AX {var0})".to_string(), domains), 1)]);

        let tree = parse_and_minimize_hctl_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);

        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    fn duplicates_single_complex() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        let formula = "(!{x}: 3{y}: ((AG EF {x} & AG EF {y}) & (EF {y}))) & (!{z}: EF {z})";
        let duplicate_domains = VarDomainMap::from([("var0".to_string(), None)]);
        let duplicate_part1 = "(AG (EF {var0}))".to_string();
        let duplicate_part2 = "(EF {var0})".to_string();
        let expected_duplicates = HashMap::from([
            ((duplicate_part1, duplicate_domains.clone()), 1),
            ((duplicate_part2, duplicate_domains), 2),
        ]);
        let tree = parse_and_minimize_hctl_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Compare automatically detected duplicate sub-formulae to expected ones.
    /// Use multiple input formulae.
    fn duplicates_multiple_formulae() {
        let formulae = vec![
            "!{x}: 3{y}: (AX {x} & AX {y})",
            "!{x}: (AX {x})",
            "!{z}: AX {z}",
        ];
        let duplicate_domains = VarDomainMap::from([("var0".to_string(), None)]);
        let duplicate_part1 = "(AX {var0})".to_string();
        let duplicate_part2 = "(!{var0}: (AX {var0}))".to_string();
        let expected_duplicates = HashMap::from([
            ((duplicate_part1, duplicate_domains), 2),
            ((duplicate_part2, VarDomainMap::new()), 1),
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
    fn duplicates_with_wild_cards() {
        // define a placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %s%) & (@{y}: %s%) & v1 & v1";
        let expected_duplicates = HashMap::from([(("%s%".to_string(), VarDomainMap::new()), 1)]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);

        let formula = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & %s%) & EF ({y} & %s%) & AX (EF ({x} & %s%) ^ EF ({y} & %s%))";
        let domains = VarDomainMap::from([("var0".to_string(), None)]);
        let expected_duplicates = HashMap::from([
            (("(EF ({var0} & %s%))".to_string(), domains.clone()), 3),
            (("(AX {var0})".to_string(), domains), 1),
        ]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }

    #[test]
    /// Test that duplicates with same duplicates are detected, but with different domains not.
    fn duplicates_with_domains() {
        // define a placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        // example to recognize the whole quantified sub-formulas with the same domains as duplicate
        let formula = "(!{x} in %d1%: AX {x}) & AX (!{x} in %d1%: AX {x}) & v1";
        let duplicate_part = "(!{var0} in %d1%: (AX {var0}))".to_string();
        let expected_duplicates = HashMap::from([((duplicate_part, VarDomainMap::new()), 1)]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);

        // example to recognize the sub-formulas with free vars with the same domains as duplicate
        let formula = "(!{x} in %d1%: v1 & AX {x}) & AX (!{x} in %d1%: AX {x})";
        let duplicate_domains = VarDomainMap::from([("var0".to_string(), Some("d1".to_string()))]);
        let expected_duplicates =
            HashMap::from([(("(AX {var0})".to_string(), duplicate_domains), 1)]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);

        // example with sub-formulae that cannot be duplicate due to domains
        let formula = "(!{x} in %d1%: AG EF {x}) & AX (!{x} in %d2%: AG EF {x}) & v1";
        let expected_duplicates = HashMap::new();
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);

        // example combining the cases when the same sub-formulae are and are not duplicate (due to domains)
        let formula =
            "(!{x} in %d1%: AX {x}) & (!{x} in %d1%: (!{y} in %d2%: (AX {y}) & (AX {x})))";
        let duplicate_domains = VarDomainMap::from([("var0".to_string(), Some("d1".to_string()))]);
        let expected_duplicates =
            HashMap::from([(("(AX {var0})".to_string(), duplicate_domains), 1)]);
        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        let duplicates = mark_duplicates_canonized_single(&tree);
        assert_eq!(duplicates, expected_duplicates);
    }
}
