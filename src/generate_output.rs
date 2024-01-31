use crate::evaluation::LabelToSetMap;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Create a full results archive for an "result map" of `string -> colored set of states`.
///
/// The archive will contain:
/// - files with BDDs for each result (corresponding key in the hashmap is used as a file name)
/// - metadata file with all the formulae (in order)
/// - the original model file (so that we can load the results later)
pub fn build_result_archive(
    results: LabelToSetMap,
    archive_name: &str,
    original_model_str: &str,
    formulae: Vec<String>,
) -> Result<(), std::io::Error> {
    let archive_path = Path::new(archive_name);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path
        .parent()
        .ok_or(std::io::Error::new(ErrorKind::Other, "Invalid path."))?;
    std::fs::create_dir_all(prefix)?;

    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive);

    for (set_name, set) in results.iter() {
        // The results (including empty BDDs) go directly into the zip archive.
        let bdd_file_name = format!("{}.bdd", set_name);
        zip_writer
            .start_file(&bdd_file_name, FileOptions::default())
            .map_err(std::io::Error::from)?;

        set.as_bdd().write_as_string(&mut zip_writer)?;
    }

    // Include the original model in the result bundle (we need to load the results back later).
    zip_writer
        .start_file("model.aeon", FileOptions::default())
        .map_err(std::io::Error::from)?;
    write!(zip_writer, "{original_model_str}")?;

    // Include the metadata file with the formulae list.
    zip_writer
        .start_file("formulae.txt", FileOptions::default())
        .map_err(std::io::Error::from)?;
    for formula in formulae {
        writeln!(zip_writer, "{formula}")?;
    }

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}

/// Create an archive with a given name, and put the original model file and file with formulae there.
pub fn build_initial_archive(
    archive_name: &str,
    original_model_str: &str,
    formulae: Vec<String>,
) -> Result<(), std::io::Error> {
    let archive_path = Path::new(archive_name);
    // If there are some non existing dirs in path, create them.
    let prefix = archive_path
        .parent()
        .ok_or(std::io::Error::new(ErrorKind::Other, "Invalid path."))?;
    std::fs::create_dir_all(prefix)?;

    // Create a zip writer for the desired archive.
    let archive = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive);

    // Include the original model in the result bundle (we need to load the results back later).
    zip_writer
        .start_file("model.aeon", FileOptions::default())
        .map_err(std::io::Error::from)?;
    write!(zip_writer, "{original_model_str}")?;

    // Include the metadata file with the formulae list.
    zip_writer
        .start_file("formulae.txt", FileOptions::default())
        .map_err(std::io::Error::from)?;
    for formula in formulae {
        writeln!(zip_writer, "{formula}")?;
    }

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}
