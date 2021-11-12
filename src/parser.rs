use crate::operation_enums::*;
use crate::tokenizer::Token;

use std::cmp::Ordering;
use std::fmt;
use std::cmp;


/// Enum of possible node types in a HCTL formula tree
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum NodeType {
    TerminalNode(Atomic),
    UnaryNode(UnaryOp, Box<Node>),
    BinaryNode(BinaryOp, Box<Node>, Box<Node>),
    HybridNode(HybridOp, String, Box<Node>),
}

/// Node structure for HCTL formula parse tree
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
pub struct Node {
    pub subform_str: String,
    pub height: i32,
    pub node_type: NodeType,
}

/// Nodes are ordered by their height
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.height.cmp(&other.height)
    }
}


impl Node {
    /// Create default node - True terminal node
    fn new() -> Self {
        Self{
            subform_str: "True".to_string(),
            height: 0,
            node_type: NodeType::TerminalNode(Atomic::True),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.subform_str)
    }
}

/// Create hybrid node from given arguments
fn create_hybrid(child: Box<Node>, var: String, op: HybridOp) -> Node {
    Node {
        subform_str: format!("({} {{{}}}: {})", op, var, child.subform_str),
        height: child.height + 1,
        node_type: NodeType::HybridNode(
            op.clone(),
            var.clone(),
            child,
        )
    }
}

/// Create unary node from given arguments
fn create_unary(child: Box<Node>, op: UnaryOp) -> Node {
    Node {
        subform_str: format!("({} {})", op, child.subform_str),
        height: child.height + 1,
        node_type: NodeType::UnaryNode(
            op.clone(),
            child,
        )
    }
}

/// Create binary node from given arguments
fn create_binary(left: Box<Node>, right: Box<Node>, op: BinaryOp) -> Node {
    Node {
        subform_str: format!("({} {} {})", left.subform_str, op, right.subform_str),
        height: cmp::max(left.height, right.height) + 1,
        node_type: NodeType::BinaryNode(
            op.clone(),
            left,
            right)
    }
}

/// Predicate whether given token represents hybrid operator
fn is_hybrid(token: &Token) -> bool {
    match token {
        Token::Hybrid(_, _) => true,
        _ => false,
    }
}

/// Predicate whether given token represents temporal binary operator
fn is_binary_temporal(token: &Token) -> bool {
    match token {
        Token::Binary(BinaryOp::Eu) => true,
        Token::Binary(BinaryOp::Au) => true,
        Token::Binary(BinaryOp::Ew) => true,
        Token::Binary(BinaryOp::Aw) => true,
        _ => false,
    }
}

/// Predicate whether given token represents unary operator
fn is_unary(token: &Token) -> bool {
    match token {
        Token::Unary(_) => true,
        _ => false,
    }
}

/// Utility method to find first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[Token], token: Token) -> Option<usize> {
    return tokens.iter().position(|t| *t == token);
}

/// Utility method to find first occurrence of hybrid operator in the token tree.
fn index_of_first_hybrid(tokens: &[Token]) -> Option<usize> {
    return tokens.iter().position(|token| is_hybrid(token));
}

/// Utility method to find first occurrence of binary temporal operator in the token tree.
fn index_of_first_binary_temp(tokens: &[Token]) -> Option<usize> {
    return tokens.iter().position(|token| is_binary_temporal(token));
}

/// Utility method to find first occurrence of unary operator in the token tree.
fn index_of_first_unary(tokens: &[Token]) -> Option<usize> {
    return tokens.iter().position(|token| is_unary(token));
}

/**
 * PRIORITY OF OPERATORS
 * unary operators (not + temporal): 1
 * temporal binary operators: 2
 * boolean binary operators: and=3, xor=4, or=5, imp=6, eq=7
 * hybrid operators: 8
 */

/// Parse a `Node` using the recursive steps.
pub fn parse_update_function(tokens: &[Token]) -> Result<Box<Node>, String> {
    parse_1_hybrid(tokens)
}

/// Recursive parsing step 1: extract hybrid operators.
fn parse_1_hybrid(tokens: &[Token]) -> Result<Box<Node>, String> {
    let hybrid_token = index_of_first_hybrid(tokens);
    Ok(if let Some(i) = hybrid_token {
        match &tokens[i] {
            Token::Hybrid(op, var) => Box::new(create_hybrid(
                parse_1_hybrid(&tokens[(i + 1)..])?,
                var.clone(),
                op.clone())
            ),
            _ => Box::new(Node::new()) // This branch cant happen
        }
    } else {
        parse_2_iff(tokens)?
    })
}

/// Recursive parsing step 2: extract `<=>` operators.
fn parse_2_iff(tokens: &[Token]) -> Result<Box<Node>, String> {
    let iff_token = index_of_first(tokens, Token::Binary(BinaryOp::Iff));
    Ok(if let Some(i) = iff_token {
        Box::new(create_binary(
            parse_3_imp(&tokens[..i])?,
            parse_2_iff(&tokens[(i + 1)..])?,
            BinaryOp::Iff,)
        )
    } else {
        parse_3_imp(tokens)?
    })
}

/// Recursive parsing step 3: extract `=>` operators.
fn parse_3_imp(tokens: &[Token]) -> Result<Box<Node>, String> {
    let imp_token = index_of_first(tokens, Token::Binary(BinaryOp::Imp));
    Ok(if let Some(i) = imp_token {
        Box::new(create_binary(
            parse_4_or(&tokens[..i])?,
            parse_3_imp(&tokens[(i + 1)..])?,
            BinaryOp::Imp,)
        )
    } else {
        parse_4_or(tokens)?
    })
}

/// Recursive parsing step 4: extract `|` operators.
fn parse_4_or(tokens: &[Token]) -> Result<Box<Node>, String> {
    let or_token = index_of_first(tokens, Token::Binary(BinaryOp::Or));
    Ok(if let Some(i) = or_token {
        Box::new(create_binary(
            parse_5_xor(&tokens[..i])?,
            parse_4_or(&tokens[(i + 1)..])?,
            BinaryOp::Or,)
        )
    } else {
        parse_5_xor(tokens)?
    })
}

/// Recursive parsing step 5: extract `^` operators.
fn parse_5_xor(tokens: &[Token]) -> Result<Box<Node>, String> {
    let xor_token = index_of_first(tokens, Token::Binary(BinaryOp::Xor));
    Ok(if let Some(i) = xor_token {
        Box::new(create_binary(
            parse_6_and(&tokens[..i])?,
            parse_5_xor(&tokens[(i + 1)..])?,
            BinaryOp::Xor,)
        )

    } else {
        parse_6_and(tokens)?
    })
}

/// Recursive parsing step 6: extract `&` operators.
fn parse_6_and(tokens: &[Token]) -> Result<Box<Node>, String> {
    let and_token = index_of_first(tokens, Token::Binary(BinaryOp::And));
    Ok(if let Some(i) = and_token {
        Box::new(create_binary(
            parse_7_binary_temp(&tokens[..i])?,
            parse_6_and(&tokens[(i + 1)..])?,
            BinaryOp::And,)
        )
    } else {
        parse_7_binary_temp(tokens)?
    })
}

/// Recursive parsing step 7: extract binary temporal operators.
fn parse_7_binary_temp(tokens: &[Token]) -> Result<Box<Node>, String> {
    let binary_token = index_of_first_binary_temp(tokens);
    Ok(if let Some(i) = binary_token {
        match &tokens[i] {
            Token::Binary(op) => Box::new(create_binary(
                parse_8_unary(&tokens[..i])?,
                parse_7_binary_temp(&tokens[(i + 1)..])?,
                op.clone(),)
            ),
            _ => Box::new(Node::new()) // This branch cant happen
        }
    } else {
        parse_8_unary(tokens)?
    })
}

/// Recursive parsing step 8: extract unary temporal operators.
fn parse_8_unary(tokens: &[Token]) -> Result<Box<Node>, String> {
    let unary_token = index_of_first_unary(tokens);
    Ok(if let Some(i) = unary_token {
        match &tokens[i] {
            Token::Unary(op) => Box::new(create_unary(
                parse_8_unary(&tokens[(i + 1)..])?,
                op.clone())
            ),
            _ => Box::new(Node::new()) // This branch cant happen
        }
    } else {
        parse_9_terminal(tokens)?
    })
}

/// Recursive parsing step 9: extract terminals.
fn parse_9_terminal(tokens: &[Token]) -> Result<Box<Node>, String> {
    if tokens.is_empty() {
        Err("Expected formula, found nothing.".to_string())
    } else {
        if tokens.len() == 1 {
            // This should be name (var/prop) or a parenthesis group, everything else does not make sense.
            match &tokens[0] {
                Token::Atom(Atomic::Prop(name)) => {
                    return if name == "true" {
                        Ok(Box::new(Node {
                            subform_str: "True".to_string(),
                            height: 0,
                            node_type: NodeType::TerminalNode(Atomic::True),
                        }))
                    } else if name == "false" {
                        Ok(Box::new(Node {
                            subform_str: "False".to_string(),
                            height: 0,
                            node_type: NodeType::TerminalNode(Atomic::False),
                        }))
                    } else {
                        Ok(Box::new(Node {
                            subform_str: name.clone(),
                            height: 0,
                            node_type: NodeType::TerminalNode(Atomic::Prop(name.clone())),
                        }))
                    }
                }
                Token::Atom(Atomic::Var(name)) => return Ok(Box::new(Node {
                    subform_str: format!("{{{}}}", name),
                    height: 0,
                    node_type: NodeType::TerminalNode(Atomic::Var(name.clone())),
                })),
                Token::Tokens(inner) => return parse_update_function(inner),
                _ => {} // otherwise, fall through to the error at the end.
            }
        }
        Err(format!("Unexpected: {:?}. Expecting formula.", tokens))
    }
}
