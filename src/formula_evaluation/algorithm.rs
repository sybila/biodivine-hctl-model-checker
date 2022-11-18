use crate::aeon::scc_computation::compute_attractor_states;
use crate::formula_evaluation::eval_hctl_components::*;
use crate::formula_evaluation::mark_duplicate_subform::*;
use crate::formula_preprocessing::operation_enums::*;
use crate::formula_preprocessing::parser::{Node, NodeType};

use biodivine_lib_bdd::{Bdd, bdd};

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

use std::collections::HashMap;

pub struct EvalInfo {
    duplicates: HashMap<String, i32>,
    cache: HashMap<String, GraphColoredVertices>,
}

/// Preprocess and evaluate formula on the given transition graph
pub fn eval_minimized_tree(tree: Node, graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    // prepare the list of duplicates & cache
    let mut eval_info: EvalInfo = EvalInfo {
        duplicates: mark_duplicates(&tree),
        cache: HashMap::new(),
    };

    // compute states with self-loops which will be needed
    let steady_states = compute_steady_states(graph);
    println!("steady states computed");
    let graph_with_steady_states =
        SymbolicAsyncGraph::new_add_steady_states_to_existing(
            graph.clone(),
            steady_states,
        );

    eval_node(tree, &graph_with_steady_states, &mut eval_info)
}

/// Same as previous fn, but UNSAFE
/// It does not explicitly compute states with self-loops, but instead allows caller to provide them
/// which is useful 1) when computing several properties on the same model
/// or 2) if we want to ignore self-loops (optimisation that may be valid sometimes)
/// Do not use if you are not sure that it does not affect the result
pub fn eval_minimized_tree_unsafe_ex(
    tree: Node,
    graph: &SymbolicAsyncGraph,
    self_loop_states: GraphColoredVertices
) -> GraphColoredVertices {
    let mut eval_info: EvalInfo = EvalInfo {
        duplicates: mark_duplicates(&tree),
        cache: HashMap::new(),
    };
    let graph_with_steady_states =
        SymbolicAsyncGraph::new_add_steady_states_to_existing(
            graph.clone(),
            self_loop_states,
        );
    eval_node(tree, &graph_with_steady_states, &mut eval_info)
}

/// Recursively evaluates the formula sub-tree `node` on the given `graph`
/// Uses pre-computed set of `duplicate` sub-formulae to allow for caching
pub fn eval_node(
    node: Node,
    graph: &SymbolicAsyncGraph,
    eval_info: &mut EvalInfo,
) -> GraphColoredVertices {
    // first check whether this node does not belong in the duplicates
    let mut save_to_cache = false;
    if eval_info.duplicates.contains_key(node.subform_str.as_str()) {
        if eval_info.cache.contains_key(node.subform_str.as_str()) {
            // decrement number of duplicates left
            *eval_info.duplicates.get_mut(node.subform_str.as_str()).unwrap() -= 1;

            // we will get bdd, but sometimes we must rename its vars
            // because it might have differently named state variables before
            let result = eval_info
                .cache
                .get(node.subform_str.as_str())
                .unwrap()
                .clone();

            // if we already visited all of the duplicates, lets delete the cached value
            if eval_info.duplicates[node.subform_str.as_str()] == 0 {
                eval_info.duplicates.remove(node.subform_str.as_str());
                eval_info.cache.remove(node.subform_str.as_str());
            }
            // since we are working with canonical cache, we must rename vars in result bdd
            return result;
        } else {
            // if the cache does not contain result for this subformula, set insert flag
            save_to_cache = true;
        }
    }

    // first lets check for special cases, which can be optimised:
    // 1) attractors
    if is_attractor_pattern(node.clone()) {
        let result = compute_attractor_states(graph, graph.mk_unit_colored_vertices());
        if save_to_cache {
            eval_info.cache.insert(node.subform_str.clone(), result.clone());
        }
        return result;
    }
    // 2) fixed-points
    if is_fixed_point_pattern(node.clone()) {
        return graph.steady_states().unwrap();
    }

    let result = match node.node_type {
        NodeType::TerminalNode(atom) => match atom {
            Atomic::True => graph.mk_unit_colored_vertices(),
            Atomic::False => graph.mk_empty_vertices(),
            Atomic::Var(name) => eval_hctl_var(graph, name.as_str()),
            Atomic::Prop(name) => eval_prop(graph, &name),
        },
        NodeType::UnaryNode(op, child) => match op {
            UnaryOp::Not => eval_neg(graph, &eval_node(*child, graph, eval_info)),
            UnaryOp::Ex => eval_ex(graph, &eval_node(*child, graph, eval_info)),
            UnaryOp::Ax => eval_ax(graph, &eval_node(*child, graph, eval_info)),
            UnaryOp::Ef => eval_ef_saturated(graph, &eval_node(*child, graph, eval_info)),
            UnaryOp::Af => eval_af(graph, &eval_node(*child, graph, eval_info)),
            UnaryOp::Eg => eval_eg(graph, &eval_node(*child, graph, eval_info)),
            UnaryOp::Ag => eval_ag(graph, &eval_node(*child, graph, eval_info)),
        },
        NodeType::BinaryNode(op, left, right) => match op {
            BinaryOp::And => eval_node(*left, graph, eval_info)
                .intersect(&eval_node(*right, graph, eval_info)),
            BinaryOp::Or => eval_node(*left, graph, eval_info)
                .union(&eval_node(*right, graph, eval_info)),
            BinaryOp::Xor => eval_xor(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
            BinaryOp::Imp => eval_imp(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
            BinaryOp::Iff => eval_equiv(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
            BinaryOp::Eu => eval_eu_saturated(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
            BinaryOp::Au => eval_au(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
            BinaryOp::Ew => eval_ew(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
            BinaryOp::Aw => eval_aw(
                graph,
                &eval_node(*left, graph, eval_info),
                &eval_node(*right, graph, eval_info),
            ),
        },
        NodeType::HybridNode(op, var, child) => match op {
            HybridOp::Bind => eval_bind(graph, &eval_node(*child, graph, eval_info), var.as_str()),
            HybridOp::Jump => eval_jump(graph, &eval_node(*child, graph, eval_info), var.as_str()),
            HybridOp::Exists => eval_exists(graph, &eval_node(*child, graph, eval_info), var.as_str()),
            HybridOp::Forall => eval_forall(graph, &eval_node(*child, graph, eval_info), var.as_str()),
        },
    };

    // save result to cache if needed
    if save_to_cache {
        eval_info.cache.insert(node.subform_str.clone(), result.clone());
    }
    result
}

/// Checks whether node represents formula for attractors !{x}: AG EF {x}
/// This recognition step is used to later optimize the attractor pattern
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

/// Checks whether node represents formula for fixed-points !{x}: AX {x}
/// This recognition step is used to later optimize the fixed-point pattern
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

/// Computes steady states using "(V1 <=> f_V1) & ... & (Vn <=> f_Vn)" computation
/// Steady states are used for adding self-loops in the EX computation
/// Can also be used as optimised procedure for formula "!{x}: AX {x}"
pub fn compute_steady_states(graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    // TODO: make nicer
    let context = graph.symbolic_context();
    let network = graph.as_network();
    let update_functions: Vec<Bdd> = network
        .as_graph()
        .variables()
        .map(|variable| {
            let regulators = network.regulators(variable);
            let function_is_one = network
                .get_update_function(variable)
                .as_ref()
                .map(|fun| context.mk_fn_update_true(fun))
                .unwrap_or_else(|| context.mk_implicit_function_is_true(variable, &regulators));
            let variable_is_one = context.mk_state_variable_is_true(variable);
            bdd!(variable_is_one <=> function_is_one)
        })
        .collect();

    GraphColoredVertices::new(
        update_functions
            .iter()
            .fold(graph.mk_unit_colored_vertices().into_bdd(), |r, v| r.and(v)),
        context,
    )
}
