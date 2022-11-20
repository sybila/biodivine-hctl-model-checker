use crate::formula_preprocessing::operation_enums::*;
use crate::formula_preprocessing::parser::*;

use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::HashMap;

/// Checks that all vars in formula are quantified (exactly once) and props are valid
/// Renames hctl vars in the formula tree to canonical form - "x", "xx", ...
/// Renames as many state-vars as possible to identical names, without changing the semantics
pub fn check_props_and_rename_vars(
    orig_node: Node,
    mut mapping_dict: HashMap<String, String>,
    mut last_used_name: String,
    bn: &BooleanNetwork
) -> Result<Node, String> {
    // If we find hybrid node with binder or exist, we add new var-name to rename_dict and stack (x, xx, xxx...)
    // After we leave this binder/exist, we remove its var from rename_dict
    // When we find terminal with free var or jump node, we rename the var using rename-dict
    return match orig_node.node_type {
        // rename vars in terminal state-var nodes
        NodeType::TerminalNode(ref atom) => match atom {
            Atomic::Var(name) => {
                // check that variable is not free (it must be already in mapping dict)
                if !mapping_dict.contains_key(name.as_str()) {
                    return Err(format!("Variable {} is free.", name));
                }
                let renamed_var = mapping_dict.get(name.as_str()).unwrap();
                Ok(Node {
                    subform_str: format!("{{{}}}", renamed_var.to_string()),
                    height: 0,
                    node_type: NodeType::TerminalNode(Atomic::Var(renamed_var.to_string())),
                })
            }
            Atomic::Prop(name) => {
                // check that proposition corresponds to valid BN variable
                let network_variable = bn.as_graph().find_variable(name);
                if network_variable.is_none() {
                    return Err(format!("There is no network variable named {}.", name));
                }
                Ok(orig_node)
            }
            // constants are always fine
            _ => return Ok(orig_node),
        },
        // just dive one level deeper for unary nodes, and rename string
        NodeType::UnaryNode(op, child) => {
            let node = check_props_and_rename_vars(*child, mapping_dict, last_used_name.clone(), bn)?;
            Ok(create_unary(Box::new(node), op))
        }
        // just dive deeper for binary nodes, and rename string
        NodeType::BinaryNode(op, left, right) => {
            let node1 =
                check_props_and_rename_vars(*left, mapping_dict.clone(), last_used_name.clone(), bn);
            let node2 = check_props_and_rename_vars(*right, mapping_dict, last_used_name, bn);
            Ok(create_binary(Box::new(node1?), Box::new(node2?), op))
        }
        // hybrid nodes are more complicated
        NodeType::HybridNode(op, var, child) => {
            // if we hit binder or exist, we are adding its new var name to dict & stack
            // no need to do this for jump, jump is not quantifier
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    // check that var is not already quantified
                    if mapping_dict.contains_key(var.as_str()) {
                        return Err(format!("Variable {} is quantified several times in one sub-formula", var));
                    }
                    last_used_name.push('x'); // this represents adding to stack
                    mapping_dict.insert(var.clone(), last_used_name.clone());
                }
                _ => {}
            }

            // dive deeper
            let node =
                check_props_and_rename_vars(*child, mapping_dict.clone(), last_used_name.clone(), bn)?;

            // rename the variable in the node
            let renamed_var = mapping_dict.get(var.as_str()).unwrap();
            Ok(create_hybrid(Box::new(node), renamed_var.clone(), op))
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::formula_preprocessing::parser::parse_hctl_formula;
    use crate::formula_preprocessing::vars_props_manipulation::check_props_and_rename_vars;
    use crate::formula_preprocessing::tokenizer::tokenize_formula;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    /// Compare tree for formula with automatically minimized state var number to the
    /// tree for the semantically same, but already minimized formula
    fn test_state_var_minimization(formula: String, formula_minimized: String) {
        // automatically modify the original formula
        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        let modified_tree = check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).unwrap();

        // get expected tree using the same formula with already minimized vars
        let tokens_minimized = tokenize_formula(formula_minimized).unwrap();
        let tree_minimized = parse_hctl_formula(&tokens_minimized).unwrap();

        assert_eq!(*tree_minimized, modified_tree);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming
    fn test_state_var_minimization_simple() {
        let formula = "(!{x}: AG EF {x}) | (!{y}: !{x}: (AX {y} & AX {x})) | (!{z}: AG EF {z})";

        // same formula with already minimized vars
        let formula_minimized =
            "(!{x}: AG EF {x}) | (!{x}: !{xx}: (AX {x} & AX {xx})) | (!{x}: AG EF {x})";

        test_state_var_minimization(formula.to_string(), formula_minimized.to_string());
    }

    #[test]
    /// Test minimization of number of state variables and their renaming
    fn test_state_var_minimization_complex() {
        // formula "FORKS1 & FORKS2" - both parts are semantically same, just use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))";

        // same formula with already minimized vars
        let formula_minimized = "(!{x}: 3{xx}: (@{x}: ~{xx} & (!{xxx}: AX {xxx})) & (@{xx}: (!{xxx}: AX {xxx}))) & (!{x}: 3{xx}: (@{x}: ~{xx} & (!{xxx}: AX {xxx})) & (@{xx}: (!{xxx}: AX {xxx})))";

        test_state_var_minimization(formula.to_string(), formula_minimized.to_string());
    }

    #[test]
    /// Test that function errors correctly if formula contains free variables
    fn test_check_vars_error_1() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        // define formula with free variable
        let formula = "AX {x}".to_string();
        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        assert!(check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains several times quantified vars
    fn test_check_vars_error_2() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        // define formula with two variables
        let formula = "!{x}: !{x}: AX {x}".to_string();
        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        assert!(check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains invalid propositions
    fn test_check_props_error_1() {
        // define a placeholder bn with only 1 prop v1
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();

        // define formula with invalid proposition
        let formula = "AX invalid_proposition".to_string();
        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();

        assert!(check_props_and_rename_vars(*tree, HashMap::new(), String::new(), &bn).is_err());
    }
}
