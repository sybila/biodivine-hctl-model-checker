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

#[allow(dead_code)]
fn case_study_1(fully_parametrized: bool) {
    let aeon_string = if fully_parametrized {
        read_to_string("benchmark_models/inference/TLGL_reduced/TLGL_reduced_no_updates.aeon").unwrap()
    } else {
        read_to_string("benchmark_models/inference/TLGL_reduced/TLGL_reduced_partial_updates.aeon").unwrap()
    };

    let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    println!("Loaded model with {} vars.", bn.num_vars());
    let mut graph = SymbolicAsyncGraph::new(bn, 2).unwrap();

    // define the observations
    let diseased_attractor = "~Apoptosis_ & S1P & sFas & ~Fas & ~Ceramide_ & ~Caspase & MCL1 & ~BID_ & ~DISC_ & FLIP_ & ~IFNG_ & GPCR_";
    let healthy_attractor = "Apoptosis_ & ~S1P & ~sFas & ~Fas & ~Ceramide_ & ~Caspase & ~MCL1 & ~BID_ & ~DISC_ & ~FLIP_ & ~CTLA4_ & ~TCR & ~IFNG_ & ~CREB & ~P2 & ~SMAD_ & ~GPCR_ & ~IAP_";

    let mut inferred_colors = graph.mk_unit_colors();
    println!(
        "After applying static constraints, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

    let formulae: Vec<String> = vec![
        mk_steady_state_formula_specific(healthy_attractor.to_string()),
        mk_attractor_formula_nonspecific(diseased_attractor.to_string()),
    ];

    // first ensure attractor existence
    for formula in formulae {
        inferred_colors = model_check_formula_unsafe(formula, &graph).colors();
        graph = SymbolicAsyncGraph::new_restrict_colors_from_existing(graph, &inferred_colors);
        println!("attractor ensured")
    }
    println!(
        "After ensuring attractor presence, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

    // then prohibit all other attractors
    let attr_set = vec![healthy_attractor.to_string(), diseased_attractor.to_string()];
    let formula = mk_forbid_other_attractors_formula(attr_set);
    inferred_colors = model_check_formula_unsafe(formula, &graph).colors();
    println!(
        "{} suitable networks found in total",
        inferred_colors.approx_cardinality()
    );

    // println!("{}", graph.pick_witness(&inferred_colors).to_string());
}

#[allow(dead_code)]
fn case_study_2() {

}

#[allow(dead_code)]
fn case_study_3() {
    let aeon_string = read_to_string("benchmark_models/inference/CNS_development/model.aeon").unwrap();
    let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    println!("Loaded model with {} vars.", bn.num_vars());
    let mut graph = SymbolicAsyncGraph::new(bn, 1).unwrap();

    // define the observations
    /*
    "zero": {n: 0 for n in dom1}, # all nodes are 0
    "init": {n: 1 if n == "Pax6" else 0 for n in dom1}, # all nodes are 0 but Pax6
    "tM": {"Pax6": 1, "Tuj1": 0, "Scl": 0, "Aldh1L1": 0, "Olig2": 0, "Sox8": 0},
    "fT": {"Pax6": 1, "Tuj1": 1, "Brn2": 1, "Zic1": 1, "Aldh1L1": 0, "Sox8": 0},
    "tO": {"Pax6": 1, "Tuj1": 0 ,"Scl": 0, "Aldh1L1": 0, "Olig2": 1, "Sox8": 0},
    "fMS": {"Pax6": 1, "Tuj1": 0, "Zic1": 0, "Brn2": 0, "Aldh1L1": 0, "Sox8": 1},
    "tS": {"Pax6": 1, "Tuj1": 0, "Scl": 1, "Aldh1L1": 0, "Olig2": 0, "Sox8": 0},
    "fA": {"Pax6": 1, "Tuj1": 0, "Zic1": 0, "Brn2": 0, "Aldh1L1": 1, "Sox8": 0},
     */
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

    // constraints from the first part of the case study
    let formulae_v1: Vec<String> = vec![
        mk_steady_state_formula_nonspecific(f_a.to_string()),
        mk_steady_state_formula_nonspecific(f_ms.to_string()),

        mk_trap_space_formula(f_t.to_string()),

        mk_reachability_formula(init_state.to_string(), t_m.to_string(), false, false),
        mk_reachability_formula(init_state.to_string(), t_o.to_string(), false, false),
        mk_reachability_formula(init_state.to_string(), t_s.to_string(), false, false),
        mk_reachability_formula(t_m.to_string(), f_t.to_string(), false, false),
        mk_reachability_formula(t_o.to_string(), f_ms.to_string(), false, false),
        mk_reachability_formula(t_s.to_string(), f_a.to_string(), false, false),

        mk_reachability_formula(zero_state.to_string(), f_t.to_string(), false, true),
        mk_reachability_formula(zero_state.to_string(), f_ms.to_string(), false, true),
        mk_reachability_formula(zero_state.to_string(), f_a.to_string(), false, true),
    ];

    // constraints from the second part of the case study
    let universal_fps = vec![f_a.to_string(), f_ms.to_string(), f_t.to_string(), zero_state.to_string()];
    let formulae_v2: Vec<String> = vec![
        mk_forbid_other_steady_states_formula(universal_fps),
        // any fixed point reachable from "init" must be one of {f_a, f_ms, f_t}
        // if we use previous constraint, we can just prohibit reaching the zero fixed point
        format!("3{{x}}:@{{x}}:(({}) & ~EF(({}) & AX {}))", init_state, zero_state, zero_state),
    ];

    for formula in formulae_v1 {
        inferred_colors = model_check_formula_unsafe(formula, &graph).colors();
        graph = SymbolicAsyncGraph::new_restrict_colors_from_existing(graph, &inferred_colors);
        println!("constraint ensured")
    }
    println!(
        "After first set of constraints, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

    for formula in formulae_v2 {
        inferred_colors = model_check_formula_unsafe(formula, &graph).colors();
        graph = SymbolicAsyncGraph::new_restrict_colors_from_existing(graph, &inferred_colors);
        println!("constraint ensured")
    }
    println!(
        "After second set of constraints, {} concretizations remain.",
        inferred_colors.approx_cardinality(),
    );

}

fn main() {
    let start = SystemTime::now();
    case_study_1(false);
    //case_study_2();
    //case_study_3();
    println!("Elapsed time: {}ms", start.elapsed().unwrap().as_millis());
}
