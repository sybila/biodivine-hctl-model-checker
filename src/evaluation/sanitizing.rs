//! Contains operations to sanitize bdds of their additional symbolic variables,
//! making them compatible with remaining biodivine libraries.

use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, SymbolicAsyncGraph, SymbolicContext,
};
use std::collections::HashMap;
use std::io::Write;

/// Turns a BDD that is a valid representation of coloured state set in the
/// `model_checking_context` into a BDD of the equivalent set in the `canonical_context`.
///
/// The method assumes that variables and parameters are ordered equivalently, they are just
/// augmented with extra model checking variables that are unused in the original BDD.
fn sanitize_bdd(
    model_checking_context: &SymbolicContext,
    canonical_context: &SymbolicContext,
    bdd: &Bdd,
) -> Bdd {
    // First, build a map that translates a "model checking" symbolic variable
    // into an equivalent "canonical" variable.
    let mut variable_map = HashMap::new();
    let mc_state_variables = model_checking_context.state_variables().iter();
    let mc_param_variables = model_checking_context.parameter_variables().iter();
    for mc_var in mc_state_variables.chain(mc_param_variables) {
        let var_name = model_checking_context.bdd_variable_set().name_of(*mc_var);
        let c_var = canonical_context
            .bdd_variable_set()
            .var_by_name(&var_name)
            .expect("Mismatch in model variables.");
        variable_map.insert(*mc_var, c_var);
    }

    // Then verify that the ordering of these variables is the same across both contexts.
    let mut mc_variables_sorted = Vec::from_iter(variable_map.keys().cloned());
    mc_variables_sorted.sort();
    let mut c_variables_sorted = Vec::from_iter(variable_map.values().cloned());
    c_variables_sorted.sort();
    for (mc, c) in mc_variables_sorted.iter().zip(c_variables_sorted.iter()) {
        assert_eq!(
            variable_map.get(mc),
            Some(c),
            "Mismatch in variable ordering."
        );
    }

    // Now we know it is actually safe to translate the BDD.

    // Now we have to do a very dumb thing to translate a BDD variable to its actual "raw index".
    // Sadly there isn't a nicer way to do this in lib-bdd right now.
    let mut variable_map = variable_map
        .into_iter()
        .map(|(a, b)| (format!("{a}"), format!("{b}")))
        .collect::<HashMap<_, _>>();
    // BDD terminal nodes contain information about the number of variables instead of a variable id.
    // We have to map this information too in the new BDD.
    variable_map.insert(
        format!("{}", model_checking_context.bdd_variable_set().num_vars()),
        format!("{}", canonical_context.bdd_variable_set().num_vars()),
    );

    let mc_string = bdd.to_string();
    let mut c_string: Vec<u8> = Vec::new();
    write!(c_string, "|").unwrap();

    for mc_node in mc_string.split('|') {
        if mc_node.is_empty() {
            // First and last item will be empty because there is an additional separator
            // at the beginning/end of the string.
            continue;
        }
        let mut node_data = mc_node.split(',');
        // Low/high links remain the same, but the variable ID is translated.
        let node_var = node_data.next().unwrap();
        let node_low = node_data.next().unwrap();
        let node_high = node_data.next().unwrap();
        let new_node_var = variable_map
            .get(node_var)
            .expect("Model checking BDD is using unexpected variables.");
        write!(c_string, "{new_node_var},{node_low},{node_high}|").unwrap();
    }

    Bdd::from_string(&String::from_utf8(c_string).unwrap())
}

/// Sanitize underlying BDD of a given coloured state set by removing its symbolic variables
/// that were used for representing HCTL state-variables.
///
/// The method assumes that variables and parameters are ordered equivalently, they are just
/// augmented with extra model checking variables that are unused in the original BDD.
pub fn sanitize_graph_colored_vertices(
    stg: &SymbolicAsyncGraph,
    colored_vertices: &GraphColoredVertices,
) -> GraphColoredVertices {
    let canonical_bn = stg.as_network();
    let canonical_context = SymbolicContext::new(canonical_bn).unwrap();
    let sanitized_result_bdd = sanitize_bdd(
        stg.symbolic_context(),
        &canonical_context,
        colored_vertices.as_bdd(),
    );
    GraphColoredVertices::new(sanitized_result_bdd, &canonical_context)
}
