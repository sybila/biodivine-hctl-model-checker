//! Contains functionality regarding parsing formula (or formula tokens) into a syntax tree.
//!
//! The operator precedence is following (the lower, the stronger):
//!  - unary operators (negation + temporal): 1
//!  - binary temporal operators: 2
//!  - boolean binary operators: and=3, xor=4, or=5, imp=6, equiv=7
//!  - hybrid operators: 8
//!

use crate::preprocessing::hctl_tree::*;
use crate::preprocessing::operator_enums::*;
use crate::preprocessing::tokenizer::{
    try_tokenize_extended_formula, try_tokenize_formula, HctlToken,
};
use crate::preprocessing::utils::validate_props_and_rename_vars;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicContext;

/// Parse an HCTL formula string representation into an actual formula tree.
/// Basically a wrapper for tokenize+parse (used often for testing/debug purposes).
///
/// NEEDS to call [validate_props_and_rename_vars] to fully finish the preprocessing step.
pub fn parse_hctl_formula(formula: &str) -> Result<HctlTreeNode, String> {
    let tokens = try_tokenize_formula(formula.to_string())?;
    let tree = parse_hctl_tokens(&tokens)?;
    Ok(tree)
}

/// Parse an extended HCTL formula string representation into an actual formula tree.
/// Extended formulae can include `wild-card propositions` in form "%proposition%".
///
/// NEEDS to call [validate_props_and_rename_vars] to fully finish the preprocessing step.
pub fn parse_extended_formula(formula: &str) -> Result<HctlTreeNode, String> {
    let tokens = try_tokenize_extended_formula(formula.to_string())?;
    let tree = parse_hctl_tokens(&tokens)?;
    Ok(tree)
}

/// Parse an HCTL formula string representation into an actual formula tree with renamed (minimized)
/// set of variables.
/// Basically a wrapper for the whole preprocessing step (tokenize + parse + rename vars).
pub fn parse_and_minimize_hctl_formula(
    symbolic_context: &SymbolicContext,
    formula: &str,
) -> Result<HctlTreeNode, String> {
    let tree = parse_hctl_formula(formula)?;
    let tree = validate_props_and_rename_vars(tree, symbolic_context)?;
    Ok(tree)
}

/// Parse an extended HCTL formula string representation into an actual formula tree
/// with renamed (minimized) set of variables.
/// Extended formulae can include `wild-card propositions` in form "%proposition%".
pub fn parse_and_minimize_extended_formula(
    symbolic_context: &SymbolicContext,
    formula: &str,
) -> Result<HctlTreeNode, String> {
    let tree = parse_extended_formula(formula)?;
    let tree = validate_props_and_rename_vars(tree, symbolic_context)?;
    Ok(tree)
}

/// Predicate for whether given token represents hybrid operator.
fn is_hybrid(token: &HctlToken) -> bool {
    matches!(token, HctlToken::Hybrid(..))
}

/// Predicate for whether given token represents temporal binary operator.
fn is_binary_temporal(token: &HctlToken) -> bool {
    matches!(
        token,
        HctlToken::Binary(BinaryOp::EU)
            | HctlToken::Binary(BinaryOp::AU)
            | HctlToken::Binary(BinaryOp::EW)
            | HctlToken::Binary(BinaryOp::AW)
    )
}

/// Predicate for whether given token represents unary operator.
fn is_unary(token: &HctlToken) -> bool {
    matches!(token, HctlToken::Unary(_))
}

/// Utility method to find the first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[HctlToken], token: HctlToken) -> Option<usize> {
    tokens.iter().position(|t| *t == token)
}

/// Utility method to find the first occurrence of a hybrid operator in the token tree.
fn index_of_first_hybrid(tokens: &[HctlToken]) -> Option<usize> {
    tokens.iter().position(is_hybrid)
}

/// Utility method to find the first occurrence of a binary temporal operator in the token tree.
fn index_of_first_binary_temp(tokens: &[HctlToken]) -> Option<usize> {
    tokens.iter().position(is_binary_temporal)
}

/// Utility method to find the first occurrence of an unary operator in the token tree.
fn index_of_first_unary(tokens: &[HctlToken]) -> Option<usize> {
    tokens.iter().position(is_unary)
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
        if i > 0 && !matches!(&tokens[i - 1], HctlToken::Hybrid(..)) {
            return Err(format!(
                "Hybrid operator can't be directly preceded by {}.",
                &tokens[i - 1]
            ));
        }
        match &tokens[i] {
            HctlToken::Hybrid(op, var, domain) => HctlTreeNode::mk_hybrid(
                parse_1_hybrid(&tokens[(i + 1)..])?,
                var.as_str(),
                domain.clone(),
                op.clone(),
            ),
            _ => unreachable!(), // we already made sure that this is indeed a hybrid token
        }
    } else {
        parse_2_iff(tokens)?
    })
}

/// Recursive parsing step 2: extract `<=>` operators.
fn parse_2_iff(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let iff_token = index_of_first(tokens, HctlToken::Binary(BinaryOp::Iff));
    Ok(if let Some(i) = iff_token {
        HctlTreeNode::mk_binary(
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
        HctlTreeNode::mk_binary(
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
        HctlTreeNode::mk_binary(
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
        HctlTreeNode::mk_binary(
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
        HctlTreeNode::mk_binary(
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
            HctlToken::Binary(op) => HctlTreeNode::mk_binary(
                parse_8_unary(&tokens[..i])?,
                parse_7_binary_temp(&tokens[(i + 1)..])?,
                op.clone(),
            ),
            _ => unreachable!(), // we already made sure that this is indeed a binary token
        }
    } else {
        parse_8_unary(tokens)?
    })
}

/// Recursive parsing step 8: extract unary temporal operators and negations.
fn parse_8_unary(tokens: &[HctlToken]) -> Result<HctlTreeNode, String> {
    let unary_token = index_of_first_unary(tokens);
    Ok(if let Some(i) = unary_token {
        // perform check that unary operator is not directly preceded by some atomic sub-formula
        if i > 0 && matches!(&tokens[i - 1], HctlToken::Atom(..)) {
            return Err(format!(
                "Unary operator can't be directly preceded by {}.",
                &tokens[i - 1]
            ));
        }

        match &tokens[i] {
            HctlToken::Unary(op) => {
                HctlTreeNode::mk_unary(parse_8_unary(&tokens[(i + 1)..])?, op.clone())
            }
            _ => unreachable!(), // we already made sure that this is indeed an unary token
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
                        Ok(HctlTreeNode::mk_constant(true))
                    } else if name == "false" || name == "False" || name == "0" {
                        Ok(HctlTreeNode::mk_constant(false))
                    } else {
                        Ok(HctlTreeNode::mk_proposition(name.as_str()))
                    }
                }
                HctlToken::Atom(Atomic::Var(name)) => {
                    return Ok(HctlTreeNode::mk_variable(name.as_str()))
                }
                HctlToken::Atom(Atomic::WildCardProp(name)) => {
                    return Ok(HctlTreeNode::mk_wild_card(name.as_str()))
                }
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
    use crate::preprocessing::hctl_tree::*;
    use crate::preprocessing::operator_enums::*;
    use crate::preprocessing::parser::{parse_extended_formula, parse_hctl_formula};

    #[test]
    /// Test whether several valid HCTL formulae are parsed without causing errors.
    /// Also check that the formula is saved correctly in the tree root.
    fn parse_valid_formulae() {
        let valid1 = "!{x}: AG EF {x}";
        let tree = parse_hctl_formula(valid1).unwrap();
        assert_eq!(tree.as_str(), "(!{x}: (AG (EF {x})))");

        let valid2 = "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})";
        let tree = parse_hctl_formula(valid2).unwrap();
        assert_eq!(
            tree.as_str(),
            "(!{x}: (3{y}: ((@{x}: ((~{y}) & (AX {x}))) & (@{y}: (AX {y})))))",
        );

        let valid3 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))";
        let tree = parse_hctl_formula(valid3).unwrap();
        assert_eq!(tree.as_str(), "(3{x}: (3{y}: ((@{x}: ((~{y}) & (AX {x}))) & ((@{y}: (AX {y})) & ((EF ({x} & (!{z}: (AX {z})))) & ((EF ({y} & (!{z}: (AX {z})))) & (AX ((EF ({x} & (!{z}: (AX {z})))) ^ (EF ({y} & (!{z}: (AX {z}))))))))))))");

        // also test propositions, constants, and other operators (and their parse order)
        // propositions names should not be modified, constants should be unified to True/False
        let valid4 = "(prop1 <=> PROP2 | false => 1) AU (True ^ 0)";
        let tree = parse_hctl_formula(valid4).unwrap();
        assert_eq!(
            tree.as_str(),
            "((prop1 <=> ((PROP2 | False) => True)) AU (True ^ False))"
        );

        // all formulae must be correctly parsed also using the extended version of HCTL
        assert!(parse_extended_formula(valid1).is_ok());
        assert!(parse_extended_formula(valid2).is_ok());
        assert!(parse_extended_formula(valid3).is_ok());
        assert!(parse_extended_formula(valid4).is_ok());
    }

    #[test]
    fn operator_priority() {
        assert_eq!(
            "(((((~a) ^ ((~b) & (~c))) | (~d)) => (~e)) <=> (~f))",
            parse_hctl_formula("~a ^ ~b & ~c | ~d => ~e <=> ~f")
                .unwrap()
                .as_str()
        )
    }

    #[test]
    fn operator_associativity() {
        assert_eq!(
            "(a & (b & c))",
            parse_hctl_formula("a & b & c").unwrap().as_str()
        );
        assert_eq!(
            "(a | (b | c))",
            parse_hctl_formula("a | b | c").unwrap().as_str()
        );
        assert_eq!(
            "(a ^ (b ^ c))",
            parse_hctl_formula("a ^ b ^ c").unwrap().as_str()
        );
        assert_eq!(
            "(a => (b => c))",
            parse_hctl_formula("a => b => c").unwrap().as_str()
        );
        assert_eq!(
            "(a <=> (b <=> c))",
            parse_hctl_formula("a <=> b <=> c").unwrap().as_str()
        );
    }

    #[test]
    /// Test parsing of several valid HCTL formulae against expected results.
    fn compare_parser_with_expected() {
        let formula = "(false & p1)";
        let expected_tree = HctlTreeNode::mk_binary(
            HctlTreeNode::mk_constant(false),
            HctlTreeNode::mk_proposition("p1"),
            BinaryOp::And,
        );
        assert_eq!(parse_hctl_formula(formula).unwrap(), expected_tree);

        let formula = "!{x}: (AX {x})";
        let expected_tree = HctlTreeNode::mk_hybrid(
            HctlTreeNode::mk_unary(HctlTreeNode::mk_variable("x"), UnaryOp::AX),
            "x",
            None,
            HybridOp::Bind,
        );
        assert_eq!(parse_hctl_formula(formula).unwrap(), expected_tree);
    }

    #[test]
    /// Test parsing of several completely invalid HCTL formulae.
    fn parse_invalid_formulae() {
        let invalid_formulae = vec![
            "!{x}: AG EK {x}",
            "!{x}: p q",
            "!{x}: p q AG {x}",
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
            // all formulae are invalid regardless if we allow for wild-card propositions
            assert!(parse_extended_formula(formula).is_err());
        }
    }

    #[test]
    /// Test parsing several extended HCTL formulae.
    fn parse_extended_formulae() {
        let formula = "(!{x}: AG EF {x}) & %p%";
        // parser for standard HCTL should fail, for extended succeed
        assert!(parse_hctl_formula(formula).is_err());
        assert!(parse_extended_formula(formula).is_ok());
        let tree = parse_extended_formula(formula).unwrap();
        assert_eq!(tree.as_str(), "((!{x}: (AG (EF {x}))) & %p%)");

        let formula = "!{x}: 3{y}: (@{x}: ~{y} & %s%) & (@{y}: %s%)";
        assert!(parse_hctl_formula(formula).is_err());
        assert!(parse_extended_formula(formula).is_ok());
        let tree = parse_extended_formula(formula).unwrap();
        assert_eq!(
            tree.as_str(),
            "(!{x}: (3{y}: ((@{x}: ((~{y}) & %s%)) & (@{y}: %s%))))"
        );
    }
}
