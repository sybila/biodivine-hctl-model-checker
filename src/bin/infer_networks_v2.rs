#[allow(unused_imports)]
use hctl_model_checker::analysis::{analyse_formula, model_check_formula_unsafe, PrintOptions};

use std::convert::TryFrom;
use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColors, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;

/// Creates the formula describing the (non)existence of reachability between two states (or partial)
/// `from_state` and `to_state` are both formulae describing particular states
/// `is_universal` is true iff we want all paths from `from_state` to reach `to_state`
/// `is_negative` is true iff we want to non-existence of path from `from_state` to `to_state`
#[allow(dead_code)]
fn create_reachability_formula(
    from_state: &str,
    to_state: &str,
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
fn create_trap_space_formula(trap_space: &str) -> String {
    assert!(!trap_space.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AG ({}))))", trap_space, trap_space)
}

/// Creates the formula describing the existence of specific attractor
/// `attractor_state` is a formula describing state in a desired attractor
#[allow(dead_code)]
fn create_attractor_formula(attractor_state: &str) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AG EF ({}))))", attractor_state, attractor_state)
}

/// Creates the formula prohibiting all but the given attractors
/// `attractor_state_set` is a vector of formulae, each describing a state in particular
/// allowed attractor
#[allow(dead_code)]
fn create_attractor_prohibition_formula(attractor_state_set: Vec<&str>) -> String {
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
fn create_steady_state_formula(steady_state: &str) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AX ({})))", steady_state, steady_state)
}

/// Creates the formula prohibiting all but the given steady-states
/// `steady_state_set` is a vector of formulae, each describing particular allowed fixed point
#[allow(dead_code)]
fn create_steady_state_prohibition_formula(steady_state_set: Vec<&str>) -> String {
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ");
    for steady_state in steady_state_set {
        assert!(!steady_state.is_empty());
        formula.push_str(format!("~({}) & ", steady_state).as_str())
    }
    formula.push_str("(AX {x})))");
    formula
}

#[allow(dead_code)]
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

fn case_study() {
    /*
    "tM": {"Pax6": 1, "Tuj1": 0, "Scl": 0, "Aldh1L1": 0, "Olig2": 0, "Sox8": 0},
    "fT": {"Pax6": 1, "Tuj1": 1, "Brn2": 1, "Zic1": 1, "Aldh1L1": 0, "Sox8": 0},
    "tO": {"Pax6": 1, "Tuj1": 0 ,"Scl": 0, "Aldh1L1": 0, "Olig2": 1, "Sox8": 0},
    "fMS": {"Pax6": 1, "Tuj1": 0, "Zic1": 0, "Brn2": 0, "Aldh1L1": 0, "Sox8": 1},
    "tS": {"Pax6": 1, "Tuj1": 0, "Scl": 1, "Aldh1L1": 0, "Olig2": 0, "Sox8": 0},
    "fA": {"Pax6": 1, "Tuj1": 0, "Zic1": 0, "Brn2": 0, "Aldh1L1": 1, "Sox8": 0},
     */
    let aeon_string = read_to_string("benchmark_models/inference/CNS_development/model.aeon").unwrap();
    let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    println!("Loaded model with {} vars.", bn.num_vars());
    let mut graph = SymbolicAsyncGraph::new(bn, 1).unwrap();

    // define the states
    let zero_state = "~Pax6 & ~Hes5 & ~Mash1 & ~Scl & ~Olig2 & ~Stat3 & ~Zic1 & ~Brn2 & ~Tuj1 & ~Myt1L & ~Sox8 & ~Aldh1L1";
    let init_state = "Pax6 & ~Hes5 & ~Mash1 & ~Scl & ~Olig2 & ~Stat3 & ~Zic1 & ~Brn2 & ~Tuj1 & ~Myt1L & ~Sox8 & ~Aldh1L1";
    let t_m = "Pax6 & ~Scl & ~Olig2 & ~Tuj1 & ~Sox8 & ~Aldh1L1";
    let f_t = "Pax6 & Zic1 & Brn2 & Tuj1 & ~Sox8 & ~Aldh1L1";
    let t_o = "Pax6 & ~Scl & Olig2 & ~Tuj1 & ~Sox8 & ~Aldh1L1";
    let f_ms = "Pax6 & ~Zic1 & ~Brn2 & ~Tuj1 & Sox8 & ~Aldh1L1";
    let t_s = "Pax6 & Scl & ~Olig2 & ~Tuj1 & ~Sox8 & ~Aldh1L1";
    let f_a = "Pax6 & ~Zic1 & ~Brn2 & ~Tuj1 & ~Sox8 & Aldh1L1";

    let mut inferred_colors = graph.mk_unit_colors();
    println!(
        "After applying static constraints, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

    let formulae: Vec<String> = vec![
        create_steady_state_formula(f_a),
        create_steady_state_formula(f_ms),
        create_trap_space_formula(f_t),
        create_reachability_formula(init_state, t_m, false, false),
        create_reachability_formula(init_state, t_o, false, false),
        create_reachability_formula(init_state, t_s, false, false),
        create_reachability_formula(t_m, f_t, false, false),
        create_reachability_formula(t_o, f_ms, false, false),
        create_reachability_formula(t_s, f_a, false, false),

    ];
}

fn main() {
    let start = SystemTime::now();
    case_study();
    println!("Elapsed time: {}ms", start.elapsed().unwrap().as_millis());
}
