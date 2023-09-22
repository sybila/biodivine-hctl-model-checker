//! A small library regarding analysis of dynamic properties of Boolean networks through HCTL model checking.
//! As of now, the library supports:
//!  - Model-checking analysis of HCTL properties on a (partially specified) BNs.
//!  - Various formulae preprocessing utilities, such as tokenizing, parsing, or some canonization.
//!  - Manipulation with abstract syntactic trees for HCTL formulae.
//!  - Searching for common sub-formulae across multiple properties.
//!  - Optimised evaluation for several patterns, such as various attractor types or reachability.
//!  - Simultaneous evaluation of several formulae, sharing common computation via cache.
//!

pub mod analysis;
pub mod evaluation;
pub mod mc_utils;
pub mod model_checking;
pub mod postprocessing;
pub mod preprocessing;
pub mod result_print;

mod aeon;
