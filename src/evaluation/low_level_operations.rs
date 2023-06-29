//! Contains the low-level operations needed to implement HCTL operators symbolically.

use biodivine_lib_bdd::BddVariable;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Create a comparator between the components of either:
///  - system states and a given HCTL variable: `(s__1 <=> var__1) & (s__2 <=> var__2) ... `
///  - two given HCTL variables: `(varA__1 <=> varB__1) & (varA__2 <=> varB__2) ... `
/// The variant is specified by the argument `other_hctl_var_name_opt` (`None` -> v1, `Some` -> v2).
fn create_comparator(
    graph: &SymbolicAsyncGraph,
    hctl_var_name: &str,
    other_hctl_var_name_opt: Option<&str>,
) -> GraphColoredVertices {
    // TODO: merge both branches to not repeat code
    let reg_graph = graph.as_network().as_graph();
    let mut comparator = graph.mk_unit_colored_vertices().as_bdd().clone();

    // HCTL variables are named x, xx, xxx, ...
    let hctl_var_id = hctl_var_name.len() - 1; // len of var codes its index

    if let Some(other_hctl_var_name) = other_hctl_var_name_opt {
        // do comparator between the two HCTL variables

        // HCTL variables are named x, xx, xxx, ...
        let other_hctl_var_id = other_hctl_var_name.len() - 1; // len of var codes its index

        for network_var_id in reg_graph.variables() {
            let network_var_name = reg_graph.get_variable_name(network_var_id);

            // extra BDD vars are called "{network_variable}_extra_{i}"
            let hctl_var1_component_name = format!("{network_var_name}_extra_{hctl_var_id}");
            let hctl_var2_component_name = format!("{network_var_name}_extra_{other_hctl_var_id}");

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
            let hctl_component_name = format!("{network_var_name}_extra_{hctl_var_id}");
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

/// Wrapper for creating a comparator between the components of the state (network vars) and
/// a given HCTL variable.
/// It is computed as `(s__1 <=> var__1) & (s__2 <=> var__2) ... `
pub fn create_comparator_var_state(
    graph: &SymbolicAsyncGraph,
    hctl_var_name: &str,
) -> GraphColoredVertices {
    create_comparator(graph, hctl_var_name, None)
}

/// Wrapper for creating a comparator between the components of two HCTL variables.
/// It is computed as `(varA__1 <=> varB__1) & (varA__2 <=> varB__2) ...`
pub fn create_comparator_two_vars(
    graph: &SymbolicAsyncGraph,
    hctl_var_name: &str,
    other_hctl_var_name: &str,
) -> GraphColoredVertices {
    create_comparator(graph, hctl_var_name, Some(other_hctl_var_name))
}

/// Project out (existentially quantify) the given HCTL variable.
/// This is used during hybrid operators evaluation or during renaming of HCTL vars.
pub fn project_out_hctl_var(
    graph: &SymbolicAsyncGraph,
    colored_state_set: &GraphColoredVertices,
    hctl_var_name: &str,
) -> GraphColoredVertices {
    let hctl_var_id = hctl_var_name.len() - 1; // len of var codes its index

    // collect all BDD vars that encode the HCTL var
    let mut bdd_vars_to_project: Vec<BddVariable> = Vec::new();
    for network_var in graph.as_network().as_graph().variables() {
        let extra_vars = graph.symbolic_context().extra_state_variables(network_var);
        bdd_vars_to_project.push(*extra_vars.get(hctl_var_id).unwrap());
    }

    // project these bdd vars out
    let result_bdd = colored_state_set.as_bdd().exists(&bdd_vars_to_project);

    // after projection we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// Project out (existentially quantify) the BDD variables encoding the state.
/// This is used during evaluation of jump operator.
pub fn project_out_state_vars(
    graph: &SymbolicAsyncGraph,
    colored_state_set: GraphColoredVertices,
) -> GraphColoredVertices {
    // project out the bdd vars coding variables from the Boolean network
    let result_bdd = colored_state_set
        .into_bdd()
        .exists(graph.symbolic_context().state_variables());
    // after projecting we do not need to intersect with unit bdd
    GraphColoredVertices::new(result_bdd, graph.symbolic_context())
}

/// Substitute (rename) HCTL variable by another (valid) HCTL variable.
/// BDD of the set must not depend on the HCTL to be substituted.
/// Can be used for more efficient caching between sub-formulae with differently named vars.
pub fn substitute_hctl_var(
    graph: &SymbolicAsyncGraph,
    colored_states: &GraphColoredVertices,
    hctl_var_before: &str,
    hctl_var_after: &str,
) -> GraphColoredVertices {
    // if both vars are identical, dont do anything
    if hctl_var_before == hctl_var_after {
        return colored_states.clone();
    }
    // TODO: check that BDD for `set` does not depend on hctl_var_after

    // set new HCTL var to the same values as the current one
    let comparator = create_comparator_two_vars(graph, hctl_var_before, hctl_var_after);
    let colored_states_new = colored_states.intersect(&comparator);

    // get rid of the old var
    project_out_hctl_var(graph, &colored_states_new, hctl_var_before)
}
