#!/usr/bin/env python3

def create_formula(data_set):
    # basic version without forbidding additional attractors
    formula = ""
    for item in data:
        if not item:
            break
        formula = formula + "(3{x}: ( @{x}: " + item + " & (!{y}: AG EF ({y} & " + item + " ) ) ) )" + " & "

    formula = formula + "True"

    # (optional) appendix for the formula which forbids additional attractors
    """
    formula = formula + "~(3{x}:(@{x}: "
    for item in data:
        formula = formula + "~(AG EF ( " + item + ")) " + " & "
    formula = formula + "(!{y}: AG EF {y})))"
    """

    return formula


if __name__ == '__main__':
    data = ["~ARF & AUXIAA & ~AUXIN & ~JKD & ~MGP & ~PLT & ~SCR & ~SHR & ~WOX5",
            "~ARF & AUXIAA & ~AUXIN & ~JKD & ~MGP & ~PLT & ~SCR & SHR & ~WOX5",
            "~ARF & AUXIAA & ~AUXIN & JKD & MGP & ~PLT & SCR & SHR & ~WOX5",
            "ARF & ~AUXIAA & AUXIN & ~JKD & ~MGP & PLT & ~SCR & ~SHR & ~WOX5",
            "ARF & ~AUXIAA & AUXIN & ~JKD & ~MGP & PLT & ~SCR & SHR & ~WOX5",
            "ARF & ~AUXIAA & AUXIN & JKD & ~MGP & PLT & SCR & SHR & WOX5",
            "ARF & ~AUXIAA & AUXIN & JKD & MGP & PLT & SCR & SHR & ~WOX5"]
    attractor_formula = create_formula(data)
    print(attractor_formula)
