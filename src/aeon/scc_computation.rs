use crate::aeon::algo_xie_beerel::xie_beerel_attractor_set;
use crate::aeon::itgr::interleaved_transition_guided_reduction;

use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Computes the set of colored states contained in terminal SCCs
/// Initial universe can be used to e.g. restrict considered colors
/// Good default value would be graph.mk_unit_colored_vertices()
pub fn compute_attractor_states(
    graph: &SymbolicAsyncGraph,
    initial_universe: GraphColoredVertices,
) -> GraphColoredVertices {
    // First, perform ITGR reduction.
    let (universe, active_variables) =
        interleaved_transition_guided_reduction(graph, initial_universe);

    // Then run Xie-Beerel to actually collect all the components
    xie_beerel_attractor_set(
        graph,
        &universe,
        &active_variables,
        graph.mk_empty_vertices(),
    )
}

/*
#[allow(dead_code)]
/// Computes terminal SCCs and outputs the contained states to the given file
/// Is made separately from previous fn because of performance and different return types
pub fn compute_and_write_attractors_to_file(graph: &SymbolicAsyncGraph, file_name: &str) -> () {
    let task_context = GraphTaskContext::new();
    task_context.restart(graph);

    // First, perform ITGR reduction.
    let (universe, active_variables) = interleaved_transition_guided_reduction(
        &task_context,
        &graph,
        graph.mk_unit_colored_vertices(),
    );

    // open the file for outputs
    let output_file = File::create(file_name).unwrap();

    // Then run Xie-Beerel to actually detect the components, write the states to file
    xie_beerel_attractors(
        &task_context,
        &graph,
        &universe,
        &active_variables,
        |component| {
            write_states_to_file(&output_file, &component);
        },
    );
}
 */
