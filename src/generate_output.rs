use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use std::collections::HashMap;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Create results archive for an "result map" of `string -> colored set of states`.
pub fn build_result_archive(
    results: HashMap<String, GraphColoredVertices>,
    archive_name: &str,
    original_model_str: &str,
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

    zip_writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}
