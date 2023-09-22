//! Model checking utilities such as generating extended STG or checking STG for variable support.

use crate::preprocessing::node::{HctlTreeNode, NodeType};
use crate::preprocessing::operator_enums::{Atomic, HybridOp};

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
pub fn collect_unique_hctl_vars(formula_tree: HctlTreeNode) -> HashSet<String> {
    collect_unique_hctl_vars_recursive(formula_tree, HashSet::new())
}

fn collect_unique_hctl_vars_recursive(
    formula_tree: HctlTreeNode,
    mut seen_vars: HashSet<String>,
) -> HashSet<String> {
    match formula_tree.node_type {
        NodeType::TerminalNode(_) => {}
        NodeType::UnaryNode(_, child) => {
            seen_vars.extend(collect_unique_hctl_vars_recursive(
                *child,
                seen_vars.clone(),
            ));
        }
        NodeType::BinaryNode(_, left, right) => {
            seen_vars.extend(collect_unique_hctl_vars_recursive(*left, seen_vars.clone()));
            seen_vars.extend(collect_unique_hctl_vars_recursive(
                *right,
                seen_vars.clone(),
            ));
        }
        // collect variables from exist and binder nodes
        NodeType::HybridNode(op, var_name, child) => {
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    seen_vars.insert(var_name); // we do not care whether insert is successful
                }
                _ => {}
            }
            seen_vars.extend(collect_unique_hctl_vars_recursive(
                *child,
                seen_vars.clone(),
            ));
        }
    }
    seen_vars
}

/// Compute the set of all uniquely named `wild-card propositions` in the formula tree.
pub fn collect_unique_wild_card_props(formula_tree: HctlTreeNode) -> HashSet<String> {
    collect_unique_wild_card_props_recursive(formula_tree, HashSet::new())
}

fn collect_unique_wild_card_props_recursive(
    formula_tree: HctlTreeNode,
    mut seen_props: HashSet<String>,
) -> HashSet<String> {
    match formula_tree.node_type {
        NodeType::TerminalNode(atom) => {
            if let Atomic::WildCardProp(prop_name) = atom {
                seen_props.insert(prop_name);
            }
        }
        NodeType::UnaryNode(_, child) => {
            seen_props.extend(collect_unique_wild_card_props_recursive(
                *child,
                seen_props.clone(),
            ));
        }
        NodeType::BinaryNode(_, left, right) => {
            seen_props.extend(collect_unique_wild_card_props_recursive(
                *left,
                seen_props.clone(),
            ));
            seen_props.extend(collect_unique_wild_card_props_recursive(
                *right,
                seen_props.clone(),
            ));
        }
        NodeType::HybridNode(_, _, child) => {
            seen_props.extend(collect_unique_wild_card_props_recursive(
                *child,
                seen_props.clone(),
            ));
        }
    }
    seen_props
}

/// Check that extended symbolic graph's BDD supports enough extra variables for the evaluation of
/// the formula given by a `hctl_syntactic_tree`.
/// There must be `num_hctl_vars` extra symbolic BDD vars for each BN variable.
pub fn check_hctl_var_support(stg: &SymbolicAsyncGraph, hctl_syntactic_tree: HctlTreeNode) -> bool {
    let num_hctl_vars = collect_unique_hctl_vars(hctl_syntactic_tree).len();
    for bn_var in stg.as_network().variables() {
        if num_hctl_vars > stg.symbolic_context().extra_state_variables(bn_var).len() {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::mc_utils::{
        check_hctl_var_support, collect_unique_hctl_vars, collect_unique_wild_card_props,
        get_extended_symbolic_graph,
    };
    use crate::preprocessing::parser::{
        parse_and_minimize_hctl_formula, parse_extended_formula, parse_hctl_formula,
    };
    use crate::preprocessing::utils::check_props_and_rename_vars;

    use biodivine_lib_param_bn::BooleanNetwork;

    use std::collections::{HashMap, HashSet};

    #[test]
    /// Test collecting state vars from HCTL formulae.
    fn test_state_var_collecting() {
        // conjunction of 2 formulae which are semantically same, just use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))";
        let tree = parse_hctl_formula(formula).unwrap();

        // test for original tree
        let expected_vars = HashSet::from_iter(vec![
            "x".to_string(),
            "y".to_string(),
            "z".to_string(),
            "x1".to_string(),
            "y1".to_string(),
            "z1".to_string(),
        ]);
        assert_eq!(collect_unique_hctl_vars(tree.clone()), expected_vars);

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // and for tree with minimized number of renamed state vars
        let modified_tree =
            check_props_and_rename_vars(tree, HashMap::new(), String::new(), &bn).unwrap();
        let expected_vars =
            HashSet::from_iter(vec!["x".to_string(), "xx".to_string(), "xxx".to_string()]);
        assert_eq!(collect_unique_hctl_vars(modified_tree), expected_vars);
    }

    #[test]
    /// Test collecting wild-card propositions from extended HCTL formulae.
    fn test_wild_card_prop_collecting() {
        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %A% & %B%) & (@{y}: %A% & %C%)";
        let tree = parse_extended_formula(formula).unwrap();

        let expected_vars =
            HashSet::from_iter(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        assert_eq!(collect_unique_wild_card_props(tree.clone()), expected_vars);
    }

    #[test]
    /// Test collecting wild-card propositions from extended HCTL formulae.
    fn test_check_hctl_var_support() {
        // define any placeholder bn and stg with enough variables
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // formula with 3 variables
        let formula = "!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))";
        let tree = parse_and_minimize_hctl_formula(&bn, formula).unwrap();

        // the stg that supports the same amount variables as the formula (3)
        let stg = get_extended_symbolic_graph(&bn, 3).unwrap();
        assert!(check_hctl_var_support(&stg, tree.clone()));

        // the stg that supports less variables than the formula (1 vs 3)
        let stg = get_extended_symbolic_graph(&bn, 1).unwrap();
        assert!(!check_hctl_var_support(&stg, tree));
    }
}
