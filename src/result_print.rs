//! Print results of the computation, either aggregated version only, or a full set of satisfying states.

use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::symbolic_async_graph::{
    GraphColoredVertices, GraphColors, SymbolicAsyncGraph,
};

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrintOptions {
    NoPrint,
    ShortPrint,
    MediumPrint,
    FullPrint,
}

/// Print the given text, but only if the correct print options are selected (long or full).
/// This simplifies the code regarding printing (no redundant if statements).
pub(crate) fn print_if_allowed(text: String, print_options: PrintOptions) {
    if print_options == PrintOptions::NoPrint || print_options == PrintOptions::ShortPrint {
        return;
    }
    println!("{}", text)
}

/// Print general info about the resulting set of colored vertices - the cardinality of the whole
/// set and its projections to colors and vertices (and the computation time).
pub(crate) fn summarize_results(results: &GraphColoredVertices, start_time: SystemTime) {
    println!(
        "Time to eval formula: {}ms",
        start_time.elapsed().unwrap().as_millis()
    );
    println!("{} results in total", results.approx_cardinality());
    println!("{} unique colors", results.colors().approx_cardinality());
    println!("{} unique states", results.vertices().approx_cardinality());
    println!("-----");
}

/// Print the general info about the resulting set and then prints all states which are included
/// in the resulting set for at least one color.
/// If param `show_names` is false, the states are displayed as a vector of 0/1; otherwise the full
/// proposition names are displayed.
pub(crate) fn print_results_full(
    graph: &SymbolicAsyncGraph,
    results: &GraphColoredVertices,
    start_time: SystemTime,
    show_names: bool,
) {
    // first print general summarizing information
    summarize_results(results, start_time);

    let network = graph.as_network();
    for valuation in results.vertices().materialize().iter() {
        // print either colored (green/red) variable literals in conjunction
        if show_names {
            let variable_name_strings = network
                .variables()
                .map(|id| network.get_variable_name(id).to_string());

            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            for (i, var) in variable_name_strings.enumerate() {
                if valuation.get(i) {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                        .unwrap();
                    write!(&mut stdout, "{} & ", var).unwrap();
                } else {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                        .unwrap();
                    write!(&mut stdout, "~{} & ", var).unwrap();
                }
            }
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::White)))
                .unwrap();
            println!();
        }
        // print just Boolean valuation vector of 0/1
        else {
            let mut valuation_str = String::new();
            for j in 0..valuation.len() {
                valuation_str.push(if valuation.get(j) { '1' } else { '0' });
            }
            println!("{}", valuation_str.as_str());
        }
    }
    println!("-----");
}

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

        // dump corresponding BDD
        let bdd_file = format!("bdd_dump_{}.txt", bool_vec_to_string(bool_indices));
        let bdd_file_path = PathBuf::from(result_dir).join(bdd_file);
        let mut bdd_file = File::create(bdd_file_path).unwrap();
        category_colors
            .as_bdd()
            .write_as_string(&mut bdd_file)
            .unwrap();
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::result_print::{bool_vec_to_string, int_to_bool_vec};

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
