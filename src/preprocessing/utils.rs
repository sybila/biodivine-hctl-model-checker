//! Functionality mostly regarding validation of propositions, and manipulation with variables of
//! syntactic trees.

use crate::evaluation::LabelToSetMap;
use crate::mc_utils::collect_unique_wild_cards;
use crate::preprocessing::hctl_tree::*;
use crate::preprocessing::operator_enums::{Atomic, HybridOp};
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
use std::collections::HashMap;

/// Checks that all HCTL variables in the formula's syntactic tree are quantified (exactly once) and that
/// its propositions are valid BN variables.
/// Then renames all HCTL vars in the formula's tree to a pseudo-canonical form - "x", "xx", ...
/// It renames as many variables as possible to identical names, without affecting the semantics.
pub fn validate_props_and_rename_vars(
    orig_tree: HctlTreeNode,
    symbolic_context: &SymbolicContext,
) -> Result<HctlTreeNode, String> {
    validate_and_rename_recursive(orig_tree, HashMap::new(), String::new(), symbolic_context)
}

/// Checks that all HCTL variables in the formula's syntactic tree are quantified (exactly once) and that
/// its propositions are valid BN variables.
/// Then renames all HCTL vars in the formula's tree to a pseudo-canonical form - "x", "xx", ...
/// It renames as many variables as possible to identical names, without affecting the semantics.
fn validate_and_rename_recursive(
    orig_tree: HctlTreeNode,
    mut renaming_map: HashMap<String, String>,
    mut last_used_name: String,
    ctx: &SymbolicContext,
) -> Result<HctlTreeNode, String> {
    // If we find hybrid node with binder or exist, we add new var-name to rename_dict and stack (x, xx, xxx...)
    // After we leave this binder/exist, we remove its var from rename_dict
    // When we find terminal with free var or jump node, we rename the var using rename-dict
    return match orig_tree.node_type {
        // rename vars in terminal state-var nodes
        NodeType::Terminal(ref atom) => match atom {
            Atomic::Var(name) => {
                // check that variable is not free (it must be already in mapping dict)
                if !renaming_map.contains_key(name.as_str()) {
                    return Err(format!("Variable {name} is free."));
                }
                let renamed_var = renaming_map.get(name.as_str()).unwrap();
                Ok(HctlTreeNode::mk_variable(renamed_var))
            }
            Atomic::Prop(name) => {
                // check that proposition corresponds to valid BN variable
                if ctx.find_network_variable(name).is_none() {
                    Err(format!("There is no network variable named {name}."))
                } else {
                    Ok(orig_tree)
                }
            }
            // constants or wild-card propositions are always considered fine
            _ => return Ok(orig_tree),
        },
        // just dive one level deeper for unary nodes, and rename string
        NodeType::Unary(op, child) => {
            let node =
                validate_and_rename_recursive(*child, renaming_map, last_used_name.clone(), ctx)?;
            Ok(HctlTreeNode::mk_unary(node, op))
        }
        // just dive deeper for binary nodes, and rename string
        NodeType::Binary(op, left, right) => {
            let node1 = validate_and_rename_recursive(
                *left,
                renaming_map.clone(),
                last_used_name.clone(),
                ctx,
            )?;
            let node2 = validate_and_rename_recursive(*right, renaming_map, last_used_name, ctx)?;
            Ok(HctlTreeNode::mk_binary(node1, node2, op))
        }
        // hybrid nodes are more complicated
        NodeType::Hybrid(op, var, domain, child) => {
            // if we hit binder or exist, we are adding its new var name to dict & stack
            // no need to do this for jump, jump is not quantifier
            match op {
                HybridOp::Bind | HybridOp::Exists | HybridOp::Forall => {
                    // check that var is not already quantified (we dont allow that)
                    if renaming_map.contains_key(var.as_str()) {
                        return Err(format!(
                            "Variable {var} is quantified several times in one sub-formula."
                        ));
                    }
                    last_used_name.push('x'); // this represents adding to stack
                    renaming_map.insert(var.clone(), last_used_name.clone());
                }
                _ => {}
            }

            // dive deeper
            let node = validate_and_rename_recursive(
                *child,
                renaming_map.clone(),
                last_used_name.clone(),
                ctx,
            )?;

            // if current operator is jump, make sure that it does not contain free var
            if matches!(op, HybridOp::Jump) && !renaming_map.contains_key(var.as_str()) {
                return Err(format!("Variable {var} is free in `@{{{var}}}:`."));
            }

            // rename the variable in the node
            let renamed_var = renaming_map.get(var.as_str()).unwrap();
            Ok(HctlTreeNode::mk_hybrid(
                node,
                renamed_var.as_str(),
                domain,
                op,
            ))
        }
    };
}

/// Check that all wild-card propositions and variable domains in the formula's syntactic tree have
/// their corresponding "raw set" (context) in `context_sets`.
pub fn validate_wild_cards(
    tree: &HctlTreeNode,
    context_sets: &LabelToSetMap,
) -> Result<(), String> {
    let (wild_card_props, var_domains) = collect_unique_wild_cards(tree.clone());
    // check that all occurring wild-card props are present in `context_props`
    for wild_card in wild_card_props {
        if !context_sets.contains_key(wild_card.as_str()) {
            return Err(format!(
                "Wild-card prop `{}` lacks evaluation context.",
                wild_card
            ));
        }
    }
    // check that all occurring wild-card props are present in `context_domains`
    for var_domain in var_domains {
        if !context_sets.contains_key(var_domain.as_str()) {
            return Err(format!(
                "Var domain `{}` lacks evaluation context.",
                var_domain
            ));
        }
    }
    Ok(())
}

/// Check that all wild-card propositions and variable domains in the formula's syntactic tree have
/// their corresponding "raw set" (context) in `substitution_context`.
///
/// Returns two individual context subsets, one for wild-card propositions, and the other for variable domains.
pub fn validate_and_divide_wild_cards(
    tree: &HctlTreeNode,
    context_sets: &LabelToSetMap,
) -> Result<(LabelToSetMap, LabelToSetMap), String> {
    let mut context_domains = HashMap::new();
    let mut context_props = HashMap::new();

    let (wild_card_props, var_domains) = collect_unique_wild_cards(tree.clone());
    // check that all occurring wild-card props are present in `substitution_context`
    for wild_card in wild_card_props {
        if !context_sets.contains_key(wild_card.as_str()) {
            return Err(format!(
                "Wild-card prop `{}` lacks evaluation context.",
                wild_card
            ));
        } else {
            context_props.insert(
                wild_card.clone(),
                context_sets.get(&wild_card).unwrap().clone(),
            );
        }
    }
    // check that all occurring wild-card props are present in `substitution_context`
    for var_domain in var_domains {
        if !context_sets.contains_key(var_domain.as_str()) {
            return Err(format!(
                "Var domain `{}` lacks evaluation context.",
                var_domain
            ));
        } else {
            context_domains.insert(
                var_domain.clone(),
                context_sets.get(&var_domain).unwrap().clone(),
            );
        }
    }
    Ok((context_props, context_domains))
}

#[cfg(test)]
mod tests {
    use crate::mc_utils::get_extended_symbolic_graph;
    use crate::preprocessing::parser::{
        parse_and_minimize_extended_formula, parse_and_minimize_hctl_formula,
        parse_extended_formula, parse_hctl_formula,
    };
    use crate::preprocessing::utils::{
        validate_and_divide_wild_cards, validate_props_and_rename_vars, validate_wild_cards,
    };
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;
    use biodivine_lib_param_bn::BooleanNetwork;
    use std::collections::HashMap;

    /// Compare tree for formula with automatically minimized state var number to the
    /// tree for the semantically same, but already minimized formula.
    fn check_state_var_minimization(formula: &str, formula_minimized: &str) {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        // automatically modify the original formula via preprocessing step
        let tree = parse_and_minimize_hctl_formula(&ctx, formula).unwrap();

        // get expected tree using the (same) formula with already manually minimized vars
        let tree_minimized = parse_hctl_formula(formula_minimized).unwrap();

        assert_eq!(tree_minimized, tree);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming.
    fn state_var_minimization_simple() {
        let formula = "(!{x}: AG EF {x}) | (!{y}: !{x}: (AX {y} & AX {x})) | (!{z}: AG EF {z})";
        // same formula with already minimized vars
        let formula_minimized =
            "(!{x}: AG EF {x}) | (!{x}: !{xx}: (AX {x} & AX {xx})) | (!{x}: AG EF {x})";

        check_state_var_minimization(formula, formula_minimized);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming.
    fn state_var_minimization_complex() {
        // conjunction of two semantically same formulae (for FORK states) that use different var names
        let formula = "(!{x}: 3{y}: (@{x}: ~{y} & (!{z}: AX {z})) & (@{y}: (!{z}: AX {z}))) & (!{x1}: 3{y1}: (@{x1}: ~{y1} & (!{z1}: AX {z1})) & (@{y1}: (!{z1}: AX {z1})))";
        // same formula with already minimized vars
        let formula_minimized = "(!{x}: 3{xx}: (@{x}: ~{xx} & (!{xxx}: AX {xxx})) & (@{xx}: (!{xxx}: AX {xxx}))) & (!{x}: 3{xx}: (@{x}: ~{xx} & (!{xxx}: AX {xxx})) & (@{xx}: (!{xxx}: AX {xxx})))";

        check_state_var_minimization(formula, formula_minimized);
    }

    #[test]
    /// Test minimization of number of state variables and their renaming, but this time with
    /// formula containing wild-card propositions and domains (to check they are not affected).
    fn state_var_minimization_with_wild_cards() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let ctx = SymbolicContext::new(&bn).unwrap();

        let formula = "!{x} in %1%: 3{y} in %2%: (@{x}: ~{y} & %subst%) & (@{y}: %subst%)";
        // same formula with already minimized vars
        let formula_minimized =
            "!{x} in %1%: 3{xx} in %2%: (@{x}: ~{xx} & %subst%) & (@{xx}: %subst%)";

        let tree = parse_and_minimize_extended_formula(&ctx, formula).unwrap();
        // get expected tree using the (same) formula with already manually minimized vars
        let tree_minimized = parse_extended_formula(formula_minimized).unwrap();
        assert_eq!(tree_minimized, tree);
    }

    #[test]
    /// Test the utility functions for validating and processing wild-cards in extended formulae.
    fn test_validate_wild_cards() {
        // create placeholder bn and symbolic graph
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let stg = get_extended_symbolic_graph(&bn, 2).unwrap();

        // test simple validation
        let context_sets = HashMap::from([
            ("p".to_string(), stg.mk_empty_colored_vertices()),
            ("d".to_string(), stg.mk_empty_colored_vertices()),
        ]);
        let formula = "!{x} in %d%: EF %p%";
        let tree = parse_and_minimize_extended_formula(stg.symbolic_context(), formula).unwrap();
        let res = validate_wild_cards(&tree, &context_sets);
        assert!(res.is_ok());

        // test validation combined with dividing set of wild-cards into propositions and domains
        let context_props = HashMap::from([("p".to_string(), stg.mk_empty_colored_vertices())]);
        let context_domains = HashMap::from([("d".to_string(), stg.mk_empty_colored_vertices())]);
        let (context1, context2) = validate_and_divide_wild_cards(&tree, &context_sets).unwrap();
        assert_eq!(context1, context_props);
        assert_eq!(context2, context_domains);
    }

    #[test]
    /// Test that function errors correctly if formula contains free variables.
    fn validation_error_free_vars() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let symbolic_context = SymbolicContext::new(&bn).unwrap();

        // define and parse formula with free variable
        let formula = "AX {x}";
        let tree = parse_hctl_formula(formula).unwrap();
        let result = validate_props_and_rename_vars(tree, &symbolic_context);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Variable x is free.".to_string());

        // define and parse formula with free variable in jump operator
        let formula = "@{x}: v1";
        let tree = parse_hctl_formula(formula).unwrap();
        let result = validate_props_and_rename_vars(tree, &symbolic_context);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Variable x is free in `@{x}:`.".to_string()
        );
    }

    #[test]
    /// Test that function errors correctly if formula contains several times quantified vars.
    fn validation_error_invalid_quantification() {
        // define any placeholder bn
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let symbolic_context = SymbolicContext::new(&bn).unwrap();

        // define and parse formula with two variables
        let formula = "!{x}: !{x}: AX {x}";
        let tree = parse_hctl_formula(formula).unwrap();

        assert!(validate_props_and_rename_vars(tree, &symbolic_context).is_err());
    }

    #[test]
    /// Test that function errors correctly if formula contains invalid propositions.
    fn validation_error_invalid_propositions() {
        // define a placeholder bn with only 1 prop v1
        let bn = BooleanNetwork::try_from_bnet("v1, v1").unwrap();
        let symbolic_context = SymbolicContext::new(&bn).unwrap();

        // define and parse formula with invalid proposition
        let formula = "AX invalid_proposition";
        let tree = parse_hctl_formula(formula).unwrap();

        assert!(validate_props_and_rename_vars(tree, &symbolic_context).is_err());
    }
}
