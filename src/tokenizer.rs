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
    Atom(Atomic),             // Proposition or variable
    Tokens(Vec<Token>),       // A block of tokens inside parentheses
}

pub fn tokenize_recursive(
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
                    return Result::Err("Expected '>' after '='.".to_string());
                }
            }
            '<' => {
                if Some('=') == input_chars.next() {
                    if Some('>') == input_chars.next() {
                        output.push(Token::Binary(BinaryOp::Iff));
                    } else {
                        return Result::Err("Expected '>' after '<='.".to_string());
                    }
                } else {
                    return Result::Err("Expected '=' after '<'.".to_string());
                }
            }
            // '>' is invalid as a start of a token
            '>' => return Result::Err("Unexpected '>'.".to_string()),

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
                        _ => return Result::Err(format!("Unexpected char '{}' after 'E'.", c2)),
                    }
                } else {
                    return Result::Err("Expected one of '{X,F,G,U,W}' after 'E'.".to_string());
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
                        _ => return Result::Err(format!("Unexpected char '{}' after 'A'.", c2)),
                    }
                } else {
                    return Result::Err("Expected one of '{X,F,G,U,W}' after 'A'.".to_string());
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
            '3' => {
                // we will collect the variable name via inside helper function
                let name = collect_var_from_operator(input_chars, '3')?;
                output.push(Token::Hybrid(HybridOp::Exist, name));
            }
            ')' => {
                return if !top_level {
                    Result::Ok(output)
                } else {
                    Result::Err("Unexpected ')'.".to_string())
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
                output.push(Token::Atom(Atomic::Var(name)));
                if Some('}') != input_chars.next() {
                    return Result::Err("Expected '}'.".to_string());
                }
            }
            // proposition name
            c if is_valid_in_name(c) => {
                let name = collect_name(input_chars)?;
                output.push(Token::Atom(Atomic::Prop(c.to_string() + &name)));
            }
            _ => return Result::Err(format!("Unexpected char '{}'.", c)),
        }
    }

    if top_level {
        Result::Ok(output)
    } else {
        Result::Err("Expected ')'.".to_string())
    }
}

/// Check if given char can appear in a name.
fn is_valid_in_name(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Check if given optional char represents valid temporal operator
fn is_valid_temp_op(option_char: Option<&char>) -> bool {
    if let Some(c) = option_char {
        match c {
            'X' | 'F' | 'G' | 'U' | 'W' => true,
            _ => false,
        }
    }
    false
}

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

fn collect_var_from_operator(
    input_chars: &mut Peekable<Chars>,
    operator: char,
) -> Result<String, String> {
    // there might be few spaces first
    let _ = input_chars.take_while(|c| c.is_whitespace());
    // now collect the variable name itself- it is in the form {var_name} for now
    if Some('{') != input_chars.next() {
        return Result::Err(format!("Expected '{{' after '{}'.", operator));
    }
    let name = collect_name(input_chars)?;

    if Some('}') != input_chars.next() {
        return Result::Err("Expected '}'.".to_string());
    }
    let _ = input_chars.take_while(|c| c.is_whitespace());

    if Some(':') != input_chars.next() {
        return Result::Err("Expected ':'.".to_string());
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
