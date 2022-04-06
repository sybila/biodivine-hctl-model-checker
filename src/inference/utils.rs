use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

pub fn check_if_solution_contains_goal(
    graph: SymbolicAsyncGraph,
    goal_aeon_string: Option<String>,
    inferred_colors: GraphColors,
) {
    // if the goal network was supplied, lets check whether it is part of the solution set
    if let Some(goal_model) = goal_aeon_string {
        let goal_bn = BooleanNetwork::try_from(goal_model.as_str()).unwrap();
        match graph.mk_subnetwork_colors(&goal_bn) {
            Ok(goal_colors) => {
                // we will need intersection of goal colors with the ones from the result
                // if the goal is subset of result, it went well
                if goal_colors.intersect(&inferred_colors).approx_cardinality()
                    == goal_colors.approx_cardinality()
                {
                    println!("OK - color of goal network is included in resulting set.")
                } else {
                    println!("NOK - color of goal network is NOT included in resulting set.")
                }
            }
            Err(e) => println!("{}", e),
        }
    } else {
        println!("Goal network not provided.")
    }
}
