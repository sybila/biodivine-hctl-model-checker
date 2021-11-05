use crate::parser::{Node, NodeType};
use crate::operation_enums::*;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;


fn eval_node(node: Node, graph: SymbolicAsyncGraph) -> GraphColoredVertices {
    // TODO
    match node.node_type {
        NodeType::TerminalNode(atom) => {
            match atom {
                Atomic::True => {}
                Atomic::False => {}
                Atomic::Var(name) => {}
                Atomic::Prop(name) => {}
            }
        }
        NodeType::UnaryNode(op, child) => {}
        NodeType::BinaryNode(op, left, right) => {}
        NodeType::HybridNode(op, var, child) => {}
    }


    // TODO - delete
    let false_bdd = graph.symbolic_context().mk_constant(false);
    GraphColoredVertices::new(false_bdd, graph.symbolic_context())
}