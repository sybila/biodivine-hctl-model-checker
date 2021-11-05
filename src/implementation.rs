use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;

// TODO
/*
Model -> SymbolicAsyncGraph
BDD.Function -> GraphColoredVertices

labeled_by -> mk_state_variable_is_true

create_comparator
bind
jump
existential

pre_E_one_var -> graph.var_pre   (var_can_pre)
pre_E_all_vars -> graph.pre      (can_pre)
 */

/// Returns set where var given by name is true
/// If var is invalid, returns empty set
pub fn labeled_by(graph: &SymbolicAsyncGraph, name: &str) -> GraphColoredVertices {
    if let Some(var_id) = graph.as_network().as_graph().find_variable(name) {
        return GraphColoredVertices::new(
            graph.symbolic_context().mk_state_variable_is_true(var_id),
            graph.symbolic_context()
        );
    }
    graph.mk_empty_vertices()
}

/// EU computed using fixpoint
pub fn eu(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let false_bdd = graph.symbolic_context().mk_constant(false);
    let mut new_set = GraphColoredVertices::new(false_bdd, graph.symbolic_context());

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&graph.pre(&old_set)))
    }
    old_set
}

/// EF computed using fixpoint
pub fn ef(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let false_bdd = graph.symbolic_context().mk_constant(false);
    let mut new_set = GraphColoredVertices::new(false_bdd, graph.symbolic_context());

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&graph.pre(&old_set))
    }
    old_set
}

/// EF computed via saturation
pub fn ef_saturated(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut result = phi.clone();
    let mut done = false;
    while !done {
        done = true;
        for var in graph.as_network().variables().rev() {
            let update = graph.var_pre(var, &result).minus(&result);
            if !update.is_empty() {
                result = result.union(&update);
                done = false;
                break;
            }
        }
    }
    result
}

/// EG computed using fixpoint
pub fn eg(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let false_bdd = graph.symbolic_context().mk_constant(false);
    let mut new_set = GraphColoredVertices::new(false_bdd, graph.symbolic_context());

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.intersect(&graph.pre(&old_set))
    }
    old_set
}

/// AX computed through the EX
pub fn ax(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let true_bdd = graph.symbolic_context().mk_constant(true);
    let unit_set = GraphColoredVertices::new(true_bdd, graph.symbolic_context());
    unit_set.minus(&graph.pre(&unit_set.minus(&phi)))
}

/// AF computed through the EG
pub fn af(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let true_bdd = graph.symbolic_context().mk_constant(true);
    let unit_set = GraphColoredVertices::new(true_bdd, graph.symbolic_context());
    unit_set.minus(&eg(graph, &unit_set.minus(&phi)))
}

/// AG computed through the EF
pub fn ag(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let true_bdd = graph.symbolic_context().mk_constant(true);
    let unit_set = GraphColoredVertices::new(true_bdd, graph.symbolic_context());
    unit_set.minus(&ef_saturated(graph, &unit_set.minus(&phi)))
}

/// AU computed through the fixpoint
pub fn au(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let false_bdd = graph.symbolic_context().mk_constant(false);
    let mut new_set = GraphColoredVertices::new(false_bdd, graph.symbolic_context());

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&ax(graph, &old_set)))
    }
    old_set
}

/// EW computed through the AU
pub fn ew(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    let true_bdd = graph.symbolic_context().mk_constant(true);
    let unit_set = GraphColoredVertices::new(true_bdd, graph.symbolic_context());
    unit_set.minus(&au(graph, &unit_set.minus(&phi1), &unit_set.minus(&phi2)))
}

/// AW computed through the EU
pub fn aw(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    let true_bdd = graph.symbolic_context().mk_constant(true);
    let unit_set = GraphColoredVertices::new(true_bdd, graph.symbolic_context());
    unit_set.minus(&eu(graph, &unit_set.minus(&phi1), &unit_set.minus(&phi2)))
}