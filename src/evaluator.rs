use crate::parser::{Node, NodeType};
use crate::operation_enums::*;
use crate::implementation::*;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use crate::operation_enums::Atomic::False;


// TODO: more efficient negation of GraphColoredVertices using just bdds ?
// TODO: equivalence, nonequivalence, implication of GraphColoredVertices using bdds
// TODO: maybe even and / or directly on BDD?


pub fn eval_node(node: Node, graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    // TODO
    let empty_set = graph.mk_empty_vertices();

    return match node.node_type {
        NodeType::TerminalNode(atom) => match atom {
                Atomic::True => graph.mk_unit_colored_vertices(),
                Atomic::False => empty_set,
                Atomic::Var(name) => empty_set, // TODO
                Atomic::Prop(name) => labeled_by(graph, &name)
        }
        // TODO
        NodeType::UnaryNode(op, child) => match op {
            UnaryOp::Not => negate_set(graph, &eval_node(*child, graph)),
            UnaryOp::Ex => graph.pre(&eval_node(*child, graph)),
            UnaryOp::Ax => ax(graph, &eval_node(*child, graph)),
            UnaryOp::Ef => ef_saturated(graph, &eval_node(*child, graph)),
            UnaryOp::Af => af(graph, &eval_node(*child, graph)),
            UnaryOp::Eg => eg(graph, &eval_node(*child, graph)),
            UnaryOp::Ag => ag(graph, &eval_node(*child, graph)),
        }
        // TODO
        NodeType::BinaryNode(op, left, right) => match op {
            BinaryOp::And => eval_node(*left, graph).intersect(&eval_node(*right, graph)),
            BinaryOp::Or => eval_node(*left, graph).union(&eval_node(*right, graph)),
            BinaryOp::Xor => negate_set(
                graph,
                &equivalence(graph,&eval_node(*left, graph), &eval_node(*right, graph))),
            BinaryOp::Imp => implication(graph,&eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Iff => equivalence(graph,&eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Eu => eu(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Au => au(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Ew => ew(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
            BinaryOp::Aw => aw(graph, &eval_node(*left, graph), &eval_node(*right, graph)),
        },
        // TODO
        NodeType::HybridNode(op, var, child) => match op {
            HybridOp::Bind => empty_set,
            HybridOp::Jump => empty_set,
            HybridOp::Exist => empty_set,
        }
    };
}

fn get_canonical(subform_string: &str) -> String {
    return "".to_string(); // TODO
}

/// find out if we have some duplicate nodes in our parse tree
/// marks duplicate node's string together with number of its appearances
/// uses some kind of canonization - EX{x} and EX{y} recognized as duplicates
pub fn mark_duplicates(root_node: Node) -> HashMap<String, i32> {
    // go through the nodes from top, use height to compare only those with the same level
    // once we find duplicate, do not continue traversing its branch (it will be skipped during eval)
    let mut duplicates: HashMap<String, i32> = HashMap::new();
    let mut heap_queue: BinaryHeap<&Node> = BinaryHeap::new();

    let mut last_height = root_node.height.clone();
    let mut same_height_nodes: HashSet<&Node> = HashSet::new();
    heap_queue.push(&root_node);

    // because we are traversing a tree, we dont care about cycles
    while let Some(node) = heap_queue.pop() {
        let mut skip = False;
        let node_canonical_subform = get_canonical(&node.subform_str);

        if last_height == node.height {
            // TODO
            // if we have saved some nodes of the same height, lets compare them
            /*
            for n, n_canonic_subform in same_height_nodes:
                # TODO: dont we compare node with itself?
                if node_canonical_subform == n_canonic_subform:
                    # TODO : subform_string string problem?
                    if n_canonic_subform in duplicates:
                        duplicates[n_canonic_subform] += 1
                        skip = True  # we wont be traversing subtree of this node (cache will be used)
                    else:
                        duplicates[n_canonic_subform] = 1
                    break
            # do not include subtree of the duplicate in the traversing (will be cached during eval)
            if skip:
                continue
            same_height_nodes.add((node, get_canonical(node.subform_string)))

             */
        }
        else {
            // else we got node from lower level, so we empty the set of nodes to compare
            last_height = node.height;
            same_height_nodes.clear();
            same_height_nodes.insert(node);
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