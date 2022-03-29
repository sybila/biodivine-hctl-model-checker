use hctl_model_checker::analysis::analyze_property;
use std::env;
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

#[allow(dead_code)]
fn create_attractor_formula(data_set: Vec<String>, forbid_extra_attr: bool) -> String {
    // basic version without forbidding additional attractors
    let mut formula = String::new();
    for item in data_set.clone() {
        if item.is_empty() {
            continue;
        }

        /*
        We can create formula in two ways - components are either "!y: AG EF y" or "!y: AG EF (y and state)"
        The latter is usually faster, but if we want to forbid other attractors, we must compute
        the formula "!y: AG EF y" anyway and we can then just cache it
        */
        if forbid_extra_attr {
            formula.push_str(
                format!("(3{{x}}: (@{{x}}: {} & (!{{y}}: AG EF {{y}} ))) & ", item).as_str()
            )
        }
        else {
            formula.push_str(
                format!("(3{{x}}: (@{{x}}: {} & (!{{y}}: AG EF ({{y}} & {} )))) & ", item, item).as_str()
            )
        }
    }
    formula.push_str("true"); // just so we dont end with "&"

    // (optional) appendix for the formula which forbids additional attractors
    if forbid_extra_attr {
        formula.push_str(" & ~(3{x}: (@{x}: ");
        for item in data_set {
            if item.is_empty() {
                continue;
            }
            formula.push_str(format!("~(AG EF ( {} ))  & ", item).as_str())
        }
        formula.push_str("(!{y}: AG EF {y})))")
    }

    formula
}

#[allow(dead_code)]
fn create_steady_state_formula(data_set: Vec<String>, forbid_extra_attr: bool) -> String {
    // basic version without forbidding additional attractors
    let mut formula = String::new();
    for item in data_set.clone() {
        if item.is_empty() {
            continue;
        }
        formula.push_str(format!("(3{{x}}: (@{{x}}: {} & (!{{y}}: AX ({{y}})))) & ", item).as_str())
    }
    formula.push_str("true");

    // (optional) appendix for the formula which forbids additional attractors
    if forbid_extra_attr {
        formula.push_str(" & ~(3{x}: (@{x}: ");
        for item in data_set {
            if item.is_empty() {
                continue;
            }
            formula.push_str(format!("~( {} )  & ", item).as_str())
        }
        formula.push_str("(!{y}: AX {y})))")
    }

    formula
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("2 arguments expected, got {}", args.len() - 1);
        println!("Usage: ./infer_networks model_file attractor_data");
        return;
    }

    let data_file = File::open(Path::new(args[2].as_str())).unwrap();
    let reader = BufReader::new(&data_file);
    let data: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let formula = create_attractor_formula(data, false);
    //let formula = create_steady_state_formula(data, false);

    println!("original formula: {}", formula.clone());

    let aeon_string = read_to_string(args[1].clone()).unwrap();

    analyze_property(aeon_string, formula, false);
}
