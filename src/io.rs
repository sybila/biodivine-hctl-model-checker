use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

use std::fs::File;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Prints general info about result set - cardinality of the set and color/vertex projections
#[allow(dead_code)]
pub fn print_results_fast(results: &GraphColoredVertices) -> () {
    println!("{} results in total", results.approx_cardinality());
    println!("{} colors", results.colors().approx_cardinality());
    println!("{} states", results.vertices().approx_cardinality());
}

/// Prints the general info about the resulting set and also all the contained items
/// If param `show_names` is true, full proposition names are displayed (otherwise 0/1 only)
#[allow(dead_code)]
pub fn print_results(
    graph: &SymbolicAsyncGraph,
    results: &GraphColoredVertices,
    show_names: bool,
) -> () {
    // first print general info
    print_results_fast(results);

    let mut counter = 0;
    let network = graph.as_network();
    for valuation in results.vertices().materialize().iter() {
        // colored var names version
        if show_names {
            let mut i = 0;
            let variable_name_strings = network
                .variables()
                .map(|id| format!("{}", network.get_variable_name(id)));

            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            for var in variable_name_strings {
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
                i += 1;
            }
            stdout
                .set_color(ColorSpec::new().set_fg(Some(Color::White)))
                .unwrap();
            println!();
        }
        // just 0/1 valuation vector version
        else {
            let mut valuation_str = String::new();
            for j in 0..valuation.len() {
                valuation_str.push(if valuation.get(j) { '1' } else { '0' });
            }
            println!("{}", valuation_str.as_str());
        }
        counter += 1;
    }
    println!("{} result states found in total.", counter)
}

/// write 0/1 vectors for all states from the given set to the given file
#[allow(dead_code)]
pub fn write_states_to_file(mut file: &File, set_of_states: &GraphColoredVertices) -> () {
    write!(file, "{}\n", set_of_states.vertices().approx_cardinality()).unwrap();
    for valuation in set_of_states.vertices().materialize().iter() {
        let mut valuation_str = String::new();
        for j in 0..valuation.len() {
            valuation_str.push(if valuation.get(j) { '1' } else { '0' });
        }
        valuation_str.push('\n');
        file.write_all(valuation_str.as_bytes()).unwrap();
    }
    file.write_all("--------------\n".as_bytes()).unwrap();
}
