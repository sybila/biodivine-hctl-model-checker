use biodivine_hctl_model_checker::analysis::model_check_formula;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

use std::fs::read_to_string;

fn main() {
    let model_name = ".\\benchmark_models\\sketches-example\\model-example-sketches.aeon";
    let formula = "3{a}: (3{b}: (3{c}: (@{c}: ((EF {a}) & (EF {b}) & (@{a}: AG EF {a}) & (@{b}: (AG EF {b} & ~ EF {a})))))) & (3{x}:@{x}: ~v_1 & ~v_2 & v_3 & AG EF {x}) & (3{x}:@{x}: v_1 & v_2 & ~v_3 & AG EF {x})";

    let aeon_string = read_to_string(model_name).unwrap();
    let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
    let stg = SymbolicAsyncGraph::new(bn, 3).unwrap();

    let result = model_check_formula(formula.to_string(), &stg).unwrap();

    let res_color = result.colors();
    let witness_bn = stg.pick_witness(&res_color);
    println!("{}", witness_bn.to_string());
}
