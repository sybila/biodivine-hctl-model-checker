//! Contains wrappers for loading inputs from the files

use biodivine_lib_param_bn::BooleanNetwork;
use std::fs::read_to_string;

/// Read the formulae from the specified file. Ignore lines starting with `#` (comments).
/// The syntax of these formulae is checked later during parsing.
pub fn load_formulae(formulae_path: &str) -> Vec<String> {
    let formulae_string = read_to_string(formulae_path).unwrap();
    let mut formulae: Vec<String> = Vec::new();
    for line in formulae_string.lines() {
        let trimmed_line = line.trim();
        if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
            formulae.push(trimmed_line.to_string());
        }
    }
    formulae
}
