/*
use biodivine_aeon_server::scc::algo_interleaved_transition_guided_reduction::interleaved_transition_guided_reduction;
use biodivine_aeon_server::scc::algo_xie_beerel::xie_beerel_attractors;
use biodivine_aeon_server::scc::Classifier;
use biodivine_aeon_server::GraphTaskContext;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

pub fn compute_scc(graph: &SymbolicAsyncGraph) {
    let classifier = Classifier::new(graph);
    let task_context = GraphTaskContext::new();
    task_context.restart(graph);

    // Now we can actually start the computation...

    // First, perform ITGR reduction.
    let (universe, active_variables) = interleaved_transition_guided_reduction(
        &task_context,
        &graph,
        graph.mk_unit_colored_vertices(),
    );

    // Then run Xie-Beerel to actually detect the components.
    xie_beerel_attractors(
        &task_context,
        &graph,
        &universe,
        &active_variables,
        |component| {
            println!("Found attractor... {}", component.approx_cardinality());
            println!("Remaining: {}", task_context.get_percent_string());
            println!(
                "Unique states: {}",
                component.vertices().approx_cardinality()
            );
            println!("Unique colors: {}", component.colors().approx_cardinality());
            classifier.add_component(component, &graph);
        },
    );

    classifier.print();
}
*/