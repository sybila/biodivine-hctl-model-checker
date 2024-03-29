//! Contains the functionality for canonizing variable names in sub-formulae.

use crate::evaluation::VarRenameMap;
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// Return a string representing the same sub-formula, but with canonized var names (var0, var1...).
///
/// Param `subform_chars` must represent valid formula processed by `validate_props_and_rename_vars`. For instance,
/// the formula must include parentheses at all places where they are needed, and var names must have a certain format.
/// For example, `(3{x}: (3{xx}: ((@{x}: ((~{xx}) & (AX {x}))) & (@{xx}: (AX {xx})))))` is a valid input.
/// Generally, any `formula_str` field of `HctlTreeNode` should have the right format.
pub fn canonize_subform(
    mut subform_chars: Peekable<Chars>,
    mut renaming_map: VarRenameMap,
    mut canonical: String,
    mut stack_len: i32,
) -> (Peekable<Chars>, String, VarRenameMap, i32) {
    while let Some(ch) = subform_chars.next() {
        let mut should_return = false;
        match ch {
            // dive deeper by one level
            '(' => {
                canonical.push(ch);
                let tuple = canonize_subform(subform_chars, renaming_map, canonical, stack_len);
                subform_chars = tuple.0;
                canonical = tuple.1;
                renaming_map = tuple.2;
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

                // the rest of the quantifier-related characters (domain label, or just ':') are
                // handled as everything else in following iterations

                // insert new mapping to dict and push it all to canonical string
                renaming_map.insert(var_name.clone(), format!("var{stack_len}"));
                canonical.push_str(format!("{ch}{{var{stack_len}}}").as_str());
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
                if !renaming_map.contains_key(var_name.as_str()) {
                    renaming_map.insert(var_name.clone(), format!("var{stack_len}"));
                    stack_len += 1;
                }

                if let Some(canonical_name) = renaming_map.get(var_name.as_str()) {
                    canonical.push_str(format!("{{{canonical_name}}}").as_str());
                } else {
                    // This branch should never happen
                    println!("Canonical name was not found for {var_name}");
                }
            }
            // all the other characters, including boolean+temporal operators, '@', prop names
            _ => {
                canonical.push(ch);
            }
        }
        if should_return {
            break;
        }
    }
    (subform_chars, canonical, renaming_map, stack_len)
}

#[allow(dead_code)]
/// Returns string of the semantically same sub-formula, but with "canonized" var names.
///
/// It is used for the purposes of marking duplicate sub-formulae and for caching.
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
///
/// It is used for the purposes of marking duplicate sub-formulae and for caching.
pub fn get_canonical_and_renaming(subform_string: String) -> (String, VarRenameMap) {
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
    use crate::evaluation::canonization::{get_canonical, get_canonical_and_renaming};
    use std::collections::HashMap;

    #[test]
    /// Compare automatically canonized formula to the expected result.
    fn canonization_simple() {
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
            get_canonical_and_renaming(sub_formula1.to_string()),
            (sub_formula_canonized.to_string(), renaming1)
        );
        assert_eq!(
            get_canonical_and_renaming(sub_formula2.to_string()),
            (sub_formula_canonized.to_string(), renaming2)
        );
    }

    #[test]
    /// Compare automatically canonized formula to the expected result.
    fn canonization_mediate() {
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
            get_canonical_and_renaming(sub_formula1.to_string()),
            (sub_formula_canonized.to_string(), renaming1)
        );
        assert_eq!(
            get_canonical_and_renaming(sub_formula2.to_string()),
            (sub_formula_canonized.to_string(), renaming2)
        );
    }

    #[test]
    /// Compare automatically canonized formula to the expected result.
    fn canonization_complex() {
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
            get_canonical_and_renaming(sub_formula1.to_string()),
            (sub_formula_canonized.to_string(), renaming1)
        );
        assert_eq!(
            get_canonical_and_renaming(sub_formula2.to_string()),
            (sub_formula_canonized.to_string(), renaming2)
        );
    }

    #[test]
    /// Compare automatically canonized formulas that contains wild-card properties and restricted
    /// var domains to the expected result.
    fn canonization_with_domains() {
        let formula = "(!{x}in%d1%:({x}&%p1%))";
        let formula_canonized = "(!{var0}in%d1%:({var0}&%p1%))";

        assert_eq!(
            get_canonical(formula.to_string()),
            formula_canonized.to_string()
        );

        // mappings of variable names between formulae and the canonized version
        let renaming1 = HashMap::from([("x".to_string(), "var0".to_string())]);
        assert_eq!(
            get_canonical_and_renaming(formula.to_string()),
            (formula_canonized.to_string(), renaming1)
        );
    }
}
