//! Contains enum structures for different kinds of HCTL operators and formula components.

use std::fmt;

/// Enum for all possible unary operators occurring in a HCTL formula string.
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

/// Enum for all possible binary operators occurring in a HCTL formula string.
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

/// Enum for all possible hybrid operators occurring in a HCTL formula string.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum HybridOp {
    Bind,   // '!'
    Jump,   // '@'
    Exists, // '3'
    Forall, // 'V'
}

/// Enum for atomic sub-formulae: propositions, variables, and constants.
///
/// There are also *wild-card propositions*, which are immediately evaluated using
/// a precomputed coloured set. We differentiate them from classical propositions
/// since they are evaluated differently.
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
            UnaryOp::Ex => write!(f, "EX"),
            UnaryOp::Ax => write!(f, "AX"),
            UnaryOp::Ef => write!(f, "EF"),
            UnaryOp::Af => write!(f, "AF"),
            UnaryOp::Eg => write!(f, "EG"),
            UnaryOp::Ag => write!(f, "AG"),
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
            BinaryOp::Eu => write!(f, "EU"),
            BinaryOp::Au => write!(f, "AU"),
            BinaryOp::Ew => write!(f, "EW"),
            BinaryOp::Aw => write!(f, "AW"),
        }
    }
}

impl fmt::Display for HybridOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HybridOp::Bind => write!(f, "!"),
            HybridOp::Jump => write!(f, "@"),
            HybridOp::Exists => write!(f, "3"),
            HybridOp::Forall => write!(f, "V"),
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

impl From<bool> for Atomic {
    fn from(value: bool) -> Self {
        if value {
            Atomic::True
        } else {
            Atomic::False
        }
    }
}
