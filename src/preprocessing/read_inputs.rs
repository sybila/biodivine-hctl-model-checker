//! Contains wrappers for loading inputs from the files

use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

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

/// Read the contents of a file from a zip archive into a string.
fn read_zipped_file(reader: &mut ZipArchive<File>, file_name: &str) -> Result<String, String> {
    let mut contents = String::new();
    let mut file = reader.by_name(file_name).map_err(|e| e.to_string())?;
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;
    Ok(contents)
}

/// Read the individual BDD files in a provided archive into a map from the strings (file names) to colored sets.
/// The files must be have the `.bdd` extension.
pub fn load_bdd_bundle(
    archive_path: &str,
    stg: SymbolicAsyncGraph,
) -> Result<HashMap<String, GraphColoredVertices>, String> {
    // Open the zip archive with classification results.
    let archive_file = File::open(archive_path).unwrap();
    let mut archive = ZipArchive::new(archive_file).unwrap();

    // Load all class BDDs from files in the archive.
    let files = archive
        .file_names()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let mut context_sets: HashMap<String, GraphColoredVertices> = HashMap::new();

    for filename in files {
        let error_msg = format!("Unexpected file `{}` in the input bundle.", filename);
        let extension = Path::new(&filename).extension().and_then(|s| s.to_str());
        if !matches!(extension, Some("bdd")) {
            return Err(error_msg);
        }
        let name = filename.strip_suffix(".bdd").unwrap();

        let bdd_string = read_zipped_file(&mut archive, filename.as_str())?;
        let bdd = Bdd::from_string(bdd_string.as_str());
        let set = GraphColoredVertices::new(bdd, stg.symbolic_context());
        context_sets.insert(name.to_string(), set);
    }
    Ok(context_sets)
}
