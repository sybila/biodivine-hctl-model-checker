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
        We can create formula in two ways - components are either "state & (AG EF state)" or "state & (!y: AG EF y)"
        The first should be faster, but if we want to forbid other attractors, we must compute
        the formula "!y: AG EF y" anyway and we can then just cache it
        */
        if forbid_extra_attr {
            formula.push_str(
                format!("(3{{x}}: (@{{x}}: {} & (!{{y}}: AG EF {{y}}))) & ", item).as_str()
            )
        }
        else {
            formula.push_str(
                format!("(3{{x}}: (@{{x}}: {} & (AG EF ({})))) & ", item, item).as_str()
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
    if args.len() != 4 {
        println!("3 arguments expected, got {}", args.len() - 1);
        println!("Usage: ./infer_networks model_file attractor_data forbid_extra_attrs");
        return;
    }
    if !(args[3].as_str() == "false" ||  args[3].as_str() == "true") {
        println!("Invalid argument \"{}\", it must be either \"true\" or \"false\"", args[3]);
        println!("Usage: ./infer_networks model_file attractor_data (true | false)");
        return;
    }
    let forbid_extra_attrs = match args[3].as_str() {
        "false" => false,
        _ => true  // we need match to be exhaustive
    };

    let data_file = File::open(Path::new(args[2].as_str())).unwrap();
    let reader = BufReader::new(&data_file);
    let data: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let formula = create_attractor_formula(data, forbid_extra_attrs);
    //let formula = create_steady_state_formula(data, forbid_extra_attrs);

    println!("original formula: {}", formula.clone());

    let aeon_string = read_to_string(args[1].clone()).unwrap();

    analyze_property(aeon_string, formula, false);
    // result should have 2^(number of vars) states - basically all states
}
