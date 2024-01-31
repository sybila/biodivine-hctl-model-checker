//! Print results of the computation, either aggregated version only, or a full set of satisfying states.

use biodivine_lib_param_bn::biodivine_std::bitvector::BitVector;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

use std::io::Write;
use std::time::SystemTime;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// Enum with options of the amount of printed results/details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrintOptions {
    /// Do not print anything at all.
    NoPrint,
    /// Print only the short summary at the end (numbers of results, time).
    JustSummary,
    /// Print the progress and details during the computation, and a summary at the end.
    WithProgress,
    /// Print the progress, and also exhaustively enumerate all satisfying states at the end.
    /// Note that this exhaustive result listing might be infeasible for large systems.
    Exhaustive,
}

/// Print the given text, but only if the correct print options are selected (long or full).
/// This simplifies the code regarding printing (no redundant if statements).
pub(crate) fn print_if_allowed(text: String, print_options: PrintOptions) {
    if print_options == PrintOptions::NoPrint || print_options == PrintOptions::JustSummary {
        return;
    }
    println!("{text}")
}

/// Print general info about the resulting set of colored vertices - the cardinality of the whole
/// set and its projections to colors and vertices (and the computation time).
pub(crate) fn summarize_results(
    formula: String,
    results: &GraphColoredVertices,
    start_time: SystemTime,
) {
    println!("Formula: {formula}");
    println!(
        "Time to model check: {}ms",
        start_time.elapsed().unwrap().as_millis()
    );
    println!("{} results in total", results.approx_cardinality());
    println!("{} unique colors", results.colors().approx_cardinality());
    println!("{} unique states", results.vertices().approx_cardinality());
    println!("-----");
}

/// Print the general info about the resulting set and then prints all states which are included
/// in the resulting set for at least one color (basically 'project out the colors' and print just
/// the states).
///
/// If param `show_names` is false, the states are displayed as a vector of 0/1; otherwise the full
/// proposition names are displayed.
pub(crate) fn print_results_full(
    formula: String,
    graph: &SymbolicAsyncGraph,
    results: &GraphColoredVertices,
    start_time: SystemTime,
    show_names: bool,
) {
    // first print general summarizing information
    summarize_results(formula, results, start_time);

    for valuation in results.vertices().materialize().iter() {
        // print either colored (green/red) variable literals in conjunction
        if show_names {
            let variable_name_strings = graph.variables().map(|id| graph.get_variable_name(id));

            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            for (i, var) in variable_name_strings.enumerate() {
                if valuation.get(i) {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                        .unwrap();
                    write!(&mut stdout, "{var} & ").unwrap();
                } else {
                    stdout
                        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                        .unwrap();
                    write!(&mut stdout, "~{var} & ").unwrap();
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
