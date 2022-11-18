pub mod analysis;
pub mod result_print;

/// HCTL formula preprocessing (parsing, tokenizing)
pub mod formula_preprocessing;

/// HCTL formula evaluation (main recursive algorithm, cache)
pub mod formula_evaluation;

/// modified SCC computation algorithms adapted from Aeon
pub mod aeon;
