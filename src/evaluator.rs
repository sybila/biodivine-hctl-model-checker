use crate::parser::{Node, NodeType};
use crate::operation_enums::*;
use crate::implementation::*;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;


// TODO: equivalence, nonequivalence, implication of GraphColoredVertices using bdds


pub fn eval_node(
    node: Node,
    graph: &SymbolicAsyncGraph,
    duplicates: &mut HashMap<String, i32>
) -> GraphColoredVertices {
    let empty_set = graph.mk_empty_vertices();

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

fn get_canonical(subform_string: &str) -> String {
    // TODO - do this correctly
    return "".to_string();
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
        let canonical_subform = get_canonical(&node.subform_str);

        if last_height == node.height {
            // if we have saved some nodes of the same height, lets compare them
            for other_string in same_height_node_strings.clone() {
                // TODO: check if we dont compare node with itself
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
            same_height_node_strings.insert(get_canonical(&node.subform_str));
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