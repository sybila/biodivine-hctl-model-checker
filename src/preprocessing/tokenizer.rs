//! Contains functionality regarding the tokenizing of HCTL formula string.

use crate::preprocessing::operator_enums::*;

use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

/// Enum of all possible tokens occurring in a HCTL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum HctlToken {
    /// Unary operators: '~','EX','AX','EF','AF','EG','AG'.
    Unary(UnaryOp),
    /// Binary operators: '&','|','^','=>','<=>','EU','AU','EW','AW'.
    Binary(BinaryOp),
    /// Hybrid operator (and its variable, and optional domain): '!', '@', '3', 'V'.
    Hybrid(HybridOp, String, Option<String>),
    /// Proposition, variable, 'true'/'false' constants, wild-card property.
    Atom(Atomic),
    /// A block of tokens inside parentheses.
    Tokens(Vec<HctlToken>),
}

/// Try to tokenize given HCTL formula string.
///
/// This is a wrapper for the (more general) recursive [try_tokenize_formula]` function.
pub fn try_tokenize_formula(formula: String) -> Result<Vec<HctlToken>, String> {
    try_tokenize_recursive(&mut formula.chars().peekable(), true, false)
}

/// Try to tokenize given `extended` HCTL formula string. That means that formula can include
/// `wild-card propositions` or variable domains.
///
/// This is a wrapper for the (more general) recursive [try_tokenize_formula]` function.
pub fn try_tokenize_extended_formula(formula: String) -> Result<Vec<HctlToken>, String> {
    try_tokenize_recursive(&mut formula.chars().peekable(), true, true)
}

/// Process a peekable iterator of characters into a vector of `HctlToken`s.
///
/// If `parse_wild_cards` is `true`, `wild-card propositions` and `variable domains` are allowed to
/// be in the formula. Otherwise, only classical HCTL components are allowed.
fn try_tokenize_recursive(
    input_chars: &mut Peekable<Chars>,
    top_level: bool,
    parse_wild_cards: bool,
) -> Result<Vec<HctlToken>, String> {
    let mut output = Vec::new();

    while let Some(c) = input_chars.next() {
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
                        'X' => output.push(HctlToken::Unary(UnaryOp::EX)),
                        'F' => output.push(HctlToken::Unary(UnaryOp::EF)),
                        'G' => output.push(HctlToken::Unary(UnaryOp::EG)),
                        'U' => output.push(HctlToken::Binary(BinaryOp::EU)),
                        'W' => output.push(HctlToken::Binary(BinaryOp::EW)),
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
                        'X' => output.push(HctlToken::Unary(UnaryOp::AX)),
                        'F' => output.push(HctlToken::Unary(UnaryOp::AF)),
                        'G' => output.push(HctlToken::Unary(UnaryOp::AG)),
                        'U' => output.push(HctlToken::Binary(BinaryOp::AU)),
                        'W' => output.push(HctlToken::Binary(BinaryOp::AW)),
                        _ => return Err(format!("Unexpected char '{c2}' after 'A'.")),
                    }
                } else {
                    return Err("Expected one of '{X,F,G,U,W}' after 'A'.".to_string());
                }
            }
            '!' => {
                // collect the variable name via inside helper function
                let (name, domain) =
                    collect_var_and_dom_from_operator(input_chars, '!', parse_wild_cards)?;
                output.push(HctlToken::Hybrid(HybridOp::Bind, name, domain));
            }
            // "3" can be either exist quantifier or part of some proposition
            '3' if !is_valid_in_name_optional(input_chars.peek()) => {
                // collect the variable name via inside helper function
                let (name, domain) =
                    collect_var_and_dom_from_operator(input_chars, '3', parse_wild_cards)?;
                output.push(HctlToken::Hybrid(HybridOp::Exists, name, domain));
            }
            // "V" can be either forall quantifier or part of some proposition
            'V' if !is_valid_in_name_optional(input_chars.peek()) => {
                // collect the variable name via inside helper function
                let (name, domain) =
                    collect_var_and_dom_from_operator(input_chars, 'V', parse_wild_cards)?;
                output.push(HctlToken::Hybrid(HybridOp::Forall, name, domain));
            }
            '@' => {
                // collect the variable name via inside helper function
                let (name, domain) = collect_var_and_dom_from_operator(input_chars, '@', false)?;
                if domain.is_some() {
                    return Err("Cannot specify domain after '@'.".to_string());
                }
                output.push(HctlToken::Hybrid(HybridOp::Jump, name, None));
            }
            // long name for hybrid operators (\bind, \exists, \forall, \jump)
            '\\' => {
                // collect the name of the operator, and its variable/domain
                let operator_name = collect_name(input_chars)?;
                if &operator_name == "exists" {
                    let (name, domain) =
                        collect_var_and_dom_from_operator(input_chars, '3', parse_wild_cards)?;
                    output.push(HctlToken::Hybrid(HybridOp::Exists, name, domain));
                } else if operator_name == "forall" {
                    let (name, domain) =
                        collect_var_and_dom_from_operator(input_chars, 'V', parse_wild_cards)?;
                    output.push(HctlToken::Hybrid(HybridOp::Forall, name, domain));
                } else if operator_name == "bind" {
                    let (name, domain) =
                        collect_var_and_dom_from_operator(input_chars, '!', parse_wild_cards)?;
                    output.push(HctlToken::Hybrid(HybridOp::Bind, name, domain));
                } else if operator_name == "jump" {
                    let (name, domain) =
                        collect_var_and_dom_from_operator(input_chars, '@', false)?;
                    if domain.is_some() {
                        return Err("Cannot specify domain after '@' operator.".to_string());
                    }
                    output.push(HctlToken::Hybrid(HybridOp::Jump, name, None));
                } else {
                    return Err(format!("Invalid hybrid operator `\\{operator_name}`."));
                }
            }
            ')' => {
                return if !top_level {
                    Ok(output)
                } else {
                    Err("Unexpected ')' without opening counterpart.".to_string())
                }
            }
            '(' => {
                // start a nested token group
                let token_group = try_tokenize_recursive(input_chars, false, parse_wild_cards)?;
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
                    return Err("Expected '}' without opening counterpart.".to_string());
                }
            }
            // wild-card proposition name
            '%' if parse_wild_cards => {
                let name = collect_name(input_chars)?;
                if name.is_empty() {
                    return Err("Wild-card proposition name can't be empty.".to_string());
                }
                output.push(HctlToken::Atom(Atomic::WildCardProp(name)));
                if Some('%') != input_chars.next() {
                    return Err("Expected '%' after wild-card proposition name.".to_string());
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
        Err("Expected ')' to previously encountered opening counterpart.".to_string())
    }
}

/// Check all whitespaces at the front of the iterator.
fn skip_whitespaces(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next(); // Skip the whitespace character
        } else {
            break; // Stop skipping when a non-whitespace character is found
        }
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

/// Retrieve the name (of a proposition or variable) from the input.
/// The first character of the name may or may not be already consumed by the caller.
fn collect_name(input_chars: &mut Peekable<Chars>) -> Result<String, String> {
    let mut name = Vec::new();
    while let Some(c) = input_chars.peek() {
        if !is_valid_in_name(*c) {
            break;
        } else {
            name.push(*c);
            input_chars.next(); // advance iterator
        }
    }
    Ok(name.into_iter().collect())
}

/// Retrieve the name of the variable, and optional name for the domain, bound by a hybrid operator.
/// Operator character is consumed by caller and is given as input for error msg purposes.
///
/// Domains are allowed (but not required) only if `parse_domains` is true.
fn collect_var_and_dom_from_operator(
    input_chars: &mut Peekable<Chars>,
    operator: char,
    parse_domains: bool,
) -> Result<(String, Option<String>), String> {
    // there might be few spaces first
    skip_whitespaces(input_chars);
    // now collect the variable name itself- it is in the form {var_name} for now
    if Some('{') != input_chars.next() {
        return Err(format!("Expected '{{' after '{operator}'."));
    }
    let name = collect_name(input_chars)?;
    if name.is_empty() {
        return Err("Variable name can't be empty.".to_string());
    }
    if Some('}') != input_chars.next() {
        return Err(format!(
            "Expected '}}' after variable name (in '{operator}' segment)."
        ));
    }
    skip_whitespaces(input_chars);

    let mut domain = None;
    if parse_domains {
        // there are 2 options:
        // a) domain is specified and thus relevant chars form "in %domain%:"
        // b) domain is not specified and thus next char must be ":"
        if let Some('i') = input_chars.peek() {
            // the "in" part
            input_chars.next();
            if Some('n') != input_chars.next() {
                return Err("Expected 'n' after 'i' (in domain specification).".to_string());
            }
            skip_whitespaces(input_chars);

            // the "%domain%" part
            if Some('%') != input_chars.next() {
                return Err("Expected '%' before domain name.".to_string());
            }
            let domain_name = collect_name(input_chars)?;
            if domain_name.is_empty() {
                return Err("Variable's domain name can't be empty.".to_string());
            }
            domain = Some(domain_name);
            if Some('%') != input_chars.next() {
                return Err("Expected '%' after domain name.".to_string());
            }
            skip_whitespaces(input_chars);
        }
    }
    if Some(':') != input_chars.next() {
        return Err(format!(
            "Expected ':' after segment of hybrid operator '{operator}'."
        ));
    }
    Ok((name, domain))
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
            HctlToken::Hybrid(op, var, None) => write!(f, "{op:?} {{{var}}}:"),
            HctlToken::Hybrid(op, var, Some(dom)) => write!(f, "{op:?} {{{var}}} in %{dom}%:"),
            HctlToken::Atom(Atomic::Prop(name)) => write!(f, "{name}"),
            HctlToken::Atom(Atomic::Var(name)) => write!(f, "{{{name}}}"),
            HctlToken::Atom(Atomic::WildCardProp(name)) => write!(f, "%{name}%"),
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
    use crate::preprocessing::tokenizer::{
        try_tokenize_extended_formula, try_tokenize_formula, HctlToken,
    };

    #[test]
    /// Test tokenization process on several valid HCTL formulae.
    /// Test both some important and meaningful formulae and formulae that include wide
    /// range of operators.
    fn tokenize_valid_formulae() {
        // formula for attractors (both variants of hybrid operator syntax)
        let formula = "!{x}: AG EF {x}".to_string();
        let tokens = try_tokenize_formula(formula).unwrap();
        let formula_v2 = "\\bind {x}: AG EF {x}".to_string();
        let tokens_v2 = try_tokenize_formula(formula_v2).unwrap();
        let expected_tokens = vec![
            HctlToken::Hybrid(HybridOp::Bind, "x".to_string(), None),
            HctlToken::Unary(UnaryOp::AG),
            HctlToken::Unary(UnaryOp::EF),
            HctlToken::Atom(Atomic::Var("x".to_string())),
        ];
        assert_eq!(tokens, expected_tokens);
        assert_eq!(tokens_v2, expected_tokens);

        // formula for cyclic attractors (both variants of hybrid operator syntax)
        let formula = "!{x}: (AX (~{x} & AF {x}))".to_string();
        let tokens = try_tokenize_formula(formula).unwrap();
        let formula_v2 = "\\bind {x}: (AX (~{x} & AF {x}))".to_string();
        let tokens_v2 = try_tokenize_formula(formula_v2).unwrap();
        let expected_tokens = vec![
            HctlToken::Hybrid(HybridOp::Bind, "x".to_string(), None),
            HctlToken::Tokens(vec![
                HctlToken::Unary(UnaryOp::AX),
                HctlToken::Tokens(vec![
                    HctlToken::Unary(UnaryOp::Not),
                    HctlToken::Atom(Atomic::Var("x".to_string())),
                    HctlToken::Binary(BinaryOp::And),
                    HctlToken::Unary(UnaryOp::AF),
                    HctlToken::Atom(Atomic::Var("x".to_string())),
                ]),
            ]),
        ];
        assert_eq!(tokens, expected_tokens);
        assert_eq!(tokens_v2, expected_tokens);

        // formula for bi-stability (both variants of hybrid operator syntax)
        let formula = "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string();
        let tokens = try_tokenize_formula(formula).unwrap();
        let formula_v2 =
            "\\bind {x}: \\exists {y}: (\\jump {x}: ~{y} & AX {x}) & (\\jump {y}: AX {y})"
                .to_string();
        let tokens_v2 = try_tokenize_formula(formula_v2).unwrap();
        let expected_tokens = vec![
            HctlToken::Hybrid(HybridOp::Bind, "x".to_string(), None),
            HctlToken::Hybrid(HybridOp::Exists, "y".to_string(), None),
            HctlToken::Tokens(vec![
                HctlToken::Hybrid(HybridOp::Jump, "x".to_string(), None),
                HctlToken::Unary(UnaryOp::Not),
                HctlToken::Atom(Atomic::Var("y".to_string())),
                HctlToken::Binary(BinaryOp::And),
                HctlToken::Unary(UnaryOp::AX),
                HctlToken::Atom(Atomic::Var("x".to_string())),
            ]),
            HctlToken::Binary(BinaryOp::And),
            HctlToken::Tokens(vec![
                HctlToken::Hybrid(HybridOp::Jump, "y".to_string(), None),
                HctlToken::Unary(UnaryOp::AX),
                HctlToken::Atom(Atomic::Var("y".to_string())),
            ]),
        ];
        assert_eq!(tokens, expected_tokens);
        assert_eq!(tokens_v2, expected_tokens);

        // Formula with all kinds of binary operators, and terminals, including propositions
        // starting on A/E. Note that constants are treated as propositions at this point.
        let formula = "(prop1 <=> PROP2 | false => 1) EU (0 AW A_prop ^ E_prop)".to_string();
        let tokens = try_tokenize_formula(formula).unwrap();
        let expected_tokens = vec![
            HctlToken::Tokens(vec![
                HctlToken::Atom(Atomic::Prop("prop1".to_string())),
                HctlToken::Binary(BinaryOp::Iff),
                HctlToken::Atom(Atomic::Prop("PROP2".to_string())),
                HctlToken::Binary(BinaryOp::Or),
                HctlToken::Atom(Atomic::Prop("false".to_string())),
                HctlToken::Binary(BinaryOp::Imp),
                HctlToken::Atom(Atomic::Prop("1".to_string())),
            ]),
            HctlToken::Binary(BinaryOp::EU),
            HctlToken::Tokens(vec![
                HctlToken::Atom(Atomic::Prop("0".to_string())),
                HctlToken::Binary(BinaryOp::AW),
                HctlToken::Atom(Atomic::Prop("A_prop".to_string())),
                HctlToken::Binary(BinaryOp::Xor),
                HctlToken::Atom(Atomic::Prop("E_prop".to_string())),
            ]),
        ];
        assert_eq!(tokens, expected_tokens);
    }

    #[test]
    /// Test tokenization process on HCTL formula with several whitespaces.
    fn tokenize_with_whitespaces() {
        let valid_formula = " 3   {x} :  @      {x}   :  AG  EF    {x} ";
        assert!(try_tokenize_formula(valid_formula.to_string()).is_ok());

        let valid_formula = " \\exists   {x}  : \\jump   {x} :  AG  EF    {x} ";
        assert!(try_tokenize_formula(valid_formula.to_string()).is_ok());
    }

    #[test]
    /// Test tokenization process on several invalid HCTL formulae.
    /// Try to cover wide range of invalid possibilities, as well as potential frequent mistakes.
    fn tokenize_invalid_formulae() {
        let invalid_formulae = vec![
            "!{x}: AG EF {x<&}",
            "\\bind {x}: A* EF {x}",
            "!{x}: A* EF {x}",
            "!{x}: AG E* {x}",
            "!{x}: AG EF {}",
            "!{x AG EF {x}",
            "!{}: AG EF {x}",
            "!EF p1",
            "{x}: AG EF {x}",
            "V{x} AG EF {x}",
            "\\forall {x} AG EF {x}",
            "!{x}: AG EX {x} $",
            "!{x}: # AG EF {x}",
            "!{x}: AG* EF {x}",
            "!{x}: (a EW b) =>= (c AU d)",
            "p1 )",
            "( p1",
            "p1 <> p2",
            "p1 >= p2",
            "p1 <= p2",
        ];

        for formula in invalid_formulae {
            assert!(try_tokenize_formula(formula.to_string()).is_err())
        }
    }

    #[test]
    /// Test tokenization process on several extended HCTL formulae containing
    /// `wild-card propositions` or `variable domains`.
    fn tokenize_extended_formulae() {
        let formula = "p & %p%";
        // tokenizer for standard HCTL should fail, for extended succeed
        assert!(try_tokenize_formula(formula.to_string()).is_err());
        assert!(try_tokenize_extended_formula(formula.to_string()).is_ok());
        let tokens = try_tokenize_extended_formula(formula.to_string()).unwrap();
        let expected_tokens = vec![
            HctlToken::Atom(Atomic::Prop("p".to_string())),
            HctlToken::Binary(BinaryOp::And),
            HctlToken::Atom(Atomic::WildCardProp("p".to_string())),
        ];
        assert_eq!(tokens, expected_tokens);

        let formula = "V{x}: (@{x}: {x} & %s%)";
        let formula_v2 = "\\forall {x}: (\\jump{x}: {x} & %s%)";
        assert!(try_tokenize_formula(formula.to_string()).is_err());
        assert!(try_tokenize_extended_formula(formula.to_string()).is_ok());
        let tokens = try_tokenize_extended_formula(formula.to_string()).unwrap();
        let tokens_v2 = try_tokenize_extended_formula(formula_v2.to_string()).unwrap();
        let expected_tokens = vec![
            HctlToken::Hybrid(HybridOp::Forall, "x".to_string(), None),
            HctlToken::Tokens(vec![
                HctlToken::Hybrid(HybridOp::Jump, "x".to_string(), None),
                HctlToken::Atom(Atomic::Var("x".to_string())),
                HctlToken::Binary(BinaryOp::And),
                HctlToken::Atom(Atomic::WildCardProp("s".to_string())),
            ]),
        ];
        assert_eq!(tokens, expected_tokens);
        assert_eq!(tokens_v2, expected_tokens);

        let formula = "3{x} in %dom_x%: {x}";
        let formula_v2 = "\\exists {x} in %dom_x%: {x}";
        assert!(try_tokenize_formula(formula.to_string()).is_err());
        assert!(try_tokenize_extended_formula(formula.to_string()).is_ok());
        let tokens = try_tokenize_extended_formula(formula.to_string()).unwrap();
        let tokens_v2 = try_tokenize_extended_formula(formula_v2.to_string()).unwrap();
        let expected_tokens = vec![
            HctlToken::Hybrid(HybridOp::Exists, "x".to_string(), Some("dom_x".to_string())),
            HctlToken::Atom(Atomic::Var("x".to_string())),
        ];
        assert_eq!(tokens, expected_tokens);
        assert_eq!(tokens_v2, expected_tokens);

        let formula = "!{x} in %dom_x%: %wild_card%";
        let formula_v2 = "\\bind {x} in %dom_x%: %wild_card%";
        assert!(try_tokenize_formula(formula.to_string()).is_err());
        assert!(try_tokenize_extended_formula(formula.to_string()).is_ok());
        let tokens = try_tokenize_extended_formula(formula.to_string()).unwrap();
        let tokens_v2 = try_tokenize_extended_formula(formula_v2.to_string()).unwrap();
        let expected_tokens = vec![
            HctlToken::Hybrid(HybridOp::Bind, "x".to_string(), Some("dom_x".to_string())),
            HctlToken::Atom(Atomic::WildCardProp("wild_card".to_string())),
        ];
        assert_eq!(tokens, expected_tokens);
        assert_eq!(tokens_v2, expected_tokens);
    }

    #[test]
    /// Test tokenization process on an extended HCTL formula with several whitespaces.
    fn tokenize_extended_with_whitespaces() {
        let valid_formula = " !   {x}   in  %d%  :  AF   %u%   ";
        assert!(try_tokenize_extended_formula(valid_formula.to_string()).is_ok())
    }

    #[test]
    /// Test tokenization process on several invalid extended HCTL formulae.
    /// Try to cover wide range of invalid possibilities, as well as potential frequent mistakes.
    fn tokenize_invalid_extended() {
        let invalid_formulae = vec![
            "!{x} in: AG EF {x}",
            "\\bind {x} in: AG EF {x}",
            "!{x} i %d%: AG EF {x}",
            "\\bind {x} i %d%: AG EF {x}",
            "!{x} in %%: AG EF {x}",
            "\\bind {x} in %%: AG EF {x}",
            "!{x} %d%: AG EF {x}",
            "!{x} in abc: AG EF {x}",
            "!{} in %d%: AG EF {x}",
            "3{x}: @{x} in %s%: AG EF {x}",
            "\\exists {x}: \\jump {x} in %s%: AG EF {x}",
            "%%",
            "%ddd %",
            "%ddd*%",
            "A & d %",
            "A & %d",
            "A & d%",
        ];

        for formula in invalid_formulae {
            assert!(try_tokenize_extended_formula(formula.to_string()).is_err())
        }
    }
}
