//! Contains functionality mostly regarding proposition validating, and manipulation with variables.

use crate::preprocessing::node::*;
use crate::preprocessing::operator_enums::{Atomic, HybridOp};

use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::HashMap;

/// Checks that all vars in formula are quantified (exactly once) and props are valid BN variables.
/// Renames hctl vars in the formula tree to canonical form - "x", "xx", ...
/// Renames as many state-vars as possible to identical names, without changing the semantics.
pub fn check_props_and_rename_vars(
    orig_node: HctlTreeNode,
    mut mapping_dict: HashMap<String, String>,
    mut last_used_name: String,
    bn: &BooleanNetwork,
) -> Result<HctlTreeNode, String> {
    // If we find hybrid node with binder or exist, we add new var-name to rename_dict and stack (x, xx, xxx...)
    // After we leave this binder/exist, we remove its var from rename_dict
    // When we find terminal with free var or jump node, we rename the var using rename-dict
    return match orig_node.node_type {
        // rename vars in terminal state-var nodes
        NodeType::TerminalNode(ref atom) => match atom {
            Atomic::Var(name) => {
                // check that variable is not free (it must be already in mapping dict)
                if !mapping_dict.contains_key(name.as_str()) {
                    return Err(format!("Variable {name} is free."));
                }
                let renamed_var = mapping_dict.get(name.as_str()).unwrap();
                Ok(HctlTreeNode::mk_var_node(renamed_var.to_string()))
            }
            Atomic::Prop(name) => {
                // check that proposition corresponds to valid BN variable
                let network_variable = bn.as_graph().find_variable(name);
                if network_variable.is_none() {
                    return Err(format!("There is no network variable named {name}."));
                }
                Ok(orig_node)
            }
            // constants or wild-card propositions are always considered fine
            _ => return Ok(orig_node),
        },
        // just dive one level deeper for unary nodes, and rename string
        NodeType::UnaryNode(op, child) => {
            let node =
                check_props_and_rename_vars(*child, mapping_dict, last_used_name.clone(), bn)?;
            Ok(HctlTreeNode::mk_unary_node(node, op))
        }
        // just dive deeper for binary nodes, and rename string
        NodeType::BinaryNode(op, left, right) => {
            let node1 = check_props_and_rename_vars(
                *left,
                mapping_dict.clone(),
                last_used_name.clone(),
                bn,
            )?;
            let node2 = check_props_and_rename_vars(*right, mapping_dict, last_used_name, bn)?;
            Ok(HctlTreeNode::mk_binary_node(node1, node2, op))
        }
        // hybrid nodes are more complicated
        NodeType::HybridNode(op, var, child) => {
            // if we hit binder or exist, we are adding its new var name to dict & stack
            // no need to do this for jump, jump is not quantifier
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    // check that var is not already quantified
                    if mapping_dict.contains_key(var.as_str()) {
                        return Err(format!(
                            "Variable {var} is quantified several times in one sub-formula"
                        ));
                    }
                    last_used_name.push('x'); // this represents adding to stack
                    mapping_dict.insert(var.clone(), last_used_name.clone());
                }
                _ => {}
            }

            // dive deeper
            let node = check_props_and_rename_vars(
                *child,
                mapping_dict.clone(),
                last_used_name.clone(),
                bn,
            )?;

            // rename the variable in the node
            let renamed_var = mapping_dict.get(var.as_str()).unwrap();
            Ok(HctlTreeNode::mk_hybrid_node(node, renamed_var.clone(), op))
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::preprocessing::parser::{
        parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula,
        parse_extended_formula, parse_hctl_formula,
    };
    use crate::preprocessing::utils::check_props_and_rename_vars;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    /// Compare tree for formula with automatically minimized state var number to the
    /// tree for the semantically same, but already minimized formula.
    fn test_state_var_minimization(formula: &str, formula_minimized: &str) {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // automatically modify the original formula via preprocessing step
        let tree = parse_and_minimize_hctl_formula(&bn, formula).unwrap();

        // get expected tree using the (same) formula with already manually minimized vars
        let tree_minimized = parse_hctl_formula(formula_minimized).unwrap();

        assert_eq!(tree_minimized, tree);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming.
    fn test_state_var_minimization_simple() {
        let formula = "(!{x}: AG EF {x}) | (!{y}: !{x}: (AX {y} & AX {x})) | (!{z}: AG EF {z})";
        // same formula with already minimized vars
        let formula_minimized =
            "(!{x}: AG EF {x}) | (!{x}: !{xx}: (AX {x} & AX {xx})) | (!{x}: AG EF {x})";

        test_state_var_minimization(formula, formula_minimized);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming.
    fn test_state_var_minimization_complex() {
        // conjunction of two semantically same formulae (for FORK states) that use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))";
        // same formula with already minimized vars
        let formula_minimized = "(!{x}: 3{xx}: (@{x}: ~{xx} & (!{xxx}: AX {xxx})) & (@{xx}: (!{xxx}: AX {xxx}))) & (!{x}: 3{xx}: (@{x}: ~{xx} & (!{xxx}: AX {xxx})) & (@{xx}: (!{xxx}: AX {xxx})))";

        test_state_var_minimization(formula, formula_minimized);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming, but this time with
    /// formula containing wild-card propositions (to check they are not affected).
    fn test_state_var_minimization_with_wild_cards() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %subst%) & (@{y}: %subst%)";
        // same formula with already minimized vars
        let formula_minimized = "!{x}: 3{xx}: (@{x}: ~{xx} & %subst%) & (@{xx}: %subst%)";

        let tree = parse_and_minimize_extended_formula(&bn, formula).unwrap();
        // get expected tree using the (same) formula with already manually minimized vars
        let tree_minimized = parse_extended_formula(formula_minimized).unwrap();
        assert_eq!(tree_minimized, tree);
    }

    #[test]
    /// Test that function errors correctly if formula contains free variables.
    fn test_check_vars_error_1() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // define and parse formula with free variable
        let formula = "AX {x}";
        let tree = parse_hctl_formula(formula).unwrap();

        assert!(check_props_and_rename_vars(tree, HashMap::new(), String::new(), &bn).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains several times quantified vars.
    fn test_check_vars_error_2() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // define and parse formula with two variables
        let formula = "!{x}: !{x}: AX {x}";
        let tree = parse_hctl_formula(formula).unwrap();

        assert!(check_props_and_rename_vars(tree, HashMap::new(), String::new(), &bn).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains invalid propositions.
    fn test_check_props_error_1() {
        // define a placeholder bn with only 1 prop v1
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // define and parse formula with invalid proposition
        let formula = "AX invalid_proposition";
        let tree = parse_hctl_formula(formula).unwrap();

        assert!(check_props_and_rename_vars(tree, HashMap::new(), String::new(), &bn).is_err());
    }
}
