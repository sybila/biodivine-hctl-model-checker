use crate::formula_preprocessing::operation_enums::*;
use crate::formula_preprocessing::parser::*;

use std::collections::HashMap;

/// Renames hctl vars in the formula tree to canonical form - "x", "xx", ...
/// Works only for formulae without free variables
/// Renames as many state-vars as possible to identical names, without changing the semantics
pub fn minimize_number_of_state_vars(
    orig_node: Node,
    mut mapping_dict: HashMap<String, String>,
    mut last_used_name: String,
) -> Node {
    // If we find hybrid node with binder or exist, we add new var-name to rename_dict and stack (x, xx, xxx...)
    // After we leave this binder/exist, we remove its var from rename_dict
    // When we find terminal with free var or jump node, we rename the var using rename-dict
    return match orig_node.node_type {
        // rename vars in terminal state-var nodes
        NodeType::TerminalNode(ref atom) => match atom {
            Atomic::Var(name) => {
                let renamed_var = mapping_dict.get(name.as_str()).unwrap();
                Node {
                    subform_str: format!("{{{}}}", renamed_var.to_string()),
                    height: 0,
                    node_type: NodeType::TerminalNode(Atomic::Var(renamed_var.to_string())),
                }
            }
            _ => return orig_node,
        },
        // just dive one level deeper for unary nodes, and rename string
        NodeType::UnaryNode(op, child) => {
            let node = minimize_number_of_state_vars(*child, mapping_dict, last_used_name.clone());
            create_unary(Box::new(node), op)
        }
        // just dive deeper for binary nodes, and rename string
        NodeType::BinaryNode(op, left, right) => {
            let node1 =
                minimize_number_of_state_vars(*left, mapping_dict.clone(), last_used_name.clone());
            let node2 = minimize_number_of_state_vars(*right, mapping_dict, last_used_name);
            create_binary(Box::new(node1), Box::new(node2), op)
        }
        // hybrid nodes are more complicated
        NodeType::HybridNode(op, var, child) => {
            // if we hit binder or exist, we are adding its new var name to dict & stack
            // no need to do this for jump, jump is not quantifier
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    last_used_name.push('x'); // this represents adding to stack
                    mapping_dict.insert(var.clone(), last_used_name.clone());
                }
                _ => {}
            }

            // dive deeper
            let node =
                minimize_number_of_state_vars(*child, mapping_dict.clone(), last_used_name.clone());

            // rename the variable in the node
            let renamed_var = mapping_dict.get(var.as_str()).unwrap();
            create_hybrid(Box::new(node), renamed_var.clone(), op)
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::formula_preprocessing::parser::parse_hctl_formula;
    use crate::formula_preprocessing::rename_vars::minimize_number_of_state_vars;
    use crate::formula_preprocessing::tokenizer::tokenize_formula;
    use std::collections::HashMap;

    /// Compare tree for formula with automatically minimized state var number to the
    /// tree for the semantically same, but already minimized formula
    fn test_state_var_minimization(formula: String, formula_minimized: String) {
        // automatically modify the original formula
        let tokens = tokenize_formula(formula).unwrap();
        let tree = parse_hctl_formula(&tokens).unwrap();
        let modified_tree = minimize_number_of_state_vars(*tree, HashMap::new(), String::new());

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
}
