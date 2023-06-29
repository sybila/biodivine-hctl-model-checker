//! Contains the implementation of symbolic evaluation of HCTL operators for Boolean network models.

use crate::evaluation::low_level_operations::*;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Shortcut for negation which respects the allowed universe.
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

/// Return a set where the network var (proposition in HCTL formula) given by `name` is true.
/// Note that validity of formula's propositions are checked beforehand.
pub fn eval_prop(graph: &SymbolicAsyncGraph, name: &str) -> GraphColoredVertices {
    // propositions are checked during preproc, and must be valid network variables
    let network_variable = graph.as_network().as_graph().find_variable(name).unwrap();

    GraphColoredVertices::new(
        graph
            .symbolic_context()
            .mk_state_variable_is_true(network_variable),
        graph.symbolic_context(),
    )
}

/// Evaluate atomic sub-formula with only a HCTL variable.
pub fn eval_hctl_var(graph: &SymbolicAsyncGraph, hctl_var_name: &str) -> GraphColoredVertices {
    create_comparator_var_state(graph, hctl_var_name)
}

/// Evaluate binder operator - does intersection with comparator and projects out hctl var.
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

/// Evaluate existential quantifier - projects out given hctl var from bdd.
pub fn eval_exists(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    project_out_hctl_var(graph, phi, var_name)
}

/// Evaluate universal quantifier.
pub fn eval_forall(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_exists(graph, &eval_neg(graph, phi), var_name))
}

/// Evaluate jump operator - does intersection with comparator and projects out BN variables.
pub fn eval_jump(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator_var_state(graph, var_name);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding variables from the Boolean network
    project_out_state_vars(graph, intersection)
}

/// Evaluate EX operator by computing predecessors, but adds self-loops to steady states.
/// Computation is done like `EX phi == PRE(phi) | (phi & steady_states)`
pub fn eval_ex(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    graph.pre(phi).union(&phi.intersect(self_loop_states))
}

/*
/// Evaluate EU operator using fixpoint algorithm
/// deprecated version, use eval_eu_saturated
pub fn eu(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&ex(graph, &old_set)));
    }
    old_set
}

/// Evaluate EF operator using fixpoint algorithm
/// deprecated version, use eval_ef_saturated
pub fn ef(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&ex(graph, &old_set));
    }
    old_set
}
 */

/// Evaluate EU operator using algorithm with saturation.
pub fn eval_eu_saturated(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    // TODO: check generating predecessors (check if including self-loops really is not needed)
    let mut result = phi2.clone();
    let mut done = false;
    while !done {
        done = true;
        for var in graph.as_network().variables().rev() {
            let update = phi1.intersect(&graph.var_pre(var, &result)).minus(&result);
            if !update.is_empty() {
                result = result.union(&update);
                done = false;
                break;
            }
        }
    }
    result
}

/// Evaluate EF operator via the algorithm for EU with saturation.
/// This is possible because `EF(phi) = EU(true,phi)`.
pub fn eval_ef_saturated(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    eval_eu_saturated(graph, &unit_set, phi)
}

/// Evaluate EG operator using fixpoint algorithm.
pub fn eval_eg(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.intersect(&eval_ex(graph, &old_set, self_loop_states));
    }
    old_set
}

/// Evaluate AX operator through the EX computation.
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

/// Evaluate AF operator using the EG computation.
pub fn eval_af(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_eg(graph, &eval_neg(graph, phi), self_loop_states),
    )
}

/// Evaluate AG operator using the EF computation.
pub fn eval_ag(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    eval_neg(graph, &eval_ef_saturated(graph, &eval_neg(graph, phi)))
}

/// Evaluate AU operator using the fixpoint algorithm.
pub fn eval_au(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&eval_ax(graph, &old_set, self_loop_states)));
    }
    old_set
}

/// Evaluate EW operator using the AU computation.
pub fn eval_ew(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
    self_loop_states: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_au(
            graph,
            &eval_neg(graph, phi1),
            &eval_neg(graph, phi2),
            self_loop_states,
        ),
    )
}

/// Evaluate AW using the EU computation.
pub fn eval_aw(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_eu_saturated(graph, &eval_neg(graph, phi1), &eval_neg(graph, phi2)),
    )
}
