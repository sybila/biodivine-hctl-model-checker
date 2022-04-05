#[allow(unused_imports)]
use hctl_model_checker::analysis::{analyse_formula, model_check_formula_unsafe, PrintOptions};

use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

/// Creates the formula describing the (non)existence of reachability between two states
/// `from_state` and `to_state` are both formulae describing particular states
/// `is_universal` is true iff we want all paths from `from_state` to reach `to_state`
/// `is_negative` is true iff we want to non-existence of path from `from_state` to `to_state`
#[allow(dead_code)]
fn create_state_reachability_formula(
    from_state: String,
    to_state: String,
    is_universal: bool,
    is_negative: bool,
) -> String {
    assert!(!(is_negative && is_universal));
    assert!(!to_state.is_empty() && !from_state.is_empty());
    if is_universal {
        return format!("(3{{x}}: (@{{x}}: {} & (AF ({}))))", from_state, to_state);
    }
    if is_negative {
        return format!("(3{{x}}: (@{{x}}: {} & (~EF ({}))))", from_state, to_state);
    }
    format!("(3{{x}}: (@{{x}}: {} & (EF ({}))))", from_state, to_state)
}

/// Creates the formula describing the existence of a particular trap space
/// trap space is a part of the state space from which we cannot escape
/// `trap_space` is a formula describing some proposition' values in a desired trap space
#[allow(dead_code)]
fn create_trap_space_formula(trap_space: String) -> String {
    assert!(!trap_space.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AG ({}))))", trap_space, trap_space)
}

/// Creates the formula describing the existence of specific attractor
/// `attractor_state` is a formula describing state in a desired attractor
#[allow(dead_code)]
fn create_attractor_formula(attractor_state: String) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AG EF ({}))))", attractor_state, attractor_state)
}

/// Creates the formula prohibiting all but the given attractors
/// `attractor_state_set` is a vector of formulae, each describing a state in particular
/// allowed attractor
#[allow(dead_code)]
fn create_attractor_prohibition_formula(attractor_state_set: Vec<String>) -> String {
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ~(AG EF (");
    for attractor_state in attractor_state_set {
        assert!(!attractor_state.is_empty());
        formula.push_str(format!("({}) | ", attractor_state).as_str())
    }
    formula.push_str("false ))))");
    formula
}

/// Creates the formula describing the existence of specific steady-state
/// `steady_state` is a formula describing particular desired fixed point
#[allow(dead_code)]
fn create_steady_state_formula(steady_state: String) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AX ({})))", steady_state, steady_state)
}

/// Creates the formula prohibiting all but the given steady-states
/// `steady_state_set` is a vector of formulae, each describing particular allowed fixed point
#[allow(dead_code)]
fn create_steady_state_prohibition_formula(steady_state_set: Vec<String>) -> String {
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ");
    for steady_state in steady_state_set {
        assert!(!steady_state.is_empty());
        formula.push_str(format!("~({}) & ", steady_state).as_str())
    }
    formula.push_str("(AX {x})))");
    formula
}

fn check_if_solution_contains_goal(
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

fn perform_inference(
    aeon_string: String,
) -> (GraphColors, SymbolicAsyncGraph) {
    let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    println!("Loaded model with {} vars.", bn.num_vars());

    // To be sure, create graph object with 2 HCTL vars
    let mut graph = SymbolicAsyncGraph::new(bn, 2).unwrap();

    let mut inferred_colors = graph.mk_unit_colors();
    println!(
        "After applying static constraints, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

    // TODO
    (inferred_colors, graph)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("3 arguments expected, got {}", args.len() - 1);
        println!("Usage: ./infer_networks model_file attractor_data forbid_extra_attrs");
        return;
    }
    if !(args[3].as_str() == "false" || args[3].as_str() == "true") {
        println!(
            "Invalid argument \"{}\", it must be either \"true\" or \"false\"",
            args[3]
        );
        println!("Usage: ./infer_networks model_file attractor_data (true | false)");
        return;
    }
    let forbid_extra_attrs = match args[3].as_str() {
        "false" => false,
        _ => true, // we need match to be exhaustive
    };

    // TODO: make this automatic from CLI
    //let goal_aeon_string = Some(read_to_string("inference_goal_model.aeon".to_string()).unwrap());
    let goal_aeon_string: Option<String> = None;

    let data_file = File::open(Path::new(args[2].as_str())).unwrap();
    let reader = BufReader::new(&data_file);
    let data: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let aeon_string = read_to_string(args[1].clone()).unwrap();

    let start = SystemTime::now();
    let (inferred_colors, graph) = perform_inference(aeon_string);
    check_if_solution_contains_goal(graph, goal_aeon_string, inferred_colors);
    println!("Elapsed time: {}ms", start.elapsed().unwrap().as_millis());
}
