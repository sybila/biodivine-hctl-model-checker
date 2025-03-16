//! Contains the implementation of symbolic evaluation of HCTL operators for Boolean network models.

use crate::evaluation::low_level_operations::*;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Evaluate negation respecting the allowed universe.
pub fn eval_neg(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    unit_set.minus(set)
}

/// Evaluate the implication operation.
pub fn eval_imp(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, left).union(right)
}

/// Evaluate the equivalence operation.
pub fn eval_equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    left.intersect(right)
        .union(&eval_neg(graph, left).intersect(&eval_neg(graph, right)))
}

/// Evaluate the non-equivalence operation (xor).
pub fn eval_xor(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_equiv(graph, left, right))
}

/// Return a coloured set where a `proposition` in HCTL formula (a BN variable) is true.
/// Note that validity of formula's propositions must be checked beforehand.
pub fn eval_prop(graph: &SymbolicAsyncGraph, proposition: &str) -> GraphColoredVertices {
    // each proposition is checked during preprocessing, thus it must be a valid network variable
    let network_variable = graph
        .symbolic_context()
        .find_network_variable(proposition)
        .unwrap();

    GraphColoredVertices::new(
        graph
            .symbolic_context()
            .mk_state_variable_is_true(network_variable),
        graph.symbolic_context(),
    )
}

/// Evaluate atomic sub-formula containing only a HCTL variable.
pub fn eval_hctl_var(graph: &SymbolicAsyncGraph, hctl_var_name: &str) -> GraphColoredVertices {
    create_comparator_var_state(graph, hctl_var_name)
}

/// Evaluate binder operator.
/// It essentially does an intersection with "comparator" relation and projects out the HCTL var.
pub fn eval_bind(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator_var_state(graph, var_name);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding the hctl var we want to get rid of
    project_out_hctl_var(graph, &intersection, var_name)
}

/// Evaluate existential quantifier.
/// It essentially does an existential projection on the HCTL var.
pub fn eval_exists(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    project_out_hctl_var(graph, phi, var_name)
}

/// Evaluate jump operator.
/// It essentially does an intersection with "comparator" relation and projects out the BN variables.
pub fn eval_jump(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator_var_state(graph, var_name);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding variables from the Boolean network
    project_out_bn_vars(graph, &intersection)
}

/// Evaluate EX operator by computing predecessors, adding precomputed self-loop states.
/// Computation is done in a following way: `EX phi == PRE(phi) | (phi & steady_states)`
pub fn eval_ex(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    graph.pre(phi).union(&phi.intersect(self_loop_states))
}

#[allow(dead_code)]
/// Evaluate EU operator using the classical fixpoint algorithm.
/// Currently, this is not the most efficient version, use `eval_eu_saturated` instead.
pub fn eval_eu(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_colored_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&eval_ex(graph, &old_set, self_loop_states)));
    }
    old_set
}

#[allow(dead_code)]
/// Evaluate EF operator using the classical fixpoint algorithm.
/// Currently, this is not the most efficient version, use `eval_ef_saturated` instead.
pub fn eval_ef(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_colored_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&eval_ex(graph, &old_set, self_loop_states));
    }
    old_set
}

/// Evaluate EU operator using the saturation-based algorithm.
pub fn eval_eu_saturated<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    // TODO: for generating predecessors, check if including self-loops really is not needed
    let mut result = phi2.clone();
    let mut done = false;
    while !done {
        done = true;
        for var in graph.variables().rev() {
            let update = phi1.intersect(&graph.var_pre(var, &result)).minus(&result);
            if !update.is_empty() {
                result = result.union(&update);
                done = false;
                break;
            }
        }
        progress_callback(&result, "Computing LFP using saturation.");
    }
    result
}

/// Evaluate EF operator via the saturation-based algorithm for EU evaluation.
/// This is possible because `EF(phi) == EU(true, phi)`.
pub fn eval_ef_saturated<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    eval_eu_saturated(graph, &unit_set, phi, progress_callback)
}

/// Evaluate EG operator using the classical fixpoint algorithm.
pub fn eval_eg<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_colored_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.intersect(&eval_ex(graph, &old_set, self_loop_states));
        progress_callback(&old_set, "Computing GFP.");
    }
    old_set
}

/// Evaluate the AX operator through the EX computation.
/// This is possible because `AX(phi) == not EX(not phi)`.
pub fn eval_ax(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_ex(graph, &eval_neg(graph, phi), self_loop_states),
    )
}

/// Evaluate the AF operator using the EG computation.
/// This is possible because `AF(phi) == not EG(not phi)`.
pub fn eval_af<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_eg(
            graph,
            &eval_neg(graph, phi),
            self_loop_states,
            progress_callback,
        ),
    )
}

/// Evaluate the AG operator using the EF computation.
/// This is possible because `AG(phi) == not EF(not phi)`.
pub fn eval_ag<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_ef_saturated(graph, &eval_neg(graph, phi), progress_callback),
    )
}

/// Evaluate AU operator using the classical fixpoint algorithm.
pub fn eval_au<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_colored_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&eval_ax(graph, &old_set, self_loop_states)));
        progress_callback(&old_set, "Computing AU fixed-point.");
    }
    old_set
}

/// Evaluate the EW operator using the AU computation.
pub fn eval_ew<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_au(
            graph,
            &eval_neg(graph, phi1),
            &eval_neg(graph, phi2),
            self_loop_states,
            progress_callback,
        ),
    )
}

/// Evaluate the AW operator using the EU computation.
pub fn eval_aw<F: Fn(&GraphColoredVertices, &str)>(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    progress_callback: &F,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_eu_saturated(
            graph,
            &eval_neg(graph, phi1),
            &eval_neg(graph, phi2),
            progress_callback,
        ),
    )
}
