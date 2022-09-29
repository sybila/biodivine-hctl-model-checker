use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

use crate::operation_enums::*;

/// Enum of all possible tokens occurring in a HCTL formula string
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Token {
    Unary(UnaryOp),           // Unary operators: '~','EX','AX','EF','AF','EG','AG'
    Binary(BinaryOp),         // Binary operators: '&','|','^','=>','<=>','EU','AU','EW','AW'
    Hybrid(HybridOp, String), // Hybrid operator and its variable: '!', '@', '3'
    Atom(Atomic),             // Proposition, variable, or 'true'/'false' constant
    Tokens(Vec<Token>),       // A block of tokens inside parentheses
}

/// Tries to tokenize_formula HCTL formula
/// Wrapper for the recursive tokenize_formula function
pub fn tokenize_formula(formula: String) -> Result<Vec<Token>, String> {
    tokenize_recursive(&mut formula.chars().peekable(), true)
}

/// Process a peekable iterator of characters into a vector of `Token`s.
/// Tries to tokenize_formula HCTL formula
fn tokenize_recursive(
    input_chars: &mut Peekable<Chars>,
    top_level: bool,
) -> Result<Vec<Token>, String> {
    let mut output = Vec::new();

    while let Some(c) = input_chars.next() {
        //print!("{}", c);
        //io::stdout().flush().unwrap();

        match c {
            c if c.is_whitespace() => {} // skip whitespace
            '~' => output.push(Token::Unary(UnaryOp::Not)),
            '&' => output.push(Token::Binary(BinaryOp::And)),
            '|' => output.push(Token::Binary(BinaryOp::Or)),
            '^' => output.push(Token::Binary(BinaryOp::Xor)),
            '=' => {
                if Some('>') == input_chars.next() {
                    output.push(Token::Binary(BinaryOp::Imp));
                } else {
                    return Err("Expected '>' after '='.".to_string());
                }
            }
            '<' => {
                if Some('=') == input_chars.next() {
                    if Some('>') == input_chars.next() {
                        output.push(Token::Binary(BinaryOp::Iff));
                    } else {
                        return Err("Expected '>' after '<='.".to_string());
                    }
                } else {
                    return Err("Expected '=' after '<'.".to_string());
                }
            }
            // '>' is invalid as a start of a token
            '>' => return Err("Unexpected '>'.".to_string()),

            // pattern E{temporal}, must not be just a part of some proposition name
            'E' if is_valid_temp_op(input_chars.peek()) => {
                if let Some(c2) = input_chars.next() {
                    // check that it is not just a part of some proposition name
                    if let Some(c3) = input_chars.peek() {
                        if is_valid_in_name(*c3) {
                            let name = collect_name(input_chars)?;
                            output.push(Token::Atom(Atomic::Prop(
                                c.to_string() + c2.to_string().as_str() + &name,
                            )));
                            continue;
                        }
                    }

                    match c2 {
                        'X' => output.push(Token::Unary(UnaryOp::Ex)),
                        'F' => output.push(Token::Unary(UnaryOp::Ef)),
                        'G' => output.push(Token::Unary(UnaryOp::Eg)),
                        'U' => output.push(Token::Binary(BinaryOp::Eu)),
                        'W' => output.push(Token::Binary(BinaryOp::Ew)),
                        _ => return Err(format!("Unexpected char '{}' after 'E'.", c2)),
                    }
                } else {
                    return Err("Expected one of '{X,F,G,U,W}' after 'E'.".to_string());
                }
            }

            // pattern A{temporal}, must not be just a part of some proposition name
            'A' if is_valid_temp_op(input_chars.peek()) => {
                if let Some(c2) = input_chars.next() {
                    // check that it is not just a part of some proposition name
                    if let Some(c3) = input_chars.peek() {
                        if is_valid_in_name(*c3) {
                            let name = collect_name(input_chars)?;
                            output.push(Token::Atom(Atomic::Prop(
                                c.to_string() + c2.to_string().as_str() + &name,
                            )));
                            continue;
                        }
                    }
                    match c2 {
                        'X' => output.push(Token::Unary(UnaryOp::Ax)),
                        'F' => output.push(Token::Unary(UnaryOp::Af)),
                        'G' => output.push(Token::Unary(UnaryOp::Ag)),
                        'U' => output.push(Token::Binary(BinaryOp::Au)),
                        'W' => output.push(Token::Binary(BinaryOp::Aw)),
                        _ => return Err(format!("Unexpected char '{}' after 'A'.", c2)),
                    }
                } else {
                    return Err("Expected one of '{X,F,G,U,W}' after 'A'.".to_string());
                }
            }
            '!' => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '!')?;
                output.push(Token::Hybrid(HybridOp::Bind, name));
            }
            '@' => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '@')?;
                output.push(Token::Hybrid(HybridOp::Jump, name));
            }
            // "3" can be either exist quantifier or part of some proposition
            '3' if !is_valid_in_name_optional(input_chars.peek()) => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '3')?;
                output.push(Token::Hybrid(HybridOp::Exist, name));
            }
            ')' => {
                return if !top_level {
                    Ok(output)
                } else {
                    Err("Unexpected ')'.".to_string())
                }
            }
            '(' => {
                // start a nested token group
                let token_group = tokenize_recursive(input_chars, false)?;
                output.push(Token::Tokens(token_group));
            }
            // variable name
            '{' => {
                let name = collect_name(input_chars)?;
                if name.is_empty() {
                    return Err("Variable name can't be empty.".to_string());
                }
                output.push(Token::Atom(Atomic::Var(name)));
                if Some('}') != input_chars.next() {
                    return Err("Expected '}'.".to_string());
                }
            }
            // proposition name
            c if is_valid_in_name(c) => {
                let name = collect_name(input_chars)?;
                output.push(Token::Atom(Atomic::Prop(c.to_string() + &name)));
            }
            _ => return Err(format!("Unexpected char '{}'.", c)),
        }
    }

    if top_level {
        Ok(output)
    } else {
        Err("Expected ')'.".to_string())
    }
}

/// Check if given char can appear in a name.
fn is_valid_in_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Check if given char can appear in a name.
fn is_valid_in_name_optional(option_char: Option<&char>) -> bool {
    if let Some(c) = option_char {
        return is_valid_in_name(c.clone());
    }
    false
}

/// Check if given optional char represents valid temporal operator
fn is_valid_temp_op(option_char: Option<&char>) -> bool {
    if let Some(c) = option_char {
        return match c {
            'X' | 'F' | 'G' | 'U' | 'W' => true,
            _ => false,
        };
    }
    false
}

/// Retrieves the proposition (or variable) name from the input
/// The first character of the name might or might not be already consumed by the caller
fn collect_name(input_chars: &mut Peekable<Chars>) -> Result<String, String> {
    let mut name = Vec::new();
    while let Some(c) = input_chars.peek() {
        if c.is_whitespace() || !is_valid_in_name(*c) {
            break;
        } else {
            name.push(*c);
            input_chars.next(); // advance iterator
        }
    }
    Ok(name.into_iter().collect())
}

/// Retrieves the name of variable bound by a hybrid operator
/// Operator character is consumed by caller and is given as input for error msg purposes
fn collect_var_from_operator(
    input_chars: &mut Peekable<Chars>,
    operator: char,
) -> Result<String, String> {
    // there might be few spaces first
    let _ = input_chars.take_while(|c| c.is_whitespace());
    // now collect the variable name itself- it is in the form {var_name} for now
    if Some('{') != input_chars.next() {
        return Err(format!("Expected '{{' after '{}'.", operator));
    }
    let name = collect_name(input_chars)?;
    if name.is_empty() {
        return Err("Variable name can't be empty.".to_string());
    }

    if Some('}') != input_chars.next() {
        return Err("Expected '}'.".to_string());
    }
    let _ = input_chars.take_while(|c| c.is_whitespace());

    if Some(':') != input_chars.next() {
        return Err("Expected ':'.".to_string());
    }
    Ok(name)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Unary(UnaryOp::Not) => write!(f, "~"),
            Token::Binary(BinaryOp::And) => write!(f, "&"),
            Token::Binary(BinaryOp::Or) => write!(f, "|"),
            Token::Binary(BinaryOp::Xor) => write!(f, "^"),
            Token::Binary(BinaryOp::Imp) => write!(f, "=>"),
            Token::Binary(BinaryOp::Iff) => write!(f, "<=>"),
            Token::Unary(c) => write!(f, "{}", format!("{:?}", c)),
            Token::Binary(c) => write!(f, "{}", format!("{:?}", c)),
            Token::Hybrid(op, var) => write!(f, "{}", format!("{:?} {{{}}}:", op, var)),
            Token::Atom(Atomic::Prop(name)) => write!(f, "{}", name),
            Token::Atom(Atomic::Var(name)) => write!(f, "{{{}}}", name),
            Token::Atom(constant) => write!(f, "{:?}", constant),
            _ => write!(f, "( TOKENS )"),
        }
    }
}

#[allow(dead_code)]
fn print_tokens_recursively(tokens: &Vec<Token>) -> () {
    for token in tokens {
        match token {
            Token::Tokens(token_vec) => print_tokens_recursively(token_vec),
            _ => print!("{} ", token),
        }
    }
}

#[allow(dead_code)]
pub fn print_tokens(tokens: &Vec<Token>) -> () {
    print_tokens_recursively(tokens);
    println!();
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Token, tokenize_formula};
    use crate::operation_enums::*;

    #[test]
    fn test_tokenize_valid_formulae() {
        let valid1 = "!{x}: AG EF {x}".to_string();
        let tokens1_result = tokenize_formula(valid1);
        assert_eq!(tokens1_result.unwrap(), vec![
            Token::Hybrid(HybridOp::Bind, "x".to_string()),
            Token::Unary(UnaryOp::Ag),
            Token::Unary(UnaryOp::Ef),
            Token::Atom(Atomic::Var("x".to_string())),
        ]);

        let valid2 = "AF (!{x}: (AX (~{x} & AF {x})))".to_string();
        let tokens2_result = tokenize_formula(valid2);
        assert_eq!(tokens2_result.unwrap(), vec![
            Token::Unary(UnaryOp::Af),
            Token::Tokens(vec![
                Token::Hybrid(HybridOp::Bind, "x".to_string()),
                Token::Tokens(vec![
                    Token::Unary(UnaryOp::Ax),
                    Token::Tokens(vec![
                        Token::Unary(UnaryOp::Not),
                        Token::Atom(Atomic::Var("x".to_string())),
                        Token::Binary(BinaryOp::And),
                        Token::Unary(UnaryOp::Af),
                        Token::Atom(Atomic::Var("x".to_string())),
                    ]),
                ]),
            ]),
        ]);

        let valid3 = "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string();
        let tokens3_result = tokenize_formula(valid3);
        assert_eq!(tokens3_result.unwrap(), vec![
            Token::Hybrid(HybridOp::Bind, "x".to_string()),
            Token::Hybrid(HybridOp::Exist, "y".to_string()),
            Token::Tokens(vec![
                Token::Hybrid(HybridOp::Jump, "x".to_string()),
                Token::Unary(UnaryOp::Not),
                Token::Atom(Atomic::Var("y".to_string())),
                Token::Binary(BinaryOp::And),
                Token::Unary(UnaryOp::Ax),
                Token::Atom(Atomic::Var("x".to_string())),
            ]),
            Token::Binary(BinaryOp::And),
            Token::Tokens(vec![
                Token::Hybrid(HybridOp::Jump, "y".to_string()),
                Token::Unary(UnaryOp::Ax),
                Token::Atom(Atomic::Var("y".to_string())),
            ]),
        ]);
    }

    #[test]
    fn test_tokenize_invalid_formulae() {
        let invalid_formulae = vec![
            "!{x}: AG EF {x<&}",
            "!{x AG EF {x}",
            "!{}: AG EF {x}",
            "{x}: AG EF {x}",
        ];

        for formula in invalid_formulae {
            assert!(tokenize_formula(formula.to_string()).is_err())
        }
    }
}
