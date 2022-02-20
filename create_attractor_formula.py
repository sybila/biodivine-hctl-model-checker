#!/usr/bin/env python3

def create_formula(data_set):
    # basic version without forbidding additional attractors
    formula = ""
    for item in data:
        if not item:
            break
        formula = formula + "(3{x}: ( @{x}: " + item + " & (!{y}: AG EF ({y} & " + item + " ) ) ) )" + " & "

    formula = formula + "true"

    # (optional) appendix for the formula which forbids additional attractors

    formula = formula + " & ~(3{x}:(@{x}: "
    for item in data:
        formula = formula + "~(AG EF ( " + item + ")) " + " & "
    formula = formula + "(!{y}: AG EF {y})))"


    return formula


if __name__ == '__main__':
    data = ["ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & ~CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & ~E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & ~MPP5 & ~Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & ~CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 & Maintenance_of_tight_junction_phenotype & ~Na__ion & ~PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            "ASIC1 & ~ASIC1_trimer_H__STOML3_complex & ~ATP1A_ATP1B_FXYDs_complex & ~Activity_of_sodium_channels_phenotype & ~BRD2 & ~BRD4 & CCNT1 & CDK9 & CRB3 & CRB3_PALS1_PATJ_complex_complex & ~Chromatin_organization_phenotype & E_PALS1_complex & E_cell & E_nucleus & H2A & H2BC21 & H3C1 & H3C15 & H4C1 & H4C14 & H4C9 & H4_16 & H__ion & JQ_1_simple_molecule & ~K__ion & MPP5 &Maintenance_of_tight_junction_phenotype & ~Na__ion & PATJ & ~P_TEFb_complex & ~RNA_Polymerase_II_dependent_Transcription__phenotype & STOML3 & ~TBP & ~csa1_histone_complex_nucleus & csa2_histone_complex_nucleus",
            ]
    attractor_formula = create_formula(data)
    print(attractor_formula)
