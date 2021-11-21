use crate::parser::{Node, NodeType};
use crate::operation_enums::*;
use crate::implementation::*;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::iter::Peekable;
use std::str::Chars;


// TODO: equivalence, nonequivalence, implication of GraphColoredVertices using bdds


pub fn eval_node(
    node: Node,
    graph: &SymbolicAsyncGraph,
    duplicates: &mut HashMap<String, i32>
) -> GraphColoredVertices {
    let empty_set = graph.mk_empty_vertices();
    println!("{}", get_canonical(node.subform_str));

    return match node.node_type {
        NodeType::TerminalNode(atom) => match atom {
                Atomic::True => graph.mk_unit_colored_vertices(),
                Atomic::False => empty_set,
                // TODO - change this when we have HCTL vars included
                Atomic::Var(name) => empty_set,
                Atomic::Prop(name) => labeled_by(graph, &name)
        }
        NodeType::UnaryNode(op, child) => match op {
            UnaryOp::Not => negate_set(graph, &eval_node(*child, graph, duplicates)),
            UnaryOp::Ex => graph.pre(&eval_node(*child, graph, duplicates)),
            UnaryOp::Ax => ax(graph, &eval_node(*child, graph, duplicates)),
            UnaryOp::Ef => ef_saturated(graph, &eval_node(*child, graph, duplicates)),
            UnaryOp::Af => af(graph, &eval_node(*child, graph, duplicates)),
            UnaryOp::Eg => eg(graph, &eval_node(*child, graph, duplicates)),
            UnaryOp::Ag => ag(graph, &eval_node(*child, graph, duplicates)),
        }
        NodeType::BinaryNode(op, left, right) => match op {
            BinaryOp::And => eval_node(*left, graph, duplicates).intersect(&eval_node(*right, graph, duplicates)),
            BinaryOp::Or => eval_node(*left, graph, duplicates).union(&eval_node(*right, graph, duplicates)),
            BinaryOp::Xor => non_equiv(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
            BinaryOp::Imp => imp(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
            BinaryOp::Iff => equiv(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
            BinaryOp::Eu => eu(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
            BinaryOp::Au => au(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
            BinaryOp::Ew => ew(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
            BinaryOp::Aw => aw(graph, &eval_node(*left, graph, duplicates), &eval_node(*right, graph, duplicates)),
        },
        // TODO - change this when we have HCTL vars included
        NodeType::HybridNode(op, var, child) => match op {
            HybridOp::Bind => empty_set,
            HybridOp::Jump => empty_set,
            HybridOp::Exist => empty_set,
        }
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
                    if name_char == '}' { break; }
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
                    if name_char == '}' { break; }
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
                }
                else {
                    // This branch should never happen
                    println!("{}", format!("Canonical name was not found for {}", var_name));
                }
            }
            // all the other character, including boolean+temporal operators, '@', prop names
            _ => { canonical.push(ch); }
        }
        if should_return { break; }

    }
    (subform_chars, canonical, mapping_dict, stack_len)
}

/// returns semantically same subformula, but with "canonized" var names
fn get_canonical(subform_string: String) -> String {
    return canonize_subform(subform_string.chars().peekable(), HashMap::new(), String::new(), 0).1;
}

/// find out if we have some duplicate nodes in our parse tree
/// marks duplicate node's string together with number of its appearances
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