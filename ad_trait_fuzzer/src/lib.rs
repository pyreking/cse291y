// src/lib.rs

//! Core library for the Automatic Differentiation (AD) fuzzing harness.
//!
//! This crate contains all the modular components for:
//! 1. Decoding fuzzer input bytes.
//! 2. Defining and generating test cases (RPN expressions).
//! 3. Evaluating test cases using various AD types.
//! 4. Calculating ground truth derivatives (via PyTorch).
//! 5. Running and comparing results via a set of Oracles.

pub mod input_decoder;
pub mod oracles;
pub mod fuzz_harness;
pub mod gt_calculators;
pub mod test_definition;
pub mod rpn_evaluator;
pub mod test_generator;