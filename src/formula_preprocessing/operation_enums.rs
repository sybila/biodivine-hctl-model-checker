use std::fmt;

/// Enum for all possible unary operators occurring in a HCTL formula string
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum UnaryOp {
    Not, // '~'
    Ex,  // 'EX'
    Ax,  // 'AX'
    Ef,  // 'EF'
    Af,  // 'AF'
    Eg,  // 'EG'
    Ag,  // 'AG'
}

/// Enum for all possible binary operators occurring in a HCTL formula string
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum BinaryOp {
    And, // '&'
    Or,  // '|'
    Xor, // '^'
    Imp, // '=>'
    Iff, // '<=>'
    Eu,  // 'EU'
    Au,  // 'AU'
    Ew,  // 'EW'
    Aw,  // 'AW'
}

/// Enum for all possible hybrid operators occurring in a HCTL formula string
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum HybridOp {
    Bind,   // '!'
    Jump,   // '@'
    Exists, // '3'
    Forall, // 'V'
}

/// Enum for atomic formulas - propositions, variables, constants
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Atomic {
    Prop(String), // A proposition name
    Var(String),  // A variable name
    True,         // A true constant
    False,        // A false constant
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "~"),
            c => write!(f, "{}", format!("{:?}", c)),
        }
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryOp::And => write!(f, "&"),
            BinaryOp::Or => write!(f, "|"),
            BinaryOp::Xor => write!(f, "^"),
            BinaryOp::Imp => write!(f, "=>"),
            BinaryOp::Iff => write!(f, "<=>"),
            c => write!(f, "{}", format!("{:?}", c)),
        }
    }
}

impl fmt::Display for HybridOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            op => write!(f, "{}", format!("{:?}", op)),
        }
    }
}

impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atomic::Var(name) => write!(f, "{{{}}}", name),
            Atomic::Prop(name) => write!(f, "{}", name),
            Atomic::True => write!(f, "True"),
            Atomic::False => write!(f, "False"),
        }
    }
}
