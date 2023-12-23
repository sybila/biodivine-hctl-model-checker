//! Contains enum structures for different kinds of HCTL operators and formula components.

use std::fmt;

/// Enum for all possible unary operators occurring in a HCTL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum UnaryOp {
    Not, // '~'
    EX,  // 'EX'
    AX,  // 'AX'
    EF,  // 'EF'
    AF,  // 'AF'
    EG,  // 'EG'
    AG,  // 'AG'
}

/// Enum for all possible binary operators occurring in a HCTL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum BinaryOp {
    And, // '&'
    Or,  // '|'
    Xor, // '^'
    Imp, // '=>'
    Iff, // '<=>'
    EU,  // 'EU'
    AU,  // 'AU'
    EW,  // 'EW'
    AW,  // 'AW'
}

/// Enum for all possible hybrid operators occurring in a HCTL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum HybridOp {
    Bind,   // '!'
    Jump,   // '@'
    Exists, // '3'
    Forall, // 'V'
}

/// Enum for atomic sub-formulae - propositions, variables, and constants.
/// There are also `wild-card propositions`, that will be directly evaluated as some precomputed
/// coloured set. We have to differentiate them from classical propositions.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Atomic {
    Prop(String),         // A proposition name
    Var(String),          // A variable name
    True,                 // A true constant
    False,                // A false constant
    WildCardProp(String), // A wild-card proposition name
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "~"),
            // temporal operators are displayed as they are
            c => write!(f, "{c:?}"),
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
            // temporal operators are displayed as they are
            c => write!(f, "{c:?}"),
        }
    }
}

impl fmt::Display for HybridOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HybridOp::Bind => write!(f, "!"),
            HybridOp::Exists => write!(f, "3"),
            HybridOp::Forall => write!(f, "V"),
            HybridOp::Jump => write!(f, "@"),
        }
    }
}

impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Atomic::Var(name) => write!(f, "{{{name}}}"),
            Atomic::Prop(name) => write!(f, "{name}"),
            Atomic::True => write!(f, "True"),
            Atomic::False => write!(f, "False"),
            Atomic::WildCardProp(name) => write!(f, "%{name}%"),
        }
    }
}
