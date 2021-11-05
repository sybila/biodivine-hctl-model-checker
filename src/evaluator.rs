use crate::parser::{Node, NodeType};
use crate::operation_enums::*;
use crate::implementation::*;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;


// TODO: more efficient negation of GraphColoredVertices
// TODO: equivalence, nonequivalence, implication of GraphColoredVertices?


fn eval_node(node: Node, graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    // TODO
    let empty_set = graph.mk_empty_vertices();
    let unit_set = graph.mk_unit_colored_vertices();

    return match node.node_type {
        NodeType::TerminalNode(atom) => match atom {
                Atomic::True => unit_set,
                Atomic::False => empty_set,
                Atomic::Var(name) => empty_set, // TODO
                Atomic::Prop(name) => labeled_by(graph, &name)
        }
        // TODO
        NodeType::UnaryNode(op, child) => match op {
            UnaryOp::Not => unit_set.minus(&eval_node(*child, graph)),
            UnaryOp::Ex => graph.pre(&eval_node(*child, graph)),
            UnaryOp::Ax => ax(graph, &eval_node(*child, graph)),
            UnaryOp::Ef => ef_saturated(graph, &eval_node(*child, graph)),
            UnaryOp::Af => af(graph, &eval_node(*child, graph)),
            UnaryOp::Eg => eg(graph, &eval_node(*child, graph)),
            UnaryOp::Ag => ag(graph, &eval_node(*child, graph)),
        }
        // TODO
        NodeType::BinaryNode(op, left, right) => match op {
            BinaryOp::And => eval_node(*child, graph).intersect(&eval_node(*child, graph)),
            BinaryOp::Or => eval_node(*child, graph).union(&eval_node(*child, graph)),
            BinaryOp::Xor => empty_set,
            BinaryOp::Imp => empty_set,
            BinaryOp::Iff => empty_set,
            BinaryOp::Eu => empty_set,
            BinaryOp::Au => empty_set,
            BinaryOp::Ew => empty_set,
            BinaryOp::Aw => empty_set,
        },
        // TODO
        NodeType::HybridNode(op, var, child) => match op {
            HybridOp::Bind => empty_set,
            HybridOp::Jump => empty_set,
            HybridOp::Exist => empty_set,
        }
    };
}