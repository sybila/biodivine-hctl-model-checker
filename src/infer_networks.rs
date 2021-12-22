use biodivine_lib_param_bn::biodivine_std::bitvector::ArrayBitVector;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::VariableId;

use biodivine_aeon_server::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use biodivine_aeon_server::GraphTaskContext;
use biodivine_aeon_server::scc::algo_saturated_reachability::{reach_bwd, reachability_step};

// use std::fs::{File, read_to_string};

use crate::evaluator::eval_tree;
use crate::parser::parse_hctl_formula;
use crate::tokenizer::tokenize_recursive;


/*
pub fn parse_for_infer(file_name: String) -> (SymbolicAsyncGraph, Vec<GraphVertices>) {

}
 */

pub fn parse_and_infer(graph: &SymbolicAsyncGraph, attractor_state_formulas: Vec<String>) {
    let mut measured_attractor_states: Vec<GraphColoredVertices> = Vec::new();

    // first parse the attractor state formulas to colored vertex sets
    for state_formula in attractor_state_formulas {
        let tokens = match tokenize_recursive(&mut state_formula.chars().peekable(), true) {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e);
                Vec::new()
            }
        };
        match parse_hctl_formula(&tokens) {
            Ok(tree) => {
                measured_attractor_states.push(eval_tree(tree, &graph));
            },
            Err(message) => println!("{}", message),
        }
    }

    // and do the inference
    infer_nw(graph, measured_attractor_states);
}

pub fn infer_nw(
    graph: &SymbolicAsyncGraph,
    measured_attractor_states: Vec<GraphColoredVertices>
) -> () {
    let task_context = GraphTaskContext::new();
    task_context.restart(graph);

    // prepare sets of colors for every "measured attractor state"
    let mut colors_per_attractor_state: Vec<GraphColors> = Vec::new();
    let mut i = 0;
    while i < measured_attractor_states.len() {
        colors_per_attractor_state.push(graph.mk_empty_colors());
        i += 1;
    }

    // First, perform ITGR reduction.
    let (universe, active_variables) = interleaved_transition_guided_reduction(
        &task_context,
        &graph,
        graph.mk_unit_colored_vertices(),
    );

    // Then run Xie-Beerel to actually collect all the components
    xie_beerel_attractors_infer_gradually(
        &task_context,
        &graph,
        &universe,
        &active_variables,
        measured_attractor_states,
        colors_per_attractor_state
    );
}

pub fn xie_beerel_attractors_infer_gradually(
    ctx: &GraphTaskContext,
    graph: &SymbolicAsyncGraph,
    universe: &GraphColoredVertices,
    active_variables: &[VariableId],
    measured_attractor_states: Vec<GraphColoredVertices>,
    mut colors_per_attractor_state: Vec<GraphColors>
) -> ()
{
    let mut counter : f64 = 0.;
    let mut already_found_colors = graph.mk_empty_colors();
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

        // Check if any of the measured states is contained in the component
        // If so, save the colors for which it is present
        // Check if all measured states were found for some color, and if so, extract the
        // color & print its corresponding BN
        if !pivot_component.is_empty() && !ctx.is_cancelled() {
            let mut i = 0;
            while i < measured_attractor_states.len() {
                // check whether a measured state was found in component and if so, add the colors
                let intersection = pivot_component.intersect(&measured_attractor_states[i]);
                if !intersection.is_empty() {
                    colors_per_attractor_state[i] = colors_per_attractor_state[i].union(&intersection.colors());
                }
                i += 1;
            }

            // do the intersection through all color sets, to see if there are shared colors
            let mut color_intersection = graph.mk_unit_colors();
            let mut i = 0;
            while i < colors_per_attractor_state.len() {
                color_intersection = color_intersection.intersect(&colors_per_attractor_state[i]);
                i += 1;
            }

            // subtract already counted colors
            color_intersection = color_intersection.minus(&already_found_colors);

            if !color_intersection.is_empty() {
                // if we have some new shared color, we found BN satisfying the network
                // we will utilize it and mark as found for the future
                counter += color_intersection.approx_cardinality();
                println!("FOUND {} NETWORKS", color_intersection.approx_cardinality());
                already_found_colors = already_found_colors.union(&color_intersection);
            }
        }
        universe = universe.minus(&pivot_basin);
    }
    println!("colors: {}", counter);
}