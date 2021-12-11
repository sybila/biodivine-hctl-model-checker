use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;

// TODO
/*
create_comparator
bind
jump
existential
 */

pub fn negate_set(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    unit_set.minus(set)
}

pub fn imp(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right : &GraphColoredVertices
) -> GraphColoredVertices {
    negate_set(graph,left).union(right)
}

pub fn equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right : &GraphColoredVertices
) -> GraphColoredVertices {
    left.intersect(right).union(
        &negate_set(graph, left).intersect(&negate_set(graph, right))
    )
}

pub fn non_equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right : &GraphColoredVertices
) -> GraphColoredVertices {
    negate_set(graph, &equiv(graph, left, right))
}

/// Returns set where var given by name is true
/// If var is invalid, prints error and returns empty set
pub fn labeled_by(graph: &SymbolicAsyncGraph, name: &str) -> GraphColoredVertices {
    if let Some(var_id) = graph.as_network().as_graph().find_variable(name) {
        return GraphColoredVertices::new(
            graph.symbolic_context().mk_state_variable_is_true(var_id),
            graph.symbolic_context()
        );
    }
    println!("Wrong proposition \"{}\"", name);
    graph.mk_empty_vertices()
}

/// creates comparator between variables from network and corresponding HCTL var' components
/// it will be a set representing expression "(s__1 <=> var__1) & (s__2 <=> var__2) ... "
pub fn create_comparator(graph: &SymbolicAsyncGraph, hctl_var_name: &str) -> GraphColoredVertices {
    let reg_graph = graph.as_network().as_graph();
    let mut comparator = graph.mk_unit_colored_vertices().as_bdd().clone();

    for nw_var_id in reg_graph.variables() {
        let nw_var_name = reg_graph.get_variable_name(nw_var_id);
        let hctl_component_name = format!("{}__{}", hctl_var_name, nw_var_name);
        let mut bdd_nw_var = graph.symbolic_context()
            .bdd_variable_set()
            .mk_var_by_name(nw_var_name);
        let mut bdd_hctl_component = graph.symbolic_context()
            .bdd_variable_set()
            .mk_var_by_name(hctl_component_name.as_str());
        comparator = comparator.and(&bdd_hctl_component.iff(&bdd_nw_var));
    }

    GraphColoredVertices::new(comparator, graph.symbolic_context())
}

/*
def create_comparator(model: Model, var: str) -> Function:
    expr_parts = [f"(s__{i} <=> {var}__{i})" for i in range(model.num_props)]
    expr = " & ".join(expr_parts)
    comparator = model.bdd.add_expr(expr)
    return comparator
 */


/// EU computed using fixpoint
pub fn eu(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&graph.pre(&old_set)))
    }
    old_set
}

/*
/// EF computed using fixpoint
pub fn ef(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&graph.pre(&old_set))
    }
    old_set
}
 */

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
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.intersect(&graph.pre(&old_set))
    }
    old_set
}

/// AX computed through the EX
pub fn ax(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    negate_set(graph, &graph.pre(&negate_set(graph, &phi)))
}

/// AF computed through the EG
pub fn af(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    negate_set(graph, &eg(graph, &negate_set(graph, &phi)))
}

/// AG computed through the EF
pub fn ag(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    negate_set(graph, &ef_saturated(graph, &negate_set(graph, &phi)))
}

/// AU computed through the fixpoint
pub fn au(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_vertices();

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
    negate_set(graph, &au(graph, &negate_set(graph, &phi1), &negate_set(graph, &phi2)))
}

/// AW computed through the EU
pub fn aw(graph: &SymbolicAsyncGraph,
      phi1: &GraphColoredVertices,
      phi2: &GraphColoredVertices
) -> GraphColoredVertices {
    negate_set(graph, &eu(graph, &negate_set(graph, &phi1), &negate_set(graph, &phi2)))
}