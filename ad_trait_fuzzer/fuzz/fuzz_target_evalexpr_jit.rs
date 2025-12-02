// fuzz/fuzz_target_evalexpr_jit.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::env;
use std::error::Error;

use fuzz_core::input_decoder::{GeneralInputDecoder, FuzzInputDecoder};
use fuzz_core::ast_evaluator::unified::EvalexprPyUnified;
use fuzz_core::ast_generator::{generate_from_bytes, AstGenConfig};
use fuzz_core::fuzz_harness::PyTorchComputable;
use fuzz_core::oracles::{EvalexprVsPyTorchCheck, GroundTruth};
use tch::{Tensor, Kind};

const NUM_GENERATED_TESTS: usize = 1;

fuzz_target!(|data: &[u8]| {
    let ast_config = {
        let max_depth = env::var("AST_MAX_DEPTH").ok().and_then(|s| s.parse().ok()).unwrap_or(4);
        let allow_division = env::var("AST_ALLOW_DIVISION").map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(true);
        let allow_power = env::var("AST_ALLOW_POWER").map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(true);
        let allow_log = env::var("AST_ALLOW_LOG").map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(false);
        let max_variables = env::var("AST_MAX_VARIABLES").ok().and_then(|s| s.parse().ok()).unwrap_or(2);

        AstGenConfig { max_depth, max_variables, allow_division, allow_power, allow_log }
    };

    let num_variables = ast_config.max_variables;
    let input_decoder = GeneralInputDecoder { input_length: num_variables };
    let min_data_size = num_variables * 8;
    if data.len() < min_data_size {
        return;
    }
    let inputs: Vec<f64> = match input_decoder.decode(&data[0..min_data_size]) {
        Ok(inputs) => inputs,
        Err(_) => return,
    };
    
    // TODO: make all arbitrary inputs finite and reasonable
    for &val in &inputs {
        if !val.is_finite() || val.abs() > 1e10 {
            return;
        }
    }

    let ast_data = &data[min_data_size..];
    let mut evaluators = Vec::new();
    let mut used_vars_list = Vec::new();
    for i in 0..NUM_GENERATED_TESTS {
        let offset = i * 32;
        let test_data = if offset < ast_data.len() { &ast_data[offset..] } else { ast_data };
        if let Ok(generated_expr) = generate_from_bytes(test_data, ast_config.clone()) {
            if let Ok(evaluator) = EvalexprPyUnified::new(generated_expr.expr, generated_expr.num_inputs) {
                used_vars_list.push(generated_expr.num_inputs);
                evaluators.push(evaluator);
            }
        }
    }
    if evaluators.is_empty() {
        return;
    }
    let oracle = EvalexprVsPyTorchCheck::new();
    for (evaluator, num_inputs) in evaluators.iter().zip(used_vars_list.iter()) {
        if *num_inputs == 0 {
            continue;
        }
        let test_inputs = &inputs[..*num_inputs];
        let mut tensors: Vec<Tensor> = Vec::new();
        for &val in test_inputs {
            tensors.push(Tensor::from(val).set_requires_grad(true).to_kind(Kind::Double));
        }
        let outputs = evaluator.compute_pytorch(&tensors).unwrap_or_default();
        if outputs.is_empty() || outputs[0].numel() != 1 {
            continue;
        }
        if !outputs[0].requires_grad() {
            continue;
        }
        outputs[0].backward();
        let mut pytorch_jacobian = Vec::new();
        for tensor in &tensors {
            let grad_tensor = tensor.grad();
            let grad = if grad_tensor.numel() > 0 { grad_tensor.double_value(&[]) } else { 0.0 };
            pytorch_jacobian.push(grad);
        }
        let ground_truth = GroundTruth { name: "PyTorch", jacobian: pytorch_jacobian };
        if let Err(e) = oracle.check_all(evaluator.evalexpr(), test_inputs, &[ground_truth]) {
            eprintln!("\n=== CRASH DETECTED ===");
            eprintln!("Expression that caused the mismatch:");
            eprintln!("  {}", evaluator.expr_string());
            eprintln!("\nInputs:");
            for (i, &val) in test_inputs.iter().enumerate() {
                eprintln!("  x_{}: {}", i, val);
            }
            eprintln!("\nError: {}", e);
            eprintln!("======================\n");
            panic!("Derivative mismatch: {}", e);
        }
    }
});
