use biodivine_lib_bdd::{BddVariable};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// shortcut for negation which respects the allowed universe
pub fn negate_set(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    unit_set.minus(set)
}

/// evaluates the implication operation
pub fn imp(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    negate_set(graph, left).union(right)
}

/// evaluates the equivalence operation
pub fn equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    left.intersect(right)
        .union(&negate_set(graph, left).intersect(&negate_set(graph, right)))
}

/// evaluates the non-equivalence operation (xor)
pub fn non_equiv(
    graph: &SymbolicAsyncGraph,
    left: &GraphColoredVertices,
    right: &GraphColoredVertices,
) -> GraphColoredVertices {
    negate_set(graph, &equiv(graph, left, right))
}

/// Returns set where var given by name is true
/// If var is invalid, prints error and returns empty set
pub fn labeled_by(graph: &SymbolicAsyncGraph, name: &str) -> GraphColoredVertices {
    GraphColoredVertices::new(
        graph
            .symbolic_context()
            .mk_state_variable_is_true(graph.as_network().as_graph().find_variable(name).unwrap()),
        graph.symbolic_context(),
    )
}

/// creates comparator between variables from network and corresponding HCTL var' components
/// it will be a set representing expression "(s__1 <=> var__1) & (s__2 <=> var__2) ... "
pub fn create_comparator(graph: &SymbolicAsyncGraph, hctl_var_name: &str) -> GraphColoredVertices {
    // TODO: use eval_expression_string() method ?
    let reg_graph = graph.as_network().as_graph();
    let mut comparator = graph.mk_unit_colored_vertices().as_bdd().clone();

    for nw_var_id in reg_graph.variables() {
        let nw_var_name = reg_graph.get_variable_name(nw_var_id);
        let hctl_component_name = format!("{}__{}", hctl_var_name, nw_var_name);
        let bdd_nw_var = graph
            .symbolic_context()
            .bdd_variable_set()
            .mk_var_by_name(nw_var_name);
        let bdd_hctl_component = graph
            .symbolic_context()
            .bdd_variable_set()
            .mk_var_by_name(hctl_component_name.as_str());
        comparator = comparator.and(&bdd_hctl_component.iff(&bdd_nw_var));
    }

    // we must do the intersection with the unit bdd (static constraints)
    GraphColoredVertices::new(comparator, graph.symbolic_context())
        .intersect(graph.unit_colored_vertices())
}

/// evaluates bind operator - does intersection with comparator and projects out hctl var
pub fn bind(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator(graph, var_name);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding the hctl var we want to get rid of
    let var_idx = var_name.len() - 1; // len of var codes its index
    let vars_total = graph.symbolic_context().num_hctl_var_sets() as usize;
    let vars_to_project: Vec<BddVariable> = graph
        .symbolic_context()
        .hctl_variables()
        .iter()
        .skip(var_idx)
        .step_by(vars_total)
        .copied()
        .collect();
    let result_bdd = intersection.into_bdd().project(&vars_to_project);

    // after projecting we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// evaluates existential operator - projects out given hctl var from bdd
pub fn existential(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    // lets just project out the bdd vars coding the hctl var we want to get rid of
    let var_idx = var_name.len() - 1; // len of var codes its index
    let vars_total = graph.symbolic_context().num_hctl_var_sets() as usize;
    let vars_to_project: Vec<BddVariable> = graph
        .symbolic_context()
        .hctl_variables()
        .iter()
        .skip(var_idx)
        .step_by(vars_total)
        .copied()
        .collect();
    let result_bdd = phi.as_bdd().project(&vars_to_project);

    // after projecting we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// evaluates jump operator - does intersection with comparator and projects out BN variables
pub fn jump(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let comparator = create_comparator(graph, var_name);
    let intersection = comparator.intersect(phi);

    // now lets project out the bdd vars coding variables from boolean network
    let result_bdd = intersection
        .into_bdd()
        .project(graph.symbolic_context().state_variables());
    // after projecting we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// EX computed using pre, but with added self-loops
/// (EX phi) == PRE(phi) | (phi & fixed-points)
pub fn ex(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
) -> GraphColoredVertices {
    graph.pre(&phi).union(&phi.intersect(&graph.steady_states().unwrap()))
}

// TODO make these commented algorithms use EX function instead of pre function
/*
/// EU computed using fixpoint algorithm
/// deprecated version, use eu_saturated
pub fn eu(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    let mut old_set = phi2.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.union(&phi1.intersect(&graph.pre(&old_set)))
    }
    old_set
}

/// EF computed using fixpoint algorithm
/// deprecated version, use ef_saturated
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

// TODO: update generating predecessors (check including self-loops)
pub fn eu_saturated(
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

/// EF computed via the EU with saturation
/// This is possible because EF(phi) = EU(true,phi)
pub fn ef_saturated(
    graph: &SymbolicAsyncGraph,
    phi: &GraphColoredVertices,
) -> GraphColoredVertices {
    let unit_set = graph.mk_unit_colored_vertices();
    eu_saturated(graph, &unit_set, phi)
}

/// EG computed using fixpoint
pub fn eg(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    let mut old_set = phi.clone();
    let mut new_set = graph.mk_empty_vertices();

    while old_set != new_set {
        new_set = old_set.clone();
        old_set = old_set.intersect(&ex(graph, &old_set))
    }
    old_set
}

/// AX computed through the EX
pub fn ax(graph: &SymbolicAsyncGraph, phi: &GraphColoredVertices) -> GraphColoredVertices {
    negate_set(graph, &ex(graph, &negate_set(graph, &phi)))
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
pub fn au(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
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
pub fn ew(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    negate_set(
        graph,
        &au(graph, &negate_set(graph, &phi1), &negate_set(graph, &phi2)
        ),
    )
}

/// AW computed through the EU
pub fn aw(
    graph: &SymbolicAsyncGraph,
    phi1: &GraphColoredVertices,
    phi2: &GraphColoredVertices,
) -> GraphColoredVertices {
    negate_set(
        graph,
        &eu_saturated(graph, &negate_set(graph, &phi1), &negate_set(graph, &phi2)),
    )
}
