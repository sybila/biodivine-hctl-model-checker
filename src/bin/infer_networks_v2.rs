#[allow(unused_imports)]
use hctl_model_checker::analysis::{analyse_formula, model_check_formula_unsafe};
use hctl_model_checker::inference::inference_formulae::*;
#[allow(unused_imports)]
use hctl_model_checker::inference::utils::*;

use std::convert::TryFrom;
use std::fs::read_to_string;
use std::time::SystemTime;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

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
        create_steady_state_formula(f_a.to_string()),
        create_steady_state_formula(f_ms.to_string()),
        create_trap_space_formula(f_t.to_string()),
        create_reachability_formula(init_state.to_string(), t_m.to_string(), false, false),
        create_reachability_formula(init_state.to_string(), t_o.to_string(), false, false),
        create_reachability_formula(init_state.to_string(), t_s.to_string(), false, false),
        create_reachability_formula(t_m.to_string(), f_t.to_string(), false, false),
        create_reachability_formula(t_o.to_string(), f_ms.to_string(), false, false),
        create_reachability_formula(t_s.to_string(), f_a.to_string(), false, false),
    ];

    for formula in formulae {
        inferred_colors = model_check_formula_unsafe(formula, &graph).colors();
        graph = SymbolicAsyncGraph::new_restrict_colors_from_existing(graph, &inferred_colors);
        println!("constraint ensured")
    }
    println!(
        "After all constraints, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

}

fn main() {
    let start = SystemTime::now();
    case_study();
    println!("Elapsed time: {}ms", start.elapsed().unwrap().as_millis());
}
