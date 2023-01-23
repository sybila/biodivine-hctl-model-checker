use biodivine_lib_param_bn::BooleanNetwork;

use std::fs::read_to_string;

/// Extract a set of formulae from the given string, one formulae per line. t
/// Does not parse or validate the formulae.
/// Ignore lines starting with `#` (comments).
pub fn parse_formulae(formulae_string: &str) -> Vec<String> {
    let mut formulae: Vec<String> = Vec::new();
    for line in formulae_string.lines() {
        let trimmed_line = line.trim();

        // ignore comments and empty lines
        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }
        formulae.push(trimmed_line.to_string());
    }
    formulae
}

#[allow(dead_code)]
/// Extract the two distinct sets of formulae from the file containing both assertions and
/// properties. The two sets are divided by `+++` delimiter.
pub fn load_assertions_and_properties(formulae_path: &str) -> (Vec<String>, Vec<String>) {
    let formulae_string = read_to_string(formulae_path).unwrap();

    // divide the file into parts divided by delimiters
    let entities: Vec<&str> = formulae_string.split("+++").collect();

    let assertion_formulae = parse_formulae(entities[0]);
    let property_formulae = if entities.len() > 1 {
        parse_formulae(entities[1])
    } else {
        Vec::new()
    };

    (assertion_formulae, property_formulae)
}

/// Extract the entities from the the extended aeon file.
fn get_extended_aeon_entities(
    extended_aeon_str: &str,
) -> Result<(BooleanNetwork, Vec<String>, Vec<String>), String> {
    // divide the file into individual parts
    let entities: Vec<&str> = extended_aeon_str.split("+++").collect();
    if entities.len() > 3 {
        return Err("Input file is not in correct format - too many delimiters.".to_string());
    }

    // parse the BN using lib-param-bn parser
    let bn = BooleanNetwork::try_from(entities[0])?;

    // extract the formulae sets if enough delimiters found
    let assertion_formulae = if entities.len() > 1 {
        parse_formulae(entities[1])
    } else {
        Vec::new()
    };
    let property_formulae = if entities.len() > 2 {
        parse_formulae(entities[2])
    } else {
        Vec::new()
    };

    Ok((bn, assertion_formulae, property_formulae))
}

/// Read the extended aeon file that contains a parametrized BN and two kinds of formulae.
/// Return the BN object and two sets of formulae - assertions and properties. These three types
/// of entities are divided by by a `+++` delimiter in the file.
/// Ignore lines starting with `#` (comments).
pub fn load_extended_aeon(
    path: &str,
) -> Result<(BooleanNetwork, Vec<String>, Vec<String>), String> {
    let extended_aeon_str = read_to_string(path).unwrap();
    get_extended_aeon_entities(extended_aeon_str.as_str())
}

/// Parse only the BN entity from the extended aeon string.
fn get_bn_from_extended_aeon(extended_aeon_str: &str) -> Result<BooleanNetwork, String> {
    // divide the file into individual parts
    let entities: Vec<&str> = extended_aeon_str.split("+++").collect();
    if entities.len() > 3 {
        return Err("Input file is not in correct format - too many delimiters.".to_string());
    }

    // parse the BN using lib-param-bn parser
    BooleanNetwork::try_from(entities[0])
}

/// Parse only the BN entity from the extended aeon file (that contains a BN and two
/// kinds of formulae, all delimited by `+++` line).
pub fn load_bn_from_extended_aeon(path: &str) -> Result<BooleanNetwork, String> {
    let extended_aeon_str = read_to_string(path).unwrap();
    get_bn_from_extended_aeon(extended_aeon_str.as_str())
}

#[cfg(test)]
mod tests {
    use crate::bn_classification::load_input::{
        get_bn_from_extended_aeon, get_extended_aeon_entities,
    };
    use biodivine_lib_param_bn::BooleanNetwork;

    #[test]
    /// Test extracting the entities from the extended AEON format.
    fn test_extracting_extended_aeon() {
        let extended_aeon = r"
            $v_1:v_2
            v_2 -> v_1
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
            +++
            3{x}: @{x}: AG EF {x}
            +++
            3{x}: @{x}: AX {x}
            true
        ";

        let aeon = r"
            $v_1:v_2
            v_2 -> v_1
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";
        let bn = BooleanNetwork::try_from(aeon).unwrap();

        let formulae1 = vec!["3{x}: @{x}: AG EF {x}".to_string()];
        let formulae2 = vec!["3{x}: @{x}: AX {x}".to_string(), "true".to_string()];

        assert_eq!(
            get_extended_aeon_entities(extended_aeon).unwrap(),
            (bn.clone(), formulae1, formulae2),
        );

        assert_eq!(get_bn_from_extended_aeon(extended_aeon).unwrap(), bn,);
    }

    #[test]
    /// Test that extracting entities from the corrupted extended AEON format fails.
    fn test_corrupted_extended_aeon() {
        let extended_aeon = r"
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
            +++
            +++
            3{x}: @{x}: AG EF {x}
            +++
            3{x}: @{x}: AX {x}
            true
        ";

        assert!(get_extended_aeon_entities(extended_aeon).is_err());
        assert!(get_bn_from_extended_aeon(extended_aeon).is_err());
    }

    #[test]
    /// Test that extend aeon extraction also works for basic aeon format.
    fn test_extended_aeon_() {
        let aeon = r"
            $v_2:!v_3
            v_3 -| v_2
            $v_3:v_3
            v_3 -> v_3
        ";

        assert_eq!(
            get_extended_aeon_entities(aeon).unwrap(),
            (BooleanNetwork::try_from(aeon).unwrap(), vec![], vec![]),
        );
        assert_eq!(
            get_bn_from_extended_aeon(aeon).unwrap(),
            BooleanNetwork::try_from(aeon).unwrap(),
        );
    }
}
