use std::fs::read_to_string;
use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate, VariableId};

/// Compute the network input variables.
fn network_inputs(network: &BooleanNetwork) -> Vec<VariableId> {
    network
        .variables()
        .filter(|v| network.regulators(*v).is_empty())
        .collect()
}


/// Create a copy of the given network with all input variables fixed to a constant.
fn fix_network_inputs(network: &BooleanNetwork, bool_values: Vec<bool>) -> BooleanNetwork {
    let mut result = network.clone();
    let mut i = 0;
    for v in network_inputs(network) {
        result
            .set_update_function(v, Some(FnUpdate::Const(bool_values[i])))
            .unwrap();
        i += 1;
    }
    result
}

/// Returns binary vector incremented by 1
fn next_bool_val(mut bool_vec: Vec<bool>) -> Result<Vec<bool>, String> {
    let mut i = 0;
    while i < bool_vec.len() {
        if bool_vec[i] {
            bool_vec[i] = false;
        }
        else {
            bool_vec[i] = true;
            return Ok(bool_vec);
        }
        i += 1;
    }
    return Err("finished".to_string());
}

fn main() {
    let aeon_string = read_to_string("experimental_model.aeon".to_string()).unwrap();
    let network = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();

    let input_num = network_inputs(&network).len();
    let mut input_values = Vec::with_capacity(input_num);
    input_values.resize(input_num, false);

    while let Ok(next_input_values) = next_bool_val(input_values.clone()) {
        input_values = next_input_values.clone();
        let fixed = fix_network_inputs(&network, input_values);
    }

    println!("{}", fixed.to_string())
}
