use biodivine_lib_param_bn::{BooleanNetwork, FnUpdate, VariableId};

/// Compute the network input variables.
fn network_inputs(network: &BooleanNetwork) -> Vec<VariableId> {
    network
        .variables()
        .filter(|v| network.regulators(*v).is_empty())
        .collect()
}


/// Create a copy of the given network with all input variables fixed to a constant.
fn fix_network_inputs(network: &BooleanNetwork, value: bool) -> BooleanNetwork {
    let mut result = network.clone();
    for v in network_inputs(network) {
        result
            .set_update_function(v, Some(FnUpdate::Const(value)))
            .unwrap();
    }
    result
}

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
    let aeon_string = "experimental_model.aeon".to_string();
    let network = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();

    let input_num = network_inputs(&network).len();
    let mut input_values = Vec::with_capacity(input_num);
    input_values.resize(input_num, false);

    let fixed = fix_network_inputs(&network, true);
    println!("{}", fixed.to_string())
}
