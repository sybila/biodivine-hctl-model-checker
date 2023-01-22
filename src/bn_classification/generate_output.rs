//! Generate results of the computation (report and BDD representation).

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColors;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
/// encoding the colors
///
/// `all_valid_colors` represents a "unit color set" - all colors satisfying the assertion formulae
/// Each result corresponds to colors satisfying some property formula
/// Each category is given by the set of colors that satisfy exactly the same properties
pub fn write_class_report_and_dump_bdds(
    assertion_formulae: &Vec<String>,
    all_valid_colors: GraphColors,
    property_formulae: &[String],
    property_results: &Vec<GraphColors>,
    result_dir: &str,
) {
    let report_file_path = PathBuf::from(result_dir).join("report.txt");
    let mut report_file = File::create(report_file_path).unwrap();

    writeln!(report_file, "### Assertion formulae\n").unwrap();
    for assertion_formula in assertion_formulae {
        writeln!(report_file, "# {}", assertion_formula).unwrap();
    }
    writeln!(
        report_file,
        "{} colors satisfy all assertions\n",
        all_valid_colors.approx_cardinality()
    )
    .unwrap();

    writeln!(report_file, "### Property formulae individually\n").unwrap();
    for (i, property_formula) in property_formulae.iter().enumerate() {
        writeln!(report_file, "# {}", property_formula).unwrap();
        writeln!(
            report_file,
            "{} colors satisfy this property\n",
            property_results[i].approx_cardinality()
        )
        .unwrap();
    }

    writeln!(report_file, "### Categories\n").unwrap();
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

        writeln!(
            report_file,
            "# {}",
            bool_vec_to_string(bool_indices.clone())
        )
        .unwrap();
        writeln!(
            report_file,
            "{} colors in this category\n",
            category_colors.approx_cardinality()
        )
        .unwrap();

        // save the corresponding BDD, if the set is not empty
        if category_colors.approx_cardinality() > 0. {
            // create the file to dump BDD
            let bdd_file = format!("bdd_dump_{}.txt", bool_vec_to_string(bool_indices.clone()));
            let bdd_file_path = PathBuf::from(result_dir).join(bdd_file);
            let mut bdd_file = File::create(bdd_file_path).unwrap();

            /*
            // write annotation
            writeln!(
                bdd_file,
                "# {}",
                bool_vec_to_string(bool_indices)
            )
            .unwrap();
            */

            // dump corresponding BDD
            category_colors
                .as_bdd()
                .write_as_string(&mut bdd_file)
                .unwrap();
        }

        i += 1;
    }
}

/// Write a short summary regarding the computation where assertions were not satisfied
pub fn write_empty_report(
    assertion_formulae: &Vec<String>,
    result_dir: &str,
) {
    let report_file_path = PathBuf::from(result_dir).join("report.txt");
    let mut report_file = File::create(report_file_path).unwrap();

    writeln!(report_file, "### Assertion formulae\n").unwrap();
    for assertion_formula in assertion_formulae {
        writeln!(report_file, "# {}", assertion_formula).unwrap();
    }
    writeln!(report_file, "0 colors satisfy all assertions\n").unwrap()
}

#[cfg(test)]
mod tests {
    use crate::bn_classification::generate_output::{bool_vec_to_string, int_to_bool_vec};

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
