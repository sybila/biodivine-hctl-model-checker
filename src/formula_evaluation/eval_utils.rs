use biodivine_lib_bdd::BddVariable;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Creates comparator between either
/// 1) network vars and given HCTL var' components: "(s__1 <=> var__1) & (s__2 <=> var__2) ... "
/// 2) two given HCTL vars' components: "(varA__1 <=> varB__1) & (varA__2 <=> varB__2) ... "
pub fn create_comparator(
    graph: &SymbolicAsyncGraph,
    hctl_var_name: &str,
    other_hctl_var_name_opt: Option<&str>,
) -> GraphColoredVertices {
    // TODO: 1) merge both branches 2)use eval_expression_string() method ?
    let reg_graph = graph.as_network().as_graph();
    let mut comparator = graph.mk_unit_colored_vertices().as_bdd().clone();

    if let Some(other_hctl_var_name) = other_hctl_var_name_opt {
        // do comparator between the two HCTL variables

        for network_var_id in reg_graph.variables() {
            let network_var_name = reg_graph.get_variable_name(network_var_id);
            let hctl_var1_component_name = format!("{}__{}", hctl_var_name, network_var_name);
            let hctl_var2_component_name = format!("{}__{}", other_hctl_var_name, network_var_name);
            let bdd_hctl_var1_component = graph
                .symbolic_context()
                .bdd_variable_set()
                .mk_var_by_name(hctl_var1_component_name.as_str());
            let bdd_hctl_var2_component = graph
                .symbolic_context()
                .bdd_variable_set()
                .mk_var_by_name(hctl_var2_component_name.as_str());
            comparator = comparator.and(&bdd_hctl_var1_component.iff(&bdd_hctl_var2_component));
        }
    } else {
        // do comparator between network vars and a HCTL variable

        for network_var_id in reg_graph.variables() {
            let network_var_name = reg_graph.get_variable_name(network_var_id);
            let hctl_component_name = format!("{}__{}", hctl_var_name, network_var_name);
            let bdd_network_var = graph
                .symbolic_context()
                .bdd_variable_set()
                .mk_var_by_name(network_var_name);
            let bdd_hctl_component = graph
                .symbolic_context()
                .bdd_variable_set()
                .mk_var_by_name(hctl_component_name.as_str());
            comparator = comparator.and(&bdd_hctl_component.iff(&bdd_network_var));
        }
    }

    // do intersection with the unit bdd (static constraints) to be sure its valid
    GraphColoredVertices::new(comparator, graph.symbolic_context())
        .intersect(graph.unit_colored_vertices())
}

/// Projects out (existentially quantifies) the given HCTL variable
/// This is used to evaluate hybrid operators or for HCTL var renaming
pub fn project_out_hctl_var(
    graph: &SymbolicAsyncGraph,
    colored_state_set: &GraphColoredVertices,
    var_name: &str,
) -> GraphColoredVertices {
    let var_idx = var_name.len() - 1; // len of var codes its index
    let vars_total = graph.symbolic_context().num_hctl_var_sets() as usize;

    // collect all BDD vars that encode the HCTL var
    let bdd_vars_to_project: Vec<BddVariable> = graph
        .symbolic_context()
        .hctl_variables()
        .iter()
        .skip(var_idx)
        .step_by(vars_total)
        .copied()
        .collect();

    // project them out
    let result_bdd = colored_state_set.as_bdd().project(&bdd_vars_to_project);

    // after projection we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// Substitute (rename) HCTL variable by another (valid) HCTL variable.
/// BDD of the set must not depend on the HCTL to be substituted.
/// Can be used for more efficient caching
pub fn substitute_hctl_var(
    graph: &SymbolicAsyncGraph,
    colored_states: &GraphColoredVertices,
    hctl_var_before: &str,
    hctl_var_after: &str
) -> GraphColoredVertices {
    // TODO: check that BDD for `set` does not depend on hctl_var_after

    // set new HCTL var to the same values as the current one
    let comparator = create_comparator(graph, hctl_var_before, Some(hctl_var_after));
    let colored_states_new = colored_states.intersect(&comparator);

    // get rid of the old var
    project_out_hctl_var(graph, &colored_states_new, hctl_var_before)
}

