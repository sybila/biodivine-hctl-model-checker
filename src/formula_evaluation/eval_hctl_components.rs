use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

use crate::formula_evaluation::eval_utils::*;

/// Shortcut for negation which respects the allowed universe
pub fn eval_neg(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    unit_set.minus(set)
}

/// Evaluates the implication operation
pub fn eval_imp(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, left).union(right)
}

/// Evaluates the equivalence operation
pub fn eval_equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    left.intersect(right)
        .union(&eval_neg(graph, left).intersect(&eval_neg(graph, right)))
}

/// Evaluates the non-equivalence operation (xor)
pub fn eval_xor(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_equiv(graph, left, right))
}

/// Returns set where network var (proposition in HCTL formula) given by name is true
/// If var is invalid, panics
pub fn eval_prop(graph: &SymbolicAsyncGraph, name: &str) -> GraphColoredVertices {
    let network_variable = graph.as_network().as_graph().find_variable(name);
    if network_variable.is_none() {
        panic!("There is no network variable named {}.", name);
    }
    GraphColoredVertices::new(
        graph
            .symbolic_context()
            .mk_state_variable_is_true(network_variable.unwrap()),
        graph.symbolic_context(),
    )
}

pub fn eval_hctl_var(graph: &SymbolicAsyncGraph, hctl_var_name: &str) -> GraphColoredVertices {
    create_comparator(graph, hctl_var_name, None)
}

/// Evaluates binder operator - does intersection with comparator and projects out hctl var
pub fn eval_bind(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator(graph, var_name, None);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding the hctl var we want to get rid of
    project_out_hctl_var(graph, &intersection, var_name)
}

/// Evaluates existential operator - projects out given hctl var from bdd
pub fn eval_exists(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    project_out_hctl_var(graph, phi, var_name)
}

pub fn eval_forall(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    eval_neg(graph, &eval_exists(graph, &eval_neg(graph, &phi), var_name))
}

/// Evaluates jump operator - does intersection with comparator and projects out BN variables
pub fn eval_jump(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator(graph, var_name, None);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding variables from the Boolean network
    let result_bdd = intersection
        .into_bdd()
        .project(graph.symbolic_context().state_variables());
    // after projecting we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// Evaluates EX operator by computing predecessors, but adds self-loops to steady states
/// (EX phi) == PRE(phi) | (phi & steady_states)
pub fn eval_ex(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    graph
        .pre(&phi)
        .union(&phi.intersect(&graph.steady_states().unwrap()))
}

/*
/// Evaluates EU operator using fixpoint algorithm
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

/// Evaluates EF operator using fixpoint algorithm
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

/// Evaluates EU operator using algorithm with saturation
/// TODO: check generating predecessors (check if including self-loops is needed)
pub fn eval_eu_saturated(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
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

/// Evaluates EF operator via the algorithm for EU with saturation
/// This is possible because EF(phi) = EU(true,phi)
pub fn eval_ef_saturated(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    eval_eu_saturated(graph, &unit_set, phi)
}

/// Evaluates EG operator using fixpoint algorithm
pub fn eval_eg(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.intersect(&eval_ex(graph, &old_set));
    }
    old_set
}

/// Evaluates AX operator through the EX computation
pub fn eval_ax(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    eval_neg(graph, &eval_ex(graph, &eval_neg(graph, &phi)))
}

/// Evaluates AF operator using the EG computation
pub fn eval_af(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    eval_neg(graph, &eval_eg(graph, &eval_neg(graph, &phi)))
}

/// Evaluates AG operator using the EF computation
pub fn eval_ag(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    eval_neg(graph, &eval_ef_saturated(graph, &eval_neg(graph, &phi)))
}

/// Evaluates AU operator using the fixpoint algorithm
pub fn eval_au(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&eval_ax(graph, &old_set)));
    }
    old_set
}

/// Evaluates EW operator using the AU computation
pub fn eval_ew(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_au(graph, &eval_neg(graph, &phi1), &eval_neg(graph, &phi2)),
    )
}

/// Evaluates AW using the EU computation
pub fn eval_aw(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    eval_neg(
        graph,
        &eval_eu_saturated(graph, &eval_neg(graph, &phi1), &eval_neg(graph, &phi2)),
    )
}
