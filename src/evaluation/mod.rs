//! Components regarding the evaluation of formulae, including the main model-checking algorithm.

pub mod algorithm;
pub mod eval_info;
pub mod mark_duplicate_subform;
pub mod sanitizing;

mod canonization;
mod hctl_operators_evaluation;
mod low_level_operations;
