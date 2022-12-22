//! Contains functionality regarding parsing formula tokens into a syntax tree.
//!
//! The operator precedence is following (the lower, the stronger):
//!  - unary operators (negation + temporal): 1
//!  - binary temporal operators: 2
//!  - boolean binary operators: and=3, xor=4, or=5, imp=6, equiv=7
//!  - hybrid operators: 8
//!

use crate::formula_preprocessing::node::*;
use crate::formula_preprocessing::operator_enums::*;
use crate::formula_preprocessing::tokenizer::Token;

/// Predicate for whether given token represents hybrid operator.
fn is_hybrid(token: &Token) -> bool {
    matches!(token, Token::Hybrid(_, _))
}

/// Predicate for whether given token represents temporal binary operator.
fn is_binary_temporal(token: &Token) -> bool {
    matches!(
        token,
        Token::Binary(BinaryOp::Eu)
            | Token::Binary(BinaryOp::Au)
            | Token::Binary(BinaryOp::Ew)
            | Token::Binary(BinaryOp::Aw)
    )
}

/// Predicate for whether given token represents unary operator.
fn is_unary(token: &Token) -> bool {
    matches!(token, Token::Unary(_))
}

/// Utility method to find the first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[Token], token: Token) -> Option<usize> {
    return tokens.iter().position(|t| *t == token);
}

/// Utility method to find the first occurrence of a hybrid operator in the token tree.
fn index_of_first_hybrid(tokens: &[Token]) -> Option<usize> {
    return tokens.iter().position(is_hybrid);
}

/// Utility method to find the first occurrence of a binary temporal operator in the token tree.
fn index_of_first_binary_temp(tokens: &[Token]) -> Option<usize> {
    return tokens.iter().position(is_binary_temporal);
}

/// Utility method to find the first occurrence of an unary operator in the token tree.
fn index_of_first_unary(tokens: &[Token]) -> Option<usize> {
    return tokens.iter().position(is_unary);
}

/// Parse `tokens` in to a syntax tree using recursive steps.
pub fn parse_hctl_formula(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    parse_1_hybrid(tokens)
}

/// Recursive parsing step 1: extract hybrid operators.
/// Hybrid operator must not be immediately preceded by any other kind of operator.
/// We only allow it to be preceded by another hybrid operator, or parentheses must be used.
/// (things like "AF !{x}: ..." are forbidden, must be written in brackets as "AF (!{x}: ...)"
fn parse_1_hybrid(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let hybrid_token = index_of_first_hybrid(tokens);
    Ok(if let Some(i) = hybrid_token {
        // perform check that hybrid operator is not preceded by other type of operators
        if i > 0 && !matches!(&tokens[i - 1], Token::Hybrid(_, _)) {
            return Err(format!(
                "Hybrid operator can't be directly preceded by {}.",
                &tokens[i - 1]
            ));
        }
        match &tokens[i] {
            Token::Hybrid(op, var) => Box::new(create_hybrid(
                parse_1_hybrid(&tokens[(i + 1)..])?,
                var.clone(),
                op.clone(),
            )),
            _ => Box::new(HctlTreeNode::new()), // This branch cant happen, but must result in same type
        }
    } else {
        parse_2_iff(tokens)?
    })
}

/// Recursive parsing step 2: extract `<=>` operators.
fn parse_2_iff(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let iff_token = index_of_first(tokens, Token::Binary(BinaryOp::Iff));
    Ok(if let Some(i) = iff_token {
        Box::new(create_binary(
            parse_3_imp(&tokens[..i])?,
            parse_2_iff(&tokens[(i + 1)..])?,
            BinaryOp::Iff,
        ))
    } else {
        parse_3_imp(tokens)?
    })
}

/// Recursive parsing step 3: extract `=>` operators.
fn parse_3_imp(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let imp_token = index_of_first(tokens, Token::Binary(BinaryOp::Imp));
    Ok(if let Some(i) = imp_token {
        Box::new(create_binary(
            parse_4_or(&tokens[..i])?,
            parse_3_imp(&tokens[(i + 1)..])?,
            BinaryOp::Imp,
        ))
    } else {
        parse_4_or(tokens)?
    })
}

/// Recursive parsing step 4: extract `|` operators.
fn parse_4_or(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let or_token = index_of_first(tokens, Token::Binary(BinaryOp::Or));
    Ok(if let Some(i) = or_token {
        Box::new(create_binary(
            parse_5_xor(&tokens[..i])?,
            parse_4_or(&tokens[(i + 1)..])?,
            BinaryOp::Or,
        ))
    } else {
        parse_5_xor(tokens)?
    })
}

/// Recursive parsing step 5: extract `^` operators.
fn parse_5_xor(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let xor_token = index_of_first(tokens, Token::Binary(BinaryOp::Xor));
    Ok(if let Some(i) = xor_token {
        Box::new(create_binary(
            parse_6_and(&tokens[..i])?,
            parse_5_xor(&tokens[(i + 1)..])?,
            BinaryOp::Xor,
        ))
    } else {
        parse_6_and(tokens)?
    })
}

/// Recursive parsing step 6: extract `&` operators.
fn parse_6_and(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let and_token = index_of_first(tokens, Token::Binary(BinaryOp::And));
    Ok(if let Some(i) = and_token {
        Box::new(create_binary(
            parse_7_binary_temp(&tokens[..i])?,
            parse_6_and(&tokens[(i + 1)..])?,
            BinaryOp::And,
        ))
    } else {
        parse_7_binary_temp(tokens)?
    })
}

/// Recursive parsing step 7: extract binary temporal operators.
fn parse_7_binary_temp(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let binary_token = index_of_first_binary_temp(tokens);
    Ok(if let Some(i) = binary_token {
        match &tokens[i] {
            Token::Binary(op) => Box::new(create_binary(
                parse_8_unary(&tokens[..i])?,
                parse_7_binary_temp(&tokens[(i + 1)..])?,
                op.clone(),
            )),
            _ => Box::new(HctlTreeNode::new()), // This branch cant happen, but must result in same type
        }
    } else {
        parse_8_unary(tokens)?
    })
}

/// Recursive parsing step 8: extract unary temporal operators and negations.
fn parse_8_unary(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    let unary_token = index_of_first_unary(tokens);
    Ok(if let Some(i) = unary_token {
        match &tokens[i] {
            Token::Unary(op) => {
                Box::new(create_unary(parse_8_unary(&tokens[(i + 1)..])?, op.clone()))
            }
            _ => Box::new(HctlTreeNode::new()), // This branch cant happen, but must result in same type
        }
    } else {
        parse_9_terminal_and_parentheses(tokens)?
    })
}

/// Recursive parsing step 9: extract terminals and recursively solve sub-formulae in parentheses.
fn parse_9_terminal_and_parentheses(tokens: &[Token]) -> Result<Box<HctlTreeNode>, String> {
    if tokens.is_empty() {
        Err("Expected formula, found nothing.".to_string())
    } else {
        if tokens.len() == 1 {
            // This should be name (var/prop) or a parenthesis group, everything else does not make sense.
            match &tokens[0] {
                Token::Atom(Atomic::Prop(name)) => {
                    return if name == "true" {
                        Ok(Box::new(HctlTreeNode {
                            subform_str: "True".to_string(),
                            height: 0,
                            node_type: NodeType::TerminalNode(Atomic::True),
                        }))
                    } else if name == "false" {
                        Ok(Box::new(HctlTreeNode {
                            subform_str: "False".to_string(),
                            height: 0,
                            node_type: NodeType::TerminalNode(Atomic::False),
                        }))
                    } else {
                        Ok(Box::new(HctlTreeNode {
                            subform_str: name.clone(),
                            height: 0,
                            node_type: NodeType::TerminalNode(Atomic::Prop(name.clone())),
                        }))
                    }
                }
                Token::Atom(Atomic::Var(name)) => {
                    return Ok(Box::new(HctlTreeNode {
                        subform_str: format!("{{{}}}", name),
                        height: 0,
                        node_type: NodeType::TerminalNode(Atomic::Var(name.clone())),
                    }))
                }
                // recursively solve sub-formulae in parentheses
                Token::Tokens(inner) => return parse_hctl_formula(inner),
                _ => {} // otherwise, fall through to the error at the end.
            }
        }
        Err(format!("Unexpected: {:?}. Expecting formula.", tokens))
    }
}

#[cfg(test)]
mod tests {
    use crate::formula_preprocessing::parser::parse_hctl_formula;
    use crate::formula_preprocessing::tokenizer::try_tokenize_formula;

    #[test]
    /// Test parsing of several valid HCTL formulae.
    fn test_parse_valid_formulae() {
        let valid1 = "!{x}: AG EF {x}".to_string();
        let tokens1 = try_tokenize_formula(valid1).unwrap();
        assert!(parse_hctl_formula(&tokens1).is_ok());

        let valid2 = "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string();
        let tokens2 = try_tokenize_formula(valid2).unwrap();
        assert!(parse_hctl_formula(&tokens2).is_ok());

        let valid3 = "3{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y}) & EF ({x} & (!{z}: AX {z})) & EF ({y} & (!{z}: AX {z})) & AX (EF ({x} & (!{z}: AX {z})) ^ EF ({y} & (!{z}: AX {z})))".to_string();
        let tokens3 = try_tokenize_formula(valid3).unwrap();
        assert!(parse_hctl_formula(&tokens3).is_ok());
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
            let tokens = try_tokenize_formula(formula.to_string()).unwrap();
            assert!(parse_hctl_formula(&tokens).is_err())
        }
    }
}
