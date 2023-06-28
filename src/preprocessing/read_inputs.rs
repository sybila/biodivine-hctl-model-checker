//! Contains wrappers for loading inputs from the files

use biodivine_lib_param_bn::BooleanNetwork;
use std::fs::read_to_string;

/// Load and parse the BN model in a given format from the specified file.
/// Return error if model is invalid.
pub fn load_and_parse_bn_model(format: &str, model_path: &str) -> Result<BooleanNetwork, String> {
    let maybe_model_string = read_to_string(model_path);
    match maybe_model_string {
        Err(e) => Err(format!("{e}")),
        Ok(model_string) => match format {
            "aeon" => BooleanNetwork::try_from(model_string.as_str()),
            "sbml" => Ok(BooleanNetwork::try_from_sbml(model_string.as_str())?.0),
            "bnet" => BooleanNetwork::try_from_bnet(model_string.as_str()),
            // this cant really happen, just here to be exhaustive
            _ => Err("Invalid model format".to_string()),
        },
    }
}

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
