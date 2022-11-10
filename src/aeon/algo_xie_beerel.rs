use crate::aeon::saturated_reachability::{reach_bwd, reachability_step};

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::VariableId;

/// Uses a simplified Xie-Beerel algorithm adapted to coloured setting to find all bottom
/// SCCs in the given `universe` set. It only tests transitions using `active_variables`.
/// All resulting components are collected into the `components` set.
pub fn xie_beerel_attractor_set(
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
    active_variables: &[VariableId],
    mut components: GraphColoredVertices,
) -> GraphColoredVertices {
    let mut universe = universe.clone();
    while !universe.is_empty() {
        let pivots = universe.pick_vertex();

        let pivot_basin = reach_bwd(graph, &pivots, &universe, active_variables);

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

            if done {
                break;
            }
        }

        if !pivot_component.is_empty() {
            components = components.union(&pivot_component);
        }

        universe = universe.minus(&pivot_basin);
    }
    components
}
