//! Contains functionality regarding parsing formula (or formula tokens) into a syntax tree.
//!
//! The operator precedence is following (the lower, the stronger):
//!  - unary operators (negation + temporal): 1
//!  - binary temporal operators: 2
//!  - boolean binary operators: and=3, xor=4, or=5, imp=6, equiv=7
//!  - hybrid operators: 8
//!

use crate::preprocessing::node::*;
use crate::preprocessing::operator_enums::*;
use crate::preprocessing::tokenizer::{try_tokenize_formula, HctlToken, try_tokenize_extended_formula};
use crate::preprocessing::utils::check_props_and_rename_vars;

use biodivine_lib_param_bn::BooleanNetwork;

use std::collections::HashMap;

/// Parse an HCTL formula string representation into an actual formula tree.
/// Basically a wrapper for tokenize+parse (used often for testing/debug purposes).
///
/// NEEDS to call procedure for renaming variables to fully finish the preprocessing step.
pub fn parse_hctl_formula(formula: &str) -> Result<HctlTreeNode, String> {
    let tokens = try_tokenize_formula(formula.to_string())?;
    let tree = parse_hctl_tokens(&tokens)?;
    Ok(tree)
}

/// Parse an extended HCTL formula string representation into an actual formula tree.
/// Extended formulae can include `wild-card propositions` in form "%proposition%".
///
/// NEEDS to call procedure for renaming variables to fully finish the preprocessing step.
pub fn parse_extended_formula(formula: &str) -> Result<HctlTreeNode, String> {
    let tokens = try_tokenize_extended_formula(formula.to_string())?;
    let tree = parse_hctl_tokens(&tokens)?;
    Ok(tree)
}

/// Parse an HCTL formula string representation into an actual formula tree with renamed (minimized)
/// set of variables.
/// Basically a wrapper for the whole preprocessing step (tokenize + parse + rename vars).
pub fn parse_and_minimize_hctl_formula(
    bn: &BooleanNetwork,
    formula: &str,
) -> Result<HctlTreeNode, String> {
    let tree = parse_hctl_formula(formula)?;
    let tree = check_props_and_rename_vars(tree, HashMap::new(), String::new(), bn)?;
    Ok(tree)
}

/// Parse an extended HCTL formula string representation into an actual formula tree
/// with renamed (minimized) set of variables.
/// Extended formulae can include `wild-card propositions` in form "%proposition%".
pub fn parse_and_minimize_extended_formula(
    bn: &BooleanNetwork,
    formula: &str,
) -> Result<HctlTreeNode, String> {
    let tree = parse_extended_formula(formula)?;
    let tree = check_props_and_rename_vars(tree, HashMap::new(), String::new(), bn)?;
    Ok(tree)
}


/// Predicate for whether given token represents hybrid operator.
fn is_hybrid(token: &HctlToken) -> bool {
    matches!(token, HctlToken::Hybrid(_, _))
}

/// Predicate for whether given token represents temporal binary operator.
fn is_binary_temporal(token: &HctlToken) -> bool {
    matches!(
        token,
        HctlToken::Binary(BinaryOp::Eu)
            | HctlToken::Binary(BinaryOp::Au)
            | HctlToken::Binary(BinaryOp::Ew)
            | HctlToken::Binary(BinaryOp::Aw)
    )
}

/// Predicate for whether given token represents unary operator.
fn is_unary(token: &HctlToken) -> bool {
    matches!(token, HctlToken::Unary(_))
}

/// Utility method to find the first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[HctlToken], token: HctlToken) -> Option<usize> {
    return tokens.iter().position(|t| *t == token);
}

/// Utility method to find the first occurrence of a hybrid operator in the token tree.
fn index_of_first_hybrid(tokens: &[HctlToken]) -> Option<usize> {
    return tokens.iter().position(is_hybrid);
}

/// Utility method to find the first occurrence of a binary temporal operator in the token tree.
fn index_of_first_binary_temp(tokens: &[HctlToken]) -> Option<usize> {
    return tokens.iter().position(is_binary_temporal);
}

/// Utility method to find the first occurrence of an unary operator in the token tree.
fn index_of_first_unary(tokens: &[HctlToken]) -> Option<usize> {
    return tokens.iter().position(is_unary);
}

/// Parse `tokens` of HCTL formula into an abstract syntax tree using recursive steps.
pub fn parse_hctl_tokens(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    parse_1_hybrid(tokens)
}

/// Recursive parsing step 1: extract hybrid operators.
/// Hybrid operator must not be immediately preceded by any other kind of operator.
/// We only allow it to be preceded by another hybrid operator, or parentheses must be used.
/// (things like "AF !{x}: ..." are forbidden, must be written in brackets as "AF (!{x}: ...)"
fn parse_1_hybrid(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let hybrid_token = index_of_first_hybrid(tokens);
    Ok(if let Some(i) = hybrid_token {
        // perform check that hybrid operator is not preceded by other type of operators
        if i > 0 && !matches!(&tokens[i - 1], HctlToken::Hybrid(_, _)) {
            return Err(format!(
                "Hybrid operator can't be directly preceded by {}.",
                &tokens[i - 1]
            ));
        }
        match &tokens[i] {
            HctlToken::Hybrid(op, var) => {
                create_hybrid_node(parse_1_hybrid(&tokens[(i + 1)..])?, var.clone(), op.clone())
            }
            _ => HctlTreeNode::new(), // This branch cant happen, but must result in same type
        }
    } else {
        parse_2_iff(tokens)?
    })
}

/// Recursive parsing step 2: extract `<=>` operators.
fn parse_2_iff(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let iff_token = index_of_first(tokens, HctlToken::Binary(BinaryOp::Iff));
    Ok(if let Some(i) = iff_token {
        create_binary_node(
            parse_3_imp(&tokens[..i])?,
            parse_2_iff(&tokens[(i + 1)..])?,
            BinaryOp::Iff,
        )
    } else {
        parse_3_imp(tokens)?
    })
}

/// Recursive parsing step 3: extract `=>` operators.
fn parse_3_imp(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let imp_token = index_of_first(tokens, HctlToken::Binary(BinaryOp::Imp));
    Ok(if let Some(i) = imp_token {
        create_binary_node(
            parse_4_or(&tokens[..i])?,
            parse_3_imp(&tokens[(i + 1)..])?,
            BinaryOp::Imp,
        )
    } else {
        parse_4_or(tokens)?
    })
}

/// Recursive parsing step 4: extract `|` operators.
fn parse_4_or(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let or_token = index_of_first(tokens, HctlToken::Binary(BinaryOp::Or));
    Ok(if let Some(i) = or_token {
        create_binary_node(
            parse_5_xor(&tokens[..i])?,
            parse_4_or(&tokens[(i + 1)..])?,
            BinaryOp::Or,
        )
    } else {
        parse_5_xor(tokens)?
    })
}

/// Recursive parsing step 5: extract `^` operators.
fn parse_5_xor(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let xor_token = index_of_first(tokens, HctlToken::Binary(BinaryOp::Xor));
    Ok(if let Some(i) = xor_token {
        create_binary_node(
            parse_6_and(&tokens[..i])?,
            parse_5_xor(&tokens[(i + 1)..])?,
            BinaryOp::Xor,
        )
    } else {
        parse_6_and(tokens)?
    })
}

/// Recursive parsing step 6: extract `&` operators.
fn parse_6_and(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let and_token = index_of_first(tokens, HctlToken::Binary(BinaryOp::And));
    Ok(if let Some(i) = and_token {
        create_binary_node(
            parse_7_binary_temp(&tokens[..i])?,
            parse_6_and(&tokens[(i + 1)..])?,
            BinaryOp::And,
        )
    } else {
        parse_7_binary_temp(tokens)?
    })
}

/// Recursive parsing step 7: extract binary temporal operators.
fn parse_7_binary_temp(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let binary_token = index_of_first_binary_temp(tokens);
    Ok(if let Some(i) = binary_token {
        match &tokens[i] {
            HctlToken::Binary(op) => create_binary_node(
                parse_8_unary(&tokens[..i])?,
                parse_7_binary_temp(&tokens[(i + 1)..])?,
                op.clone(),
            ),
            _ => HctlTreeNode::new(), // This branch cant happen, but must result in same type
        }
    } else {
        parse_8_unary(tokens)?
    })
}

/// Recursive parsing step 8: extract unary temporal operators and negations.
fn parse_8_unary(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let unary_token = index_of_first_unary(tokens);
    Ok(if let Some(i) = unary_token {
        match &tokens[i] {
            HctlToken::Unary(op) => create_unary_node(parse_8_unary(&tokens[(i + 1)..])?, op.clone()),
            _ => HctlTreeNode::new(), // This branch cant happen, but must result in same type
        }
    } else {
        parse_9_terminal_and_parentheses(tokens)?
    })
}

/// Recursive parsing step 9: extract terminals and recursively solve sub-formulae in parentheses.
fn parse_9_terminal_and_parentheses(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    if tokens.is_empty() {
        Err("Expected formula, found nothing.".to_string())
    } else {
        if tokens.len() == 1 {
            // This should be name (var/prop/wild-card prop) or a parenthesis group, anything
            // else does not make sense (constants are tokenized as propositions until now).
            match &tokens[0] {
                HctlToken::Atom(Atomic::Prop(name)) => {
                    return if name == "true" || name == "True" || name == "1" {
                        Ok(create_constant_node(true))
                    } else if name == "false" || name == "False" || name == "0" {
                        Ok(create_constant_node(false))
                    } else {
                        Ok(create_prop_node(name.clone()))
                    }
                }
                HctlToken::Atom(Atomic::Var(name)) => return Ok(create_var_node(name.clone())),
                HctlToken::Atom(Atomic::WildCardProp(name)) => return Ok(create_wild_card_node(name.clone())),
                // recursively solve sub-formulae in parentheses
                HctlToken::Tokens(inner) => return parse_hctl_tokens(inner),
                _ => {} // otherwise, fall through to the error at the end.
            }
        }
        Err(format!("Unexpected: {tokens:?}. Expecting formula."))
    }
}

#[cfg(test)]
mod tests {
    use crate::preprocessing::node::*;
    use crate::preprocessing::operator_enums::*;
    use crate::preprocessing::parser::parse_hctl_formula;

    #[test]
    /// Test whether several valid HCTL formulae are parsed without causing errors.
    /// Also check that the formula is saved correctly in the tree root.
    fn test_parse_valid_formulae() {
        let valid1 = "!{x}: AG EF {x}".to_string();
        assert!(parse_hctl_formula(valid1.as_str()).is_ok());
        let tree = parse_hctl_formula(valid1.as_str()).unwrap();
        assert_eq!(tree.subform_str, "(Bind {x}: (Ag (Ef {x})))".to_string());

        let valid2 = "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string();
        assert!(parse_hctl_formula(valid2.as_str()).is_ok());
        let tree = parse_hctl_formula(valid2.as_str()).unwrap();
        assert_eq!(
            tree.subform_str,
            "(Bind {x}: (Exists {y}: ((Jump {x}: ((~ {y}) & (Ax {x}))) & (Jump {y}: (Ax {y})))))"
                .to_string()
        );

        let valid3 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))".to_string();
        assert!(parse_hctl_formula(valid3.as_str()).is_ok());
        let tree = parse_hctl_formula(valid3.as_str()).unwrap();
        assert_eq!(tree.subform_str, "(Exists {x}: (Exists {y}: ((Jump {x}: ((~ {y}) & (Ax {x}))) & ((Jump {y}: (Ax {y})) & ((Ef ({x} & (Bind {z}: (Ax {z})))) & ((Ef ({y} & (Bind {z}: (Ax {z})))) & (Ax ((Ef ({x} & (Bind {z}: (Ax {z})))) ^ (Ef ({y} & (Bind {z}: (Ax {z}))))))))))))".to_string());

        // also test propositions and constants
        // propositions names should not be modified, constants should be unified to True/False
        let valid4 = "(prop1 & PROP2 | false) AU (True & 0)".to_string();
        assert!(parse_hctl_formula(valid4.as_str()).is_ok());
        let tree = parse_hctl_formula(valid4.as_str()).unwrap();
        assert_eq!(
            tree.subform_str,
            "(((prop1 & PROP2) | False) Au (True & False))".to_string()
        );
    }

    #[test]
    /// Test parsing of several valid HCTL formulae against expected results.
    fn compare_parser_with_expected() {
        let formula = "(false & p1)".to_string();
        let expected_tree = create_binary_node(
            create_constant_node(false),
            create_prop_node("p1".to_string()),
            BinaryOp::And,
        );
        assert_eq!(parse_hctl_formula(formula.as_str()).unwrap(), expected_tree);

        let formula = "!{x}: (AX {x})".to_string();
        let expected_tree = create_hybrid_node(
            create_unary_node(create_var_node("x".to_string()), UnaryOp::Ax),
            "x".to_string(),
            HybridOp::Bind,
        );
        assert_eq!(parse_hctl_formula(formula.as_str()).unwrap(), expected_tree);
    }

    #[test]
    /// Test parsing of several invalid HCTL formulae.
    fn test_parse_invalid_formulae() {
        let invalid_formulae = vec![
            "!{x}: AG EK {x}",
            "!{x}: AG F {x}",
            "!{x}: AG EU {x}",
            "!{x}: TU EK {x}",
            "!{x}: AU AU {x}",
            "& prop",
            "prop1 prop2",
            "AU !{x}: {x}",
            "AF (AF !{x}: {x})",
        ];

        for formula in invalid_formulae {
            assert!(parse_hctl_formula(formula).is_err());
        }
    }
}
