/// Creates the formula describing the (non)existence of reachability between two states (or partial)
/// `from_state` and `to_state` are both formulae describing particular states
/// `is_universal` is true iff we want all paths from `from_state` to reach `to_state`
/// `is_negative` is true iff we want to non-existence of path from `from_state` to `to_state`
#[allow(dead_code)]
pub fn create_reachability_formula(
    from_state: String,
    to_state: String,
    is_universal: bool,
    is_negative: bool,
) -> String {
    assert!(!(is_negative && is_universal));
    assert!(!to_state.is_empty() && !from_state.is_empty());
    if is_universal {
        return format!("(3{{x}}: (@{{x}}: {} & (AF ({}))))", from_state, to_state);
    }
    if is_negative {
        return format!("(3{{x}}: (@{{x}}: {} & (~EF ({}))))", from_state, to_state);
    }
    format!("(3{{x}}: (@{{x}}: {} & (EF ({}))))", from_state, to_state)
}

/// Creates the formula describing the existence of a particular trap space
/// trap space is a part of the state space from which we cannot escape
/// `trap_space` is a formula describing some proposition' values in a desired trap space
#[allow(dead_code)]
pub fn create_trap_space_formula(trap_space: String) -> String {
    assert!(!trap_space.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AG ({}))))", trap_space, trap_space)
}

/// Creates the formula describing the existence of specific attractor
/// `attractor_state` is a formula describing state in a desired attractor
#[allow(dead_code)]
pub fn create_attractor_formula(attractor_state: String) -> String {
    assert!(!attractor_state.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AG EF ({}))))", attractor_state, attractor_state)
}

/// Creates the formula prohibiting all but the given attractors
/// `attractor_state_set` is a vector of formulae, each describing a state in particular
/// allowed attractor
#[allow(dead_code)]
pub fn create_attractor_prohibition_formula(attractor_state_set: Vec<String>) -> String {
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ~(AG EF (");
    for attractor_state in attractor_state_set {
        assert!(!attractor_state.is_empty());
        formula.push_str(format!("({}) | ", attractor_state).as_str())
    }
    formula.push_str("false ))))"); // just we dont end with "|"
    formula
}

/// Creates the formula describing the existence of specific steady-state
/// `steady_state` is a formula describing particular desired fixed point
#[allow(dead_code)]
pub fn create_steady_state_formula(steady_state: String) -> String {
    assert!(!steady_state.is_empty());
    format!("(3{{x}}: (@{{x}}: {} & (AX ({}))))", steady_state, steady_state)
}

/// Creates the formula prohibiting all but the given steady-states
/// `steady_state_set` is a vector of formulae, each describing particular allowed fixed point
#[allow(dead_code)]
pub fn create_steady_state_prohibition_formula(steady_state_set: Vec<String>) -> String {
    let mut formula = String::new();
    formula.push_str("~(3{x}: (@{x}: ");
    for steady_state in steady_state_set {
        assert!(!steady_state.is_empty());
        formula.push_str(format!("~({}) & ", steady_state).as_str())
    }
    formula.push_str("(AX {x})))");
    formula
}

/// Creates the formula ensuring the existence of given fixed points and
/// prohibiting all other steady-states (uses advantage of cashing "AX x")
/// `steady_state_set` is a vector of formulae, each describing particular desired fixed point
#[allow(dead_code)]
pub fn create_steady_state_formula_both(steady_state_set: Vec<String>) -> String {
    // part which ensures steady states
    let mut formula = String::new();
    for steady_state in steady_state_set.clone() {
        assert!(!steady_state.is_empty());
        formula.push_str(format!("(3{{x}}: (@{{x}}: {} & (AX {{x}}))) & ", steady_state).as_str())
    }

    // appendix which forbids additional steady states
    formula.push_str("~(3{x}: (@{x}: ");
    for steady_state in steady_state_set {
        formula.push_str(format!("~( {} )  & ", steady_state).as_str())
    }
    formula.push_str("(AX {x})))");
    formula
}
