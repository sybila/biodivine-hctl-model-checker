//! Contains wrappers for loading inputs from the files

use crate::evaluation::LabelToSetMap;
use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicContext};
use std::fs::read_to_string;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Read the formulae from the specified file. Ignore lines starting with `#` (comments).
/// The syntax of these formulae is checked later during parsing.
pub fn load_formulae(formulae_path: &str) -> Result<Vec<String>, String> {
    let formulae_string = read_to_string(formulae_path).map_err(|e| e.to_string())?;

    let mut formulae: Vec<String> = Vec::new();
    for line in formulae_string.lines() {
        let trimmed_line = line.trim();
        if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
            formulae.push(trimmed_line.to_string());
        }
    }
    Ok(formulae)
}

/// Read the contents of a file from a zip archive into a string.
fn read_zipped_file(reader: &mut ZipArchive<File>, file_name: &str) -> Result<String, String> {
    let mut contents = String::new();
    let mut file = reader.by_name(file_name).map_err(|e| e.to_string())?;
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;
    Ok(contents)
}

/// Read the individual BDD files in a provided (valid) archive into a map from the strings (file names) to colored sets.
/// The files must be have the `.bdd` extension.
pub fn load_bdd_bundle(
    archive_path: &str,
    symbolic_context: &SymbolicContext,
) -> Result<LabelToSetMap, String> {
    // Open the zip archive with classification results (existence of file must be checked).
    let archive_file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(archive_file).map_err(|e| e.to_string())?;

    // Load all class BDDs from files in the archive.
    let files = archive
        .file_names()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let mut loaded_sets: LabelToSetMap = LabelToSetMap::new();

    for filename in files {
        // ignore files with different extensions (might be some metadata)
        let extension = Path::new(&filename).extension().and_then(|s| s.to_str());
        if !matches!(extension, Some("bdd")) {
            continue;
        }

        let name = filename.strip_suffix(".bdd").ok_or(format!(
            "Error loading file `{filename}` from the archive {archive_path}."
        ))?;

        let bdd_string = read_zipped_file(&mut archive, filename.as_str())?;
        let bdd = Bdd::from_string(bdd_string.as_str());
        let set = GraphColoredVertices::new(bdd, symbolic_context);
        loaded_sets.insert(name.to_string(), set);
    }
    Ok(loaded_sets)
}
