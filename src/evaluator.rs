use crate::compute_scc::compute_terminal_scc;
use crate::implementation_components::*;
use crate::operation_enums::*;
use crate::parser::{Node, NodeType};

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Peekable;
use std::str::Chars;

/// Prepares formula `tree` for efficient evaluation (duplicates & cache)
/// and then evaluates it on the given `graph`
pub fn eval_minimized_tree(tree: Node, graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    let mut duplicates = mark_duplicates(&tree);
    let mut cache: HashMap<String, GraphColoredVertices> = HashMap::new();
    let fixed_points = compute_fixed_points(graph);
    eval_node(tree, &graph, &mut duplicates, &mut cache, &fixed_points)
}

/// Evaluates the formula sub-tree `node` on the given `graph`
/// Uses pre-computed set of `duplicate` sub-formulae to allow for caching
/// TODO: fix cache
pub fn eval_node(
    node: Node,
    graph: &SymbolicAsyncGraph,
    duplicates: &mut HashMap<String, i32>,
    cache: &mut HashMap<String, GraphColoredVertices>,
    fixed_points: &GraphColoredVertices
) -> GraphColoredVertices {
    // first check whether this node does not belong in the duplicates
    let mut save_to_cache = false;
    if duplicates.contains_key(node.subform_str.as_str()) {
        if cache.contains_key(node.subform_str.as_str()) {
            // decrement number of duplicates left
            *duplicates.get_mut(node.subform_str.as_str()).unwrap() -= 1;

            // we will get bdd, but sometimes we must rename its vars
            // because it might have differently named state variables before
            let result = cache.get(node.subform_str.as_str()).unwrap().clone();

            // if we already visited all of the duplicates, lets delete the cached value
            if duplicates[node.subform_str.as_str()] == 0 {
                duplicates.remove(node.subform_str.as_str());
                cache.remove(node.subform_str.as_str());
            }
            // since we are working with canonical cache, we must rename vars in result bdd
            return result;
        } else {
            // if the cache does not contain result for this subformula, set insert flag
            save_to_cache = true;
        }
    }

    // first lets check for special cases, which can be optimised:
    // attractors
    if is_attractor_pattern(node.clone()) {
        let result = compute_terminal_scc(graph, graph.mk_unit_colored_vertices());
        if save_to_cache {
            cache.insert(node.subform_str.clone(), result.clone());
        }
        return result;
    }
    // fixed-points
    if is_fixed_point_pattern(node.clone()) {
        return fixed_points.clone();
    }

    let result = match node.node_type {
        NodeType::TerminalNode(atom) => match atom {
            Atomic::True => graph.mk_unit_colored_vertices(),
            Atomic::False => graph.mk_empty_vertices(),
            Atomic::Var(name) => create_comparator(graph, name.as_str()),
            Atomic::Prop(name) => labeled_by(graph, &name),
        },
        NodeType::UnaryNode(op, child) => match op {
            UnaryOp::Not => negate_set(graph, &eval_node(*child, graph, duplicates, cache, fixed_points)),
            UnaryOp::Ex => ex(graph, &eval_node(*child, graph, duplicates, cache, fixed_points), fixed_points),
            UnaryOp::Ax => ax(graph, &eval_node(*child, graph, duplicates, cache, fixed_points)),
            UnaryOp::Ef => ef_saturated(graph, &eval_node(*child, graph, duplicates, cache, fixed_points)),
            UnaryOp::Af => af(graph, &eval_node(*child, graph, duplicates, cache, fixed_points)),
            UnaryOp::Eg => eg(graph, &eval_node(*child, graph, duplicates, cache, fixed_points)),
            UnaryOp::Ag => ag(graph, &eval_node(*child, graph, duplicates, cache, fixed_points)),
        },
        NodeType::BinaryNode(op, left, right) => match op {
            BinaryOp::And => eval_node(*left, graph, duplicates, cache, fixed_points)
                .intersect(&eval_node(*right, graph, duplicates, cache, fixed_points)),
            BinaryOp::Or => eval_node(*left, graph, duplicates, cache, fixed_points)
                .union(&eval_node(*right, graph, duplicates, cache, fixed_points)),
            BinaryOp::Xor => non_equiv(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
            BinaryOp::Imp => imp(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
            BinaryOp::Iff => equiv(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
            BinaryOp::Eu => eu_saturated(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
            BinaryOp::Au => au(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
            BinaryOp::Ew => ew(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
            BinaryOp::Aw => aw(
                graph,
                &eval_node(*left, graph, duplicates, cache, fixed_points),
                &eval_node(*right, graph, duplicates, cache, fixed_points),
            ),
        },
        NodeType::HybridNode(op, var, child) => match op {
            HybridOp::Bind => bind(
                graph,
                &eval_node(*child, graph, duplicates, cache, fixed_points),
                var.as_str(),
            ),
            HybridOp::Jump => jump(
                graph,
                &eval_node(*child, graph, duplicates, cache, fixed_points),
                var.as_str(),
            ),
            HybridOp::Exist => existential(
                graph,
                &eval_node(*child, graph, duplicates, cache, fixed_points),
                var.as_str(),
            ),
        },
    };

    if save_to_cache {
        cache.insert(node.subform_str.clone(), result.clone());
    }
    result
}

/// checks whether node represents formula for attractors !{x}: AG EF {x}
fn is_attractor_pattern(node: Node) -> bool {
    return match node.node_type {
        NodeType::HybridNode(HybridOp::Bind, var1, child1) => match (*child1).node_type {
            NodeType::UnaryNode(UnaryOp::Ag, child2) => match (*child2).node_type {
                NodeType::UnaryNode(UnaryOp::Ef, child3) => match (*child3).node_type {
                    NodeType::TerminalNode(Atomic::Var(var2)) => var1 == var2,
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        },
        _ => false,
    };
}

/// checks whether node represents formula for fixed-points !{x}: AX {x}
fn is_fixed_point_pattern(node: Node) -> bool {
    return match node.node_type {
        NodeType::HybridNode(HybridOp::Bind, var1, child1) => match (*child1).node_type {
            NodeType::UnaryNode(UnaryOp::Ax, child2) => match (*child2).node_type {
                NodeType::TerminalNode(Atomic::Var(var2)) => var1 == var2,
                _ => false,
            },
            _ => false,
        },
        _ => false,
    };
}

/// returns string representing the same subformula, but with canonized var names (var0, var1...)
/// subform must be valid HCTL formula, minimized by minimize_number_of_state_vars function
/// subform MUST include all PARENTHESES and MUST NOT include excess spaces
/// for example "(3{x}:(3{xx}:((@{x}:((~{xx})&&(AX{x})))&&(@{xx}:(AX{xx})))))" is valid input
/// any node.subform_string field should be OK to use
fn canonize_subform(
    mut subform_chars: Peekable<Chars>,
    mut mapping_dict: HashMap<String, String>,
    mut canonical: String,
    mut stack_len: i32,
) -> (Peekable<Chars>, String, HashMap<String, String>, i32) {
    while let Some(ch) = subform_chars.next() {
        let mut should_return = false;
        match ch {
            // dive deeper by one level
            '(' => {
                canonical.push(ch);
                let tuple = canonize_subform(subform_chars, mapping_dict, canonical, stack_len);
                subform_chars = tuple.0;
                canonical = tuple.1;
                mapping_dict = tuple.2;
                stack_len = tuple.3;
            }
            // emerge back to upper level
            ')' => {
                canonical.push(ch);
                should_return = true;
            }
            // introduce new 'quantified' var (jump is not listed as it does not introduce vars)
            // we must distinguish situations where '3' is existential and when it is part of some prop name
            '!' | '3' if subform_chars.peek() == Some(&'{') => {
                // move to the beginning of the var name (skip '{')
                subform_chars.next();
                let mut var_name = String::new();
                while let Some(name_char) = subform_chars.next() {
                    if name_char == '}' {
                        break;
                    }
                    var_name.push(name_char);
                }
                // skip ':'
                subform_chars.next();
                // insert new mapping to dict and push it all to canonical string
                mapping_dict.insert(var_name.clone(), format!("var{}", stack_len));
                canonical.push_str(format!("{}{{{}}}:", ch, format!("var{}", stack_len)).as_str());
                stack_len += 1;
            }
            // rename existing var to canonical form, or handle free variables
            // this includes variable names which are part of the "jump operator"
            '{' => {
                let mut var_name = String::new();
                while let Some(name_char) = subform_chars.next() {
                    if name_char == '}' {
                        break;
                    }
                    var_name.push(name_char);
                }

                // we must be prepared for free vars to appear (not bounded by hybrid operators)
                // it is because we are canonizing all subformulas in the tree
                if !mapping_dict.contains_key(var_name.as_str()) {
                    mapping_dict.insert(var_name.clone(), format!("var{}", stack_len));
                    stack_len += 1;
                }

                if let Some(canonical_name) = mapping_dict.get(var_name.as_str()) {
                    canonical.push_str(format!("{{{}}}", canonical_name).as_str());
                } else {
                    // This branch should never happen
                    println!(
                        "{}",
                        format!("Canonical name was not found for {}", var_name)
                    );
                }
            }
            // all the other character, including boolean+temporal operators, '@', prop names
            _ => {
                canonical.push(ch);
            }
        }
        if should_return {
            break;
        }
    }
    (subform_chars, canonical, mapping_dict, stack_len)
}

#[allow(dead_code)]
/// returns string of the semantically same subformula, but with "canonized" var names
fn get_canonical(subform_string: String) -> String {
    let canonized_tuple = canonize_subform(
        subform_string.chars().peekable(),
        HashMap::new(),
        String::new(),
        0,
    );
    canonized_tuple.1
}

#[allow(dead_code)]
/// returns tuple with the canonized subformula string and mapping dictionary used for canonization
fn get_canonical_and_mapping(subform_string: String) -> (String, HashMap<String, String>) {
    let canonized_tuple = canonize_subform(
        subform_string.chars().peekable(),
        HashMap::new(),
        String::new(),
        0,
    );
    (canonized_tuple.1, canonized_tuple.2)
}

/*
/// find out if we have some duplicate subtrees in our syntax tree
/// marks duplicate nodes' string + the number of its appearances
/// uses some kind of canonization - EX{x} and EX{y} recognized as duplicates
pub fn mark_duplicates(root_node: &Node) -> HashMap<String, i32> {
    // go through the nodes from top, use height to compare only those with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    let mut heap_queue: BinaryHeap<&Node> = BinaryHeap::new();

    let mut last_height = root_node.height.clone();
    let mut same_height_node_strings: HashSet<String> = HashSet::new();
    heap_queue.push(root_node);

    // because we are traversing a tree, we dont care about cycles
    while let Some(node) = heap_queue.pop() {
        let mut skip = false;
        let canonical_subform = get_canonical(node.subform_str.clone());

        if last_height == node.height {
            // if we have saved some nodes of the same height, lets compare them
            for other_string in same_height_node_strings.clone() {
                // TODO: check this - if we dont compare node with itself
                if other_string == canonical_subform {
                    if duplicates.contains_key(&canonical_subform) {
                        duplicates.insert(canonical_subform.clone(),duplicates[&canonical_subform] + 1);
                        skip = true;
                    }
                    else {
                        duplicates.insert(canonical_subform.clone(),1);
                    }
                    break;
                }
            }

            // do not include subtree of the duplicate in the traversing (will be cached during eval)
            if skip { continue; }
            same_height_node_strings.insert(canonical_subform);
        }
        else {
            // else we got node from lower level, so we empty the set of nodes to compare
            last_height = node.height;
            same_height_node_strings.clear();
            same_height_node_strings.insert(get_canonical(node.subform_str.clone()));
        }

        // add children of node to the heap_queue
        match &node.node_type {
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

/// TEMPORARY VERSION FOR NOW, FIX CACHE AND >UPDATE< VERSION ABOVE in future (this one does not use canonical forms)
/// find out if we have some duplicate subtrees in our syntax tree
/// marks duplicate nodes' string + the number of its appearances
/// this version does not consider canonical forms
/// it also does not mark terminal node duplicates (not worth)
pub fn mark_duplicates(root_node: &Node) -> HashMap<String, i32> {
    // go through the nodes from top, use height to compare only those with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    let mut heap_queue: BinaryHeap<&Node> = BinaryHeap::new();

    let mut last_height = root_node.height.clone();
    let mut same_height_node_strings: HashSet<String> = HashSet::new();
    heap_queue.push(root_node);

    // because we are traversing a tree, we dont care about cycles
    while let Some(node) = heap_queue.pop() {
        // lets stop the process when we hit terminal nodes, not worth marking
        if node.height == 0 {
            break;
        }

        let mut skip = false;
        if last_height == node.height {
            // if we have saved some nodes of the same height, lets compare them
            for other_string in same_height_node_strings.clone() {
                if other_string == node.subform_str.as_str() {
                    if duplicates.contains_key(node.subform_str.as_str()) {
                        duplicates
                            .insert(node.subform_str.clone(), duplicates[&node.subform_str] + 1);
                    } else {
                        duplicates.insert(node.subform_str.clone(), 1);
                    }
                    skip = true; // skip the descendants of the duplicate node
                    break;
                }
            }

            // do not include subtree of the duplicate in the traversing (will be cached during eval)
            if skip {
                continue;
            }
            same_height_node_strings.insert(node.subform_str.clone());
        } else {
            // else we got node from lower level, so we empty the set of nodes to compare
            last_height = node.height;
            same_height_node_strings.clear();
            same_height_node_strings.insert(node.subform_str.clone());
        }

        // add children of node to the heap_queue
        match &node.node_type {
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
