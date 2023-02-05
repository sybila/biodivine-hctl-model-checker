//! Contains the functionality for canonizing variable names in sub-formulae.

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// Return a string representing the same subformula, but with canonized var names (var0, var1...).
/// Param `subform_chars` must represent valid formula processed by `check_props_and_rename_vars`.
/// Param `subform_chars` must include all PARENTHESES and must NOT contain excess spaces.
/// For example, `(3{x}:(3{xx}:((@{x}:((~{xx})&(AX{x})))&(@{xx}:(AX{xx})))))` is a valid input.
/// Generally, any `node.subform_string` field should be OK to use.
pub fn canonize_subform(
    mut subform_chars: Peekable<Chars>,
    mut mapping_dict: HashMap<String, String>,
    mut canonical: String,
    mut stack_len: i32,
) -> (Peekable<Chars>, String, HashMap<String, String>, i32) {
    while let Some(ch) = subform_chars.next() {
        let mut should_return = false;
        match ch {
            // dive deeper by one level
            '(' => {
                canonical.push(ch);
                let tuple = canonize_subform(subform_chars, mapping_dict, canonical, stack_len);
                subform_chars = tuple.0;
                canonical = tuple.1;
                mapping_dict = tuple.2;
                stack_len = tuple.3;
            }
            // emerge back to upper level
            ')' => {
                canonical.push(ch);
                should_return = true;
            }
            // introduce new 'quantified' var (jump is not listed as it does not introduce vars)
            // distinguish situations where '3' or 'V' is quantifier and when part of some prop name
            '!' | '3' | 'V' if subform_chars.peek() == Some(&'{') => {
                // move to the beginning of the var name (skip '{')
                subform_chars.next();
                let mut var_name = String::new();
                for name_char in subform_chars.by_ref() {
                    if name_char == '}' {
                        break;
                    }
                    var_name.push(name_char);
                }
                // skip ':'
                subform_chars.next();
                // insert new mapping to dict and push it all to canonical string
                mapping_dict.insert(var_name.clone(), format!("var{stack_len}"));
                canonical.push_str(format!("{ch}{{var{stack_len}}}:").as_str());
                stack_len += 1;
            }
            // rename existing var to canonical form, or handle free variables
            // this includes variable names which are part of the "jump operator"
            '{' => {
                let mut var_name = String::new();
                for name_char in subform_chars.by_ref() {
                    if name_char == '}' {
                        break;
                    }
                    var_name.push(name_char);
                }

                // we must be prepared for free vars to appear (not bounded by hybrid operators)
                // it is because we are canonizing all subformulas in the tree
                if !mapping_dict.contains_key(var_name.as_str()) {
                    mapping_dict.insert(var_name.clone(), format!("var{stack_len}"));
                    stack_len += 1;
                }

                if let Some(canonical_name) = mapping_dict.get(var_name.as_str()) {
                    canonical.push_str(format!("{{{canonical_name}}}").as_str());
                } else {
                    // This branch should never happen
                    println!("Canonical name was not found for {var_name}");
                }
            }
            // all the other character, including boolean+temporal operators, '@', prop names
            _ => {
                canonical.push(ch);
            }
        }
        if should_return {
            break;
        }
    }
    (subform_chars, canonical, mapping_dict, stack_len)
}

#[allow(dead_code)]
/// Returns string of the semantically same sub-formula, but with "canonized" var names.
/// It is used in the process of marking duplicate sub-formulae and caching.
pub fn get_canonical(subform_string: String) -> String {
    let canonized_tuple = canonize_subform(
        subform_string.chars().peekable(),
        HashMap::new(),
        String::new(),
        0,
    );
    canonized_tuple.1
}

/// Return a tuple with the canonized sub-formula string, and mapping of var names to their new
/// canonized form.
/// It is used in the process of marking duplicate sub-formulae and caching.
pub fn get_canonical_and_mapping(subform_string: String) -> (String, HashMap<String, String>) {
    let canonized_tuple = canonize_subform(
        subform_string.chars().peekable(),
        HashMap::new(),
        String::new(),
        0,
    );
    (canonized_tuple.1, canonized_tuple.2)
}

#[cfg(test)]
mod tests {
    use crate::evaluation::canonization::{get_canonical, get_canonical_and_mapping};
    use std::collections::HashMap;

    #[test]
    /// Compare automatically canonized formula to the expected result.
    fn test_canonization_simple() {
        // two formulae that should have same canonization
        let sub_formula1 = "(AX{x})";
        let sub_formula2 = "(AX{xx})";
        let sub_formula_canonized = "(AX{var0})";

        assert_eq!(
            get_canonical(sub_formula1.to_string()),
            sub_formula_canonized.to_string()
        );
        assert_eq!(
            get_canonical(sub_formula2.to_string()),
            sub_formula_canonized.to_string()
        );

        // mappings of variable names between formulae and the canonized version
        let renaming1 = HashMap::from([("x".to_string(), "var0".to_string())]);
        let renaming2 = HashMap::from([("xx".to_string(), "var0".to_string())]);

        assert_eq!(
            get_canonical_and_mapping(sub_formula1.to_string()),
            (sub_formula_canonized.to_string(), renaming1)
        );
        assert_eq!(
            get_canonical_and_mapping(sub_formula2.to_string()),
            (sub_formula_canonized.to_string(), renaming2)
        );
    }

    #[test]
    /// Compare automatically canonized formula to the expected result.
    fn test_canonization_mediate() {
        // two formulae that should have same canonization
        let sub_formula1 = "(AX{x})&(AG(EF{xx}))";
        let sub_formula2 = "(AX{xx})&(AG(EF{xxx}))";
        let sub_formula_canonized = "(AX{var0})&(AG(EF{var1}))";

        assert_eq!(
            get_canonical(sub_formula1.to_string()),
            sub_formula_canonized.to_string()
        );
        assert_eq!(
            get_canonical(sub_formula2.to_string()),
            sub_formula_canonized.to_string()
        );

        // mappings of variable names between formulae and the canonized version
        let renaming1 = HashMap::from([
            ("x".to_string(), "var0".to_string()),
            ("xx".to_string(), "var1".to_string()),
        ]);
        let renaming2 = HashMap::from([
            ("xx".to_string(), "var0".to_string()),
            ("xxx".to_string(), "var1".to_string()),
        ]);

        assert_eq!(
            get_canonical_and_mapping(sub_formula1.to_string()),
            (sub_formula_canonized.to_string(), renaming1)
        );
        assert_eq!(
            get_canonical_and_mapping(sub_formula2.to_string()),
            (sub_formula_canonized.to_string(), renaming2)
        );
    }

    #[test]
    /// Compare automatically canonized formula to the expected result.
    fn test_canonization_complex() {
        // two formulae that should have same canonization
        let sub_formula1 = "(3{x}:(3{xx}:((@{x}:((~{xx})&(AX{x})))&(@{xx}:(AX{xx})))))";
        let sub_formula2 = "(3{xx}:(3{x}:((@{xx}:((~{x})&(AX{xx})))&(@{x}:(AX{x})))))";
        let sub_formula_canonized =
            "(3{var0}:(3{var1}:((@{var0}:((~{var1})&(AX{var0})))&(@{var1}:(AX{var1})))))";

        assert_eq!(
            get_canonical(sub_formula1.to_string()),
            sub_formula_canonized.to_string()
        );
        assert_eq!(
            get_canonical(sub_formula2.to_string()),
            sub_formula_canonized.to_string()
        );

        // mappings of variable names between formulae and the canonized version
        let renaming1 = HashMap::from([
            ("x".to_string(), "var0".to_string()),
            ("xx".to_string(), "var1".to_string()),
        ]);
        let renaming2 = HashMap::from([
            ("xx".to_string(), "var0".to_string()),
            ("x".to_string(), "var1".to_string()),
        ]);

        assert_eq!(
            get_canonical_and_mapping(sub_formula1.to_string()),
            (sub_formula_canonized.to_string(), renaming1)
        );
        assert_eq!(
            get_canonical_and_mapping(sub_formula2.to_string()),
            (sub_formula_canonized.to_string(), renaming2)
        );
    }
}
