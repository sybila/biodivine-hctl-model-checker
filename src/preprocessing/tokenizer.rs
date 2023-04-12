//! Contains functionality regarding the tokenizing of HCTL formula string.

use crate::preprocessing::operator_enums::*;

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// Enum of all possible tokens occurring in a HCTL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HctlToken {
    Unary(UnaryOp),           // Unary operators: '~','EX','AX','EF','AF','EG','AG'
    Binary(BinaryOp),         // Binary operators: '&','|','^','=>','<=>','EU','AU','EW','AW'
    Hybrid(HybridOp, String), // Hybrid operator and its variable: '!', '@', '3', 'V'
    Atom(Atomic),             // Proposition, variable, or 'true'/'false' constant
    Tokens(Vec<HctlToken>),   // A block of tokens inside parentheses
}

/// Try to tokenize given HCTL formula string.
/// Wrapper for the recursive `try_tokenize_formula` function.
pub fn try_tokenize_formula(formula: String) -> Result<Vec<HctlToken>, String> {
    try_tokenize_recursive(&mut formula.chars().peekable(), true)
}

/// Process a peekable iterator of characters into a vector of `HctlToken`s.
fn try_tokenize_recursive(
    input_chars: &mut Peekable<Chars>,
    top_level: bool,
) -> Result<Vec<HctlToken>, String> {
    let mut output = Vec::new();

    while let Some(c) = input_chars.next() {
        //print!("{}", c);
        //io::stdout().flush().unwrap();

        match c {
            c if c.is_whitespace() => {} // skip whitespace
            '~' => output.push(HctlToken::Unary(UnaryOp::Not)),
            '&' => output.push(HctlToken::Binary(BinaryOp::And)),
            '|' => output.push(HctlToken::Binary(BinaryOp::Or)),
            '^' => output.push(HctlToken::Binary(BinaryOp::Xor)),
            '=' => {
                if Some('>') == input_chars.next() {
                    output.push(HctlToken::Binary(BinaryOp::Imp));
                } else {
                    return Err("Expected '>' after '='.".to_string());
                }
            }
            '<' => {
                if Some('=') == input_chars.next() {
                    if Some('>') == input_chars.next() {
                        output.push(HctlToken::Binary(BinaryOp::Iff));
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
                            output.push(HctlToken::Atom(Atomic::Prop(
                                c.to_string() + c2.to_string().as_str() + &name,
                            )));
                            continue;
                        }
                    }

                    match c2 {
                        'X' => output.push(HctlToken::Unary(UnaryOp::Ex)),
                        'F' => output.push(HctlToken::Unary(UnaryOp::Ef)),
                        'G' => output.push(HctlToken::Unary(UnaryOp::Eg)),
                        'U' => output.push(HctlToken::Binary(BinaryOp::Eu)),
                        'W' => output.push(HctlToken::Binary(BinaryOp::Ew)),
                        _ => return Err(format!("Unexpected char '{c2}' after 'E'.")),
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
                            output.push(HctlToken::Atom(Atomic::Prop(
                                c.to_string() + c2.to_string().as_str() + &name,
                            )));
                            continue;
                        }
                    }
                    match c2 {
                        'X' => output.push(HctlToken::Unary(UnaryOp::Ax)),
                        'F' => output.push(HctlToken::Unary(UnaryOp::Af)),
                        'G' => output.push(HctlToken::Unary(UnaryOp::Ag)),
                        'U' => output.push(HctlToken::Binary(BinaryOp::Au)),
                        'W' => output.push(HctlToken::Binary(BinaryOp::Aw)),
                        _ => return Err(format!("Unexpected char '{c2}' after 'A'.")),
                    }
                } else {
                    return Err("Expected one of '{X,F,G,U,W}' after 'A'.".to_string());
                }
            }
            '!' => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '!')?;
                output.push(HctlToken::Hybrid(HybridOp::Bind, name));
            }
            '@' => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '@')?;
                output.push(HctlToken::Hybrid(HybridOp::Jump, name));
            }
            // "3" can be either exist quantifier or part of some proposition
            '3' if !is_valid_in_name_optional(input_chars.peek()) => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '3')?;
                output.push(HctlToken::Hybrid(HybridOp::Exists, name));
            }
            // "V" can be either forall quantifier or part of some proposition
            'V' if !is_valid_in_name_optional(input_chars.peek()) => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, 'V')?;
                output.push(HctlToken::Hybrid(HybridOp::Forall, name));
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
                let token_group = try_tokenize_recursive(input_chars, false)?;
                output.push(HctlToken::Tokens(token_group));
            }
            // variable name
            '{' => {
                let name = collect_name(input_chars)?;
                if name.is_empty() {
                    return Err("Variable name can't be empty.".to_string());
                }
                output.push(HctlToken::Atom(Atomic::Var(name)));
                if Some('}') != input_chars.next() {
                    return Err("Expected '}'.".to_string());
                }
            }
            // proposition name or constant
            // these 2 are NOT distinguished now but later during parsing
            c if is_valid_in_name(c) => {
                let name = collect_name(input_chars)?;
                output.push(HctlToken::Atom(Atomic::Prop(c.to_string() + &name)));
            }
            _ => return Err(format!("Unexpected char '{c}'.")),
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
        return is_valid_in_name(*c);
    }
    false
}

/// Check if given optional char represents valid temporal operator.
fn is_valid_temp_op(option_char: Option<&char>) -> bool {
    if let Some(c) = option_char {
        return matches!(c, 'X' | 'F' | 'G' | 'U' | 'W');
    }
    false
}

/// Retrieve the proposition (or variable) name from the input.
/// The first character of the name may or may not be already consumed by the caller.
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

/// Retrieve the name of the variable bound by a hybrid operator.
/// Operator character is consumed by caller and is given as input for error msg purposes.
fn collect_var_from_operator(
    input_chars: &mut Peekable<Chars>,
    operator: char,
) -> Result<String, String> {
    // there might be few spaces first
    let _ = input_chars.take_while(|c| c.is_whitespace());
    // now collect the variable name itself- it is in the form {var_name} for now
    if Some('{') != input_chars.next() {
        return Err(format!("Expected '{{' after '{operator}'."));
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

impl fmt::Display for HctlToken {
    /// Display tokens for debug purposes.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HctlToken::Unary(UnaryOp::Not) => write!(f, "~"),
            HctlToken::Unary(c) => write!(f, "{c:?}"), // unary temporal
            HctlToken::Binary(BinaryOp::And) => write!(f, "&"),
            HctlToken::Binary(BinaryOp::Or) => write!(f, "|"),
            HctlToken::Binary(BinaryOp::Xor) => write!(f, "^"),
            HctlToken::Binary(BinaryOp::Imp) => write!(f, "=>"),
            HctlToken::Binary(BinaryOp::Iff) => write!(f, "<=>"),
            HctlToken::Binary(c) => write!(f, "{c:?}"), // binary temporal
            HctlToken::Hybrid(op, var) => write!(f, "{op:?} {{{var}}}:"),
            HctlToken::Atom(Atomic::Prop(name)) => write!(f, "{name}"),
            HctlToken::Atom(Atomic::Var(name)) => write!(f, "{{{name}}}"),
            HctlToken::Atom(constant) => write!(f, "{constant:?}"),
            HctlToken::Tokens(_) => write!(f, "( TOKENS )"), // debug purposes only
        }
    }
}

#[allow(dead_code)]
/// Recursively print tokens.
fn print_tokens_recursively(tokens: &Vec<HctlToken>) {
    for token in tokens {
        match token {
            HctlToken::Tokens(token_vec) => print_tokens_recursively(token_vec),
            _ => print!("{token} "),
        }
    }
}

#[allow(dead_code)]
/// Print the vector of tokens (for debug purposes).
pub fn print_tokens(tokens: &Vec<HctlToken>) {
    print_tokens_recursively(tokens);
    println!();
}

#[cfg(test)]
mod tests {
    use crate::preprocessing::operator_enums::*;
    use crate::preprocessing::tokenizer::{try_tokenize_formula, HctlToken};

    #[test]
    /// Test tokenization process on several valid HCTL formulae.
    fn test_tokenize_valid_formulae() {
        let valid1 = "!{x}: AG EF {x}".to_string();
        let tokens1_result = try_tokenize_formula(valid1);
        assert_eq!(
            tokens1_result.unwrap(),
            vec![
                HctlToken::Hybrid(HybridOp::Bind, "x".to_string()),
                HctlToken::Unary(UnaryOp::Ag),
                HctlToken::Unary(UnaryOp::Ef),
                HctlToken::Atom(Atomic::Var("x".to_string())),
            ]
        );

        let valid2 = "AF (!{x}: (AX (~{x} & AF {x})))".to_string();
        let tokens2_result = try_tokenize_formula(valid2);
        assert_eq!(
            tokens2_result.unwrap(),
            vec![
                HctlToken::Unary(UnaryOp::Af),
                HctlToken::Tokens(vec![
                    HctlToken::Hybrid(HybridOp::Bind, "x".to_string()),
                    HctlToken::Tokens(vec![
                        HctlToken::Unary(UnaryOp::Ax),
                        HctlToken::Tokens(vec![
                            HctlToken::Unary(UnaryOp::Not),
                            HctlToken::Atom(Atomic::Var("x".to_string())),
                            HctlToken::Binary(BinaryOp::And),
                            HctlToken::Unary(UnaryOp::Af),
                            HctlToken::Atom(Atomic::Var("x".to_string())),
                        ]),
                    ]),
                ]),
            ]
        );

        let valid3 = "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string();
        let tokens3_result = try_tokenize_formula(valid3);
        assert_eq!(
            tokens3_result.unwrap(),
            vec![
                HctlToken::Hybrid(HybridOp::Bind, "x".to_string()),
                HctlToken::Hybrid(HybridOp::Exists, "y".to_string()),
                HctlToken::Tokens(vec![
                    HctlToken::Hybrid(HybridOp::Jump, "x".to_string()),
                    HctlToken::Unary(UnaryOp::Not),
                    HctlToken::Atom(Atomic::Var("y".to_string())),
                    HctlToken::Binary(BinaryOp::And),
                    HctlToken::Unary(UnaryOp::Ax),
                    HctlToken::Atom(Atomic::Var("x".to_string())),
                ]),
                HctlToken::Binary(BinaryOp::And),
                HctlToken::Tokens(vec![
                    HctlToken::Hybrid(HybridOp::Jump, "y".to_string()),
                    HctlToken::Unary(UnaryOp::Ax),
                    HctlToken::Atom(Atomic::Var("y".to_string())),
                ]),
            ]
        );
    }

    #[test]
    /// Test tokenization process on several invalid HCTL formulae.
    fn test_tokenize_invalid_formulae() {
        let invalid_formulae = vec![
            "!{x}: AG EF {x<&}",
            "!{x AG EF {x}",
            "!{}: AG EF {x}",
            "{x}: AG EF {x}",
            "V{x} AG EF {x}",
            "!{x}: AG EX {x} $",
            "!{x}: # AG EF {x}",
            "!{x}: AG* EF {x}",
            "!{x}: (a EW b) =>= (c AU d)",
        ];

        for formula in invalid_formulae {
            assert!(try_tokenize_formula(formula.to_string()).is_err())
        }
    }
}
