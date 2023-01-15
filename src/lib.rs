//! A small library regarding analysis of dynamic properties of Boolean networks through HCTL model checking.
//! As of now, the library supports:
//!  - Overall symbolic model-checking analysis of multiple HCTL formulae on a given model at once.
//!  - Manipulation with HCTL formulae, such as tokenizing, parsing, or canonization.
//!  - Searching for common sub-formulae across multiple formulae.
//!  - Optimised evaluation for several patterns, such as various attractor types or reachability.
//!

pub mod analysis;
pub mod bn_classification;
pub mod formula_evaluation;
pub mod formula_preprocessing;
pub mod model_checking;
pub mod result_print;

mod aeon;
