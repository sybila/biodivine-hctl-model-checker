use hctl_model_checker::evaluator::eval_tree;
#[allow(unused_imports)]
use hctl_model_checker::io::{print_results, print_results_fast};
use hctl_model_checker::parser::parse_hctl_formula;
#[allow(unused_imports)]
use hctl_model_checker::tokenizer::{print_tokens, tokenize_recursive};

use std::convert::TryFrom;
use std::fs::read_to_string;
use std::time::SystemTime;

use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use biodivine_lib_param_bn::BooleanNetwork;

/* TODOs to implement for the model checking part */
// TODO: USE PROPER DUPLICATE MARKING AND IMPLEMENT PROPER CACHE FOR EVALUATOR
// TODO: SPECIAL CASES FOR EVALUATOR (attractors, stable states...)
// TODO: optims for evaluator
// TODO: separate function
// TODO: iterator for GraphColoredVertices sets - we only have for vertices (or something like that)
// TODO: more efficient operators on GraphColoredVertices (like imp, xor, equiv)?
// TODO: printer for all correct valuations in all three color/vertex sets
// TODO: documentation

/* BUGs to fix */
// TODO: formula 4 from TACAS and CAV does not work?
// TODO: AF !{x}: (AX (~{x} & AF {x})) does not work - parses as (Bind {x}: (Ax ((~ {x}) & (Af {x}))))

// TODO: "!{var}: AG EF {var} & & !{var}: AG EF {var}" DOES NOT CAUSE ERROR
// TODO: "!{var}: AG EF {var} & !{var}: AG EF {var}" DOES NOT PARSE CORRECTLY
// TODO: check that formula doesnt contain stuff like "!x (EF (!x x)) - same var quantified more times

/* TODOs to implement for the inference part */
// TODO: parse attractors from file
// TODO: implement both approaches (model-checking vs component-wise)
// TODO: create separate binaries
// TODO: printing satisfying BNs? or do something with the resulting colors

fn analyze_property(model_file_path: String, formula: String, print_all: bool) {
    let start = SystemTime::now();

    let tokens = match tokenize_recursive(&mut formula.chars().peekable(), true) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", e);
            Vec::new()
        }
    };
    //print_tokens(&tokens);

    match parse_hctl_formula(&tokens) {
        Ok(tree) => {
            println!("original formula: {}", tree.subform_str);
            let aeon_string = read_to_string(model_file_path).unwrap();
            let bn = BooleanNetwork::try_from(aeon_string.as_str()).unwrap();
            let graph = SymbolicAsyncGraph::new(bn).unwrap();

            println!(
                "Graph build time: {}ms",
                start.elapsed().unwrap().as_millis()
            );

            let result = eval_tree(tree, &graph);
            //write_attractors_to_file(&graph, "attractor_output.txt");

            println!("Eval time: {}ms", start.elapsed().unwrap().as_millis());
            println!("{} vars in network", graph.as_network().num_vars());

            if print_all {
                print_results(&graph, &result, true);
            } else {
                print_results_fast(&result);
            }
        }
        Err(message) => println!("{}", message),
    }
}

fn main() {
    let formula = "!{x}: AG EF {x}".to_string();
    //let formula = "(3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & (3{x}: ( @{x}: ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 &Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus & (!{y}: AG EF ({y} & ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 &Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus ) ) ) ) & true".to_string();
    let model_file = "test_model.aeon".to_string();
    analyze_property(model_file, formula, false);
}

#[cfg(test)]
mod tests {
    use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
    use biodivine_lib_param_bn::BooleanNetwork;
    use hctl_model_checker::evaluator::eval_tree;
    use hctl_model_checker::parser::parse_hctl_formula;
    use hctl_model_checker::tokenizer::tokenize_recursive;

    const BNET_MODEL: &str = r"
targets,factors
Cdc25, ((!Cdc2_Cdc13 & (Cdc25 & !PP)) | ((Cdc2_Cdc13 & (!Cdc25 & !PP)) | (Cdc2_Cdc13 & Cdc25)))
Cdc2_Cdc13, (!Ste9 & (!Rum1 & !Slp1))
Cdc2_Cdc13_A, (!Ste9 & (!Rum1 & (!Slp1 & (!Wee1_Mik1 & Cdc25))))
PP, Slp1
Rum1, ((!SK & (!Cdc2_Cdc13 & (!Rum1 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Rum1 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Rum1 & (!Cdc2_Cdc13_A & PP))))))))
SK, Start
Slp1, Cdc2_Cdc13_A
Start, 0
Ste9, ((!SK & (!Cdc2_Cdc13 & (!Ste9 & (!Cdc2_Cdc13_A & PP)))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & !Cdc2_Cdc13_A))) | ((!SK & (!Cdc2_Cdc13 & (Ste9 & (Cdc2_Cdc13_A & PP)))) | ((!SK & (Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP)))) | (SK & (!Cdc2_Cdc13 & (Ste9 & (!Cdc2_Cdc13_A & PP))))))))
Wee1_Mik1, ((!Cdc2_Cdc13 & (!Wee1_Mik1 & PP)) | ((!Cdc2_Cdc13 & Wee1_Mik1) | (Cdc2_Cdc13 & (Wee1_Mik1 & PP))))
";

    #[test]
    fn basic_formulas() {
        fn check_formula(formula: String, stg: &SymbolicAsyncGraph) -> GraphColoredVertices {
            let tokens = tokenize_recursive(&mut formula.chars().peekable(), true).unwrap();
            let tree = parse_hctl_formula(&tokens).unwrap();
            eval_tree(tree, stg)
        }

        let bn = BooleanNetwork::try_from_bnet(BNET_MODEL).unwrap();
        let stg = SymbolicAsyncGraph::new(bn).unwrap();

        let mut result = check_formula("!{x}: AG EF {x}".to_string(), &stg);
        assert_eq!(76., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(76., result.vertices().approx_cardinality());

        result = check_formula("!{x}: AX {x}".to_string(), &stg);
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());

        result = check_formula("!{x}: AX EF {x}".to_string(), &stg);
        assert_eq!(132., result.approx_cardinality());
        assert_eq!(2., result.colors().approx_cardinality());
        assert_eq!(132., result.vertices().approx_cardinality());

        result = check_formula("AF (!{x}: AX {x})".to_string(), &stg);
        assert_eq!(60., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(60., result.vertices().approx_cardinality());

        result = check_formula(
            "!{x}: 3{y}: (@{x}: ~{y} & AX {x}) & (@{y}: AX {y})".to_string(),
            &stg,
        );
        assert_eq!(12., result.approx_cardinality());
        assert_eq!(1., result.colors().approx_cardinality());
        assert_eq!(12., result.vertices().approx_cardinality());
    }
}
