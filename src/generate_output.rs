use biodivine_lib_bdd::Bdd;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Create a full results archive for an "result map" of `string -> colored set of states`.
///
/// It contains one BDD file for each result, metadata file with all the formulae, and the original model file.
pub fn build_result_archive(
    results: HashMap<String, GraphColoredVertices>,
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

/// Create an archive with a given name, and put the original model file and formulae list file there.
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

/// Add a file with an BDD dump to the existing zip archive.
pub fn dump_bdd_to_zip(
    zip_path: &str,
    bdd: &Bdd,
    file_name_in_zip: &str,
) -> Result<(), std::io::Error> {
    // Open the existing ZIP file and create a ZipWriter instance.
    let mut file = OpenOptions::new().read(true).write(true).open(zip_path)?;
    let mut zip_writer = ZipWriter::new(&mut file);

    // Make a new file with the given content to the ZIP archive.
    zip_writer
        .start_file(file_name_in_zip, FileOptions::default())
        .map_err(std::io::Error::from)?;
    bdd.write_as_string(&mut zip_writer)?;

    zip_writer.finish()?;
    Ok(())
}
