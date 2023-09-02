//! Model checking utilities such as generating extended STG or checking STG for variable support.

use crate::preprocessing::node::{HctlTreeNode, NodeType};
use crate::preprocessing::operator_enums::HybridOp;

use biodivine_lib_param_bn::symbolic_async_graph::{SymbolicAsyncGraph, SymbolicContext};
use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::{HashMap, HashSet};

/// Create an extended symbolic transition graph that supports the number of needed HCTL variables.
pub fn get_extended_symbolic_graph(
    bn: &BooleanNetwork,
    num_hctl_vars: u16,
) -> Result<SymbolicAsyncGraph, String> {
    // for each BN var, `num_hctl_vars` new BDD vars must be created
    let mut map_num_vars = HashMap::new();
    for bn_var in bn.variables() {
        map_num_vars.insert(bn_var, num_hctl_vars);
    }
    let context = SymbolicContext::with_extra_state_variables(bn, &map_num_vars)?;
    let unit = context.mk_constant(true);

    SymbolicAsyncGraph::with_custom_context(bn.clone(), context, unit)
}

/// Compute the set of all uniquely named HCTL variables in the formula tree.
/// Variable names are collected from three quantifiers: bind, exists, forall (which is sufficient,
/// as the formula must not contain free variables).
pub fn collect_unique_hctl_vars(
    formula_tree: HctlTreeNode,
    mut seen_vars: HashSet<String>,
) -> HashSet<String> {
    match formula_tree.node_type {
        NodeType::TerminalNode(_) => {}
        NodeType::UnaryNode(_, child) => {
            seen_vars.extend(collect_unique_hctl_vars(*child, seen_vars.clone()));
        }
        NodeType::BinaryNode(_, left, right) => {
            seen_vars.extend(collect_unique_hctl_vars(*left, seen_vars.clone()));
            seen_vars.extend(collect_unique_hctl_vars(*right, seen_vars.clone()));
        }
        // collect variables from exist and binder nodes
        NodeType::HybridNode(op, var_name, child) => {
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    seen_vars.insert(var_name); // we do not care whether insert is successful
                }
                _ => {}
            }
            seen_vars.extend(collect_unique_hctl_vars(*child, seen_vars.clone()));
        }
    }
    seen_vars
}

/// Check that extended symbolic graph's BDD supports enough extra variables for the computation.
/// There must be `num_hctl_vars` extra symbolic BDD vars for each BN variable.
pub fn check_hctl_var_support(stg: &SymbolicAsyncGraph, num_hctl_vars: usize) -> bool {
    for bn_var in stg.as_network().variables() {
        if num_hctl_vars > stg.symbolic_context().extra_state_variables(bn_var).len() {
            return false;
        }
    }
    true
}
