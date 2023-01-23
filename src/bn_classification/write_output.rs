//! Finish the classification process and generate the results (report and BDD representation).

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;

use std::fs::create_dir_all;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use zip::write::{FileOptions, ZipWriter};

/// Transform integer into a corresponding binary number of given length
fn int_to_bool_vec(mut number: i32, bits_num: i32) -> Vec<bool> {
    let mut bits: Vec<bool> = Vec::new();

    while number > 0 {
        bits.push(number % 2 == 1);
        number /= 2;
    }
    let mut missing_bits = bits_num - i32::try_from(bits.len()).unwrap();
    while missing_bits > 0 {
        bits.push(false);
        missing_bits -= 1;
    }
    bits.reverse();
    bits
}

/// Convert a vector of bools to the corresponding binary string
fn bool_vec_to_string(bool_vec: Vec<bool>) -> String {
    bool_vec.into_iter().fold("".to_string(), |mut s, b| {
        if b {
            s.push('1');
            s
        } else {
            s.push('0');
            s
        }
    })
}

/// Write a short summary regarding each category of the color decomposition, and dump a BDD
/// encoding the colors, all into the `archive_name` zip.
///
/// `all_valid_colors` represents a "unit color set" - all colors satisfying the assertion formulae.
/// Each result corresponds to colors satisfying some property formula.
/// Each category is given by the set of colors that satisfy exactly the same properties.
pub fn write_class_report_and_dump_bdds(
    assertion_formulae: &Vec<String>,
    all_valid_colors: GraphColors,
    property_formulae: &[String],
    property_results: &Vec<GraphColors>,
    archive_name: &str,
    num_hctl_vars: usize,
) {
    let archive_path = Path::new(archive_name);
    // if there are some non existing dirs in path, create them
    let prefix = archive_path.parent().unwrap();
    create_dir_all(prefix).unwrap();

    let archive = File::create(archive_path).unwrap();
    let mut zip_writer = ZipWriter::new(archive);

    // write the metadata regarding the number of (symbolic) HCTL vars used during the computation
    zip_writer
        .start_file("metadata.txt", FileOptions::default())
        .unwrap();
    zip_writer
        .write_all(format!("{}", num_hctl_vars).as_bytes())
        .unwrap();

    let mut report_buffer = String::new();
    report_buffer.push_str("### Assertion formulae\n\n");
    for assertion_formula in assertion_formulae {
        report_buffer.push_str(format!("# {}\n", assertion_formula).as_str());
    }
    report_buffer.push_str(
        format!(
            "{} colors satisfy all assertions\n\n",
            all_valid_colors.approx_cardinality()
        )
        .as_str(),
    );

    report_buffer.push_str("### Property formulae individually\n\n");
    for (i, property_formula) in property_formulae.iter().enumerate() {
        report_buffer.push_str(format!("# {}\n", property_formula).as_str());
        report_buffer.push_str(
            format!(
                "{} colors satisfy this property\n\n",
                property_results[i].approx_cardinality()
            )
            .as_str(),
        );
    }

    report_buffer.push_str("### Categories\n\n");
    let mut i = 0;
    let i_max = i32::pow(2, property_results.len() as u32);
    while i < i_max {
        // for this category, get indices of properties that are satisfied/unsatisfied
        let bool_indices = int_to_bool_vec(i, property_results.len() as i32);

        // this category contains colors that are conjunction of results or their negation
        let mut category_colors = all_valid_colors.clone();
        for (j, property_res) in property_results.iter().enumerate() {
            if bool_indices[j] {
                category_colors = category_colors.intersect(property_res);
            } else {
                let complement_res = all_valid_colors.minus(property_res);
                category_colors = category_colors.intersect(&complement_res);
            }
        }

        report_buffer
            .push_str(format!("# {}\n", bool_vec_to_string(bool_indices.clone())).as_str());
        report_buffer.push_str(
            format!(
                "{} colors in this category\n\n",
                category_colors.approx_cardinality()
            )
            .as_str(),
        );

        // save the corresponding BDD, if the set is not empty
        if category_colors.approx_cardinality() > 0. {
            // dump the corresponding BDD
            let bdd_file_name =
                format!("bdd_dump_{}.txt", bool_vec_to_string(bool_indices.clone()));
            let bdd_str = category_colors.as_bdd().to_string();
            zip_writer
                .start_file(bdd_file_name.as_str(), FileOptions::default())
                .unwrap();
            zip_writer.write_all(bdd_str.as_bytes()).unwrap();
        }

        i += 1;
    }

    // write the report output
    zip_writer
        .start_file("report.txt", FileOptions::default())
        .unwrap();
    zip_writer.write_all(report_buffer.as_bytes()).unwrap();
    zip_writer.finish().unwrap();
}

/// Write a short summary regarding the computation where assertions were not satisfied
pub fn write_empty_report(assertion_formulae: &Vec<String>, archive_name: &str) {
    let archive_path = Path::new(archive_name);
    let archive = File::create(archive_path).unwrap();
    let mut zip_writer = ZipWriter::new(archive);

    // write the shortened report
    zip_writer
        .start_file("report.txt", FileOptions::default())
        .unwrap();
    let mut report_buffer = String::new();
    report_buffer.push_str("### Assertion formulae\n\n");
    for assertion_formula in assertion_formulae {
        report_buffer.push_str(format!("# {}\n", assertion_formula).as_str());
    }
    report_buffer.push_str("0 colors satisfy all assertions\n\n");
    zip_writer.write_all(report_buffer.as_bytes()).unwrap();
    zip_writer.finish().unwrap();
}

#[cfg(test)]
mod tests {
    use crate::bn_classification::write_output::{bool_vec_to_string, int_to_bool_vec};

    #[test]
    fn test_int_to_bool_vec() {
        let expected_vec = vec![false, false, false];
        assert_eq!(int_to_bool_vec(0, 3), expected_vec);

        let expected_vec = vec![false, true];
        assert_eq!(int_to_bool_vec(1, 2), expected_vec);

        let expected_vec = vec![false, false, false, true];
        assert_eq!(int_to_bool_vec(1, 4), expected_vec);

        let expected_vec = vec![false, false, true, false];
        assert_eq!(int_to_bool_vec(2, 4), expected_vec);

        let expected_vec = vec![true, true, true, true];
        assert_eq!(int_to_bool_vec(15, 4), expected_vec);
    }

    #[test]
    fn test_bool_vec_to_string() {
        assert_eq!(bool_vec_to_string(Vec::new()), "".to_string());

        let bool_vec = vec![true, false];
        assert_eq!(bool_vec_to_string(bool_vec), "10".to_string());

        let bool_vec = vec![true, true, false];
        assert_eq!(bool_vec_to_string(bool_vec), "110".to_string());
    }
}
