use biodivine_aeon_server::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use biodivine_aeon_server::scc::algo_xie_beerel::xie_beerel_attractors;
use biodivine_aeon_server::scc::Classifier;
use biodivine_aeon_server::GraphTaskContext;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

use biodivine_aeon_server::scc::algo_saturated_reachability::{reach_bwd, reachability_step};
use biodivine_lib_param_bn::VariableId;

/// Uses a simplified Xie-Beerel algorithm adapted to coloured setting to find all bottom
/// SCCs in the given `universe` set. It only tests transitions using `active_variables`.
/// Resulting components are collected into `components`.
pub fn xie_beerel_attractors_collect(
    ctx: &GraphTaskContext,
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
    active_variables: &[VariableId],
    mut components: GraphColoredVertices
) -> GraphColoredVertices
{
    let mut universe = universe.clone();
    while !universe.is_empty() {
        // Check cancellation and update remaining progress
        if ctx.is_cancelled() {
            break;
        }
        ctx.update_remaining(&universe);

        let pivots = universe.pick_vertex();

        let pivot_basin = reach_bwd(ctx, graph, &pivots, &universe, active_variables);

        let mut pivot_component = pivots.clone();

        // Iteratively compute the pivot component. If some color leaves `pivot_basin`, it is
        // removed from `pivot_component`, as it does not have to be processed any more.
        //
        // At the end of the loop, `pivot_component` contains only colors for which the component
        // is an attractor (other colors will leave the `pivot_basin` at some point).
        loop {
            let done = reachability_step(
                &mut pivot_component,
                &universe,
                active_variables,
                |var, set| graph.var_post(var, set),
            );

            // This ensures `pivot_component` is still subset of `pivot_basin` even if we do not
            // enforce it explicitly in `reachability_step`, since anything that leaks out
            // is completely eliminated.
            let escaped_basin = pivot_component.minus(&pivot_basin);
            if !escaped_basin.is_empty() {
                pivot_component = pivot_component.minus_colors(&escaped_basin.colors());
            }

            if done || ctx.is_cancelled() {
                break;
            }
        }

        if !pivot_component.is_empty() && !ctx.is_cancelled() {
            components = components.union(&pivot_component);
        }

        universe = universe.minus(&pivot_basin);
    }
    components
}

pub fn compute_scc(graph: &SymbolicAsyncGraph) -> GraphColoredVertices {
    let task_context = GraphTaskContext::new();
    task_context.restart(graph);

    // Now we can actually start the computation...

    // First, perform ITGR reduction.
    let (universe, active_variables) = interleaved_transition_guided_reduction(
        &task_context,
        &graph,
        graph.mk_unit_colored_vertices(),
    );

    // Then run Xie-Beerel to actually collect all the components
    xie_beerel_attractors_collect(
        &task_context,
        &graph,
        &universe,
        &active_variables,
        graph.mk_empty_vertices()
    )
}