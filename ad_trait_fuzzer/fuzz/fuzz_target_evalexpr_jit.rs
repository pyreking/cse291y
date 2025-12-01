// fuzz/fuzz_target_evalexpr_jit.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::env;
use std::error::Error;

use fuzz_core::input_decoder::{GeneralInputDecoder, FuzzInputDecoder};
use fuzz_core::ast_evaluator::{EvalexprEvaluator, PyTorchEvaluator, InfixPrinter};
use fuzz_core::ast_generator::{generate_from_bytes, AstGenConfig};
use fuzz_core::fuzz_harness::PyTorchComputable;
use fuzz_core::ast_expr::Expr;
use tch::{Tensor, Kind};

const NUM_GENERATED_TESTS: usize = 1;
const ABS_TOLERANCE: f64 = 1e-12;
const REL_TOLERANCE: f64 = 1e-9;

// --- Configuration Structures ---

#[derive(Clone, Debug)]
enum HarnessMode {
    PanicOnFirstError,
    Continuous,
}

#[derive(Clone, Debug)]
struct FuzzConfig {
    mode: HarnessMode,
    num_generated_tests: usize,
}

fn get_fuzz_config() -> FuzzConfig {
    let mode = match env::var("FUZZ_MODE") {
        Ok(val) if val.eq_ignore_ascii_case("continuous") => HarnessMode::Continuous,
        _ => HarnessMode::PanicOnFirstError,
    };

    let num_generated_tests = match env::var("FUZZ_TESTS") {
        Ok(val) => val.parse::<usize>().unwrap_or(NUM_GENERATED_TESTS),
        _ => NUM_GENERATED_TESTS,
    };

    FuzzConfig {
        mode,
        num_generated_tests,
    }
}


fn get_ast_config() -> AstGenConfig {
    let max_depth = env::var("AST_MAX_DEPTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    
    let allow_division = env::var("AST_ALLOW_DIVISION")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    
    let allow_power = env::var("AST_ALLOW_POWER")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(true);
    
    let allow_log = env::var("AST_ALLOW_LOG")
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);  // Disable by default

    let max_variables = env::var("AST_MAX_VARIABLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    AstGenConfig {
        max_depth,
        max_variables,
        allow_division,
        allow_power,
        allow_log,
    }
}



fn compute_pytorch_gradients(
    pytorch_eval: &PyTorchEvaluator<()>,
    inputs: &[f64],
) -> Result<Vec<f64>, Box<dyn Error>> {
    let mut tensors: Vec<Tensor> = Vec::new();
    for &val in inputs {
        tensors.push(
            Tensor::from(val)
                .set_requires_grad(true)
                .to_kind(Kind::Double)
        );
    }
    
    let outputs = pytorch_eval.compute_pytorch(&tensors)?;
    if outputs.is_empty() {
        return Err("PyTorch function returned no output.".into());
    }
    
    if outputs[0].numel() != 1 {
        return Err("PyTorch output is not a scalar, skipping derivative calculation.".into());
    }

    if !outputs[0].requires_grad() {
        let zero_gradients = vec![0.0; inputs.len()];
        return Ok(zero_gradients);
    }

    outputs[0].backward();
    
    let mut gradients = Vec::new();
    for tensor in &tensors {
        let grad_tensor = tensor.grad();
        let grad = if grad_tensor.numel() > 0 {
            grad_tensor.double_value(&[])
        } else {
            0.0
        };
        gradients.push(grad);
    }
    
    Ok(gradients)
}


const ABS_TOLERANCE: f64 = 1e-12;
const REL_TOLERANCE: f64 = 1e-9;

fn compare_derivatives(
    evalexpr_grad: f64,
    pytorch_grad: f64,
    var_index: usize,
) -> Result<(), String> {
    if !pytorch_grad.is_finite() {
        return Ok(());
    }

    let diff = (evalexpr_grad - pytorch_grad).abs();
    
    let scaled_rel_threshold = pytorch_grad.abs() * REL_TOLERANCE;
    let threshold = ABS_TOLERANCE.max(scaled_rel_threshold);
    
    if diff > threshold {
        return Err(format!(
            "Derivative mismatch for x_{}: evalexpr-jit = {}, PyTorch = {}, diff = {} (threshold = {})",
            var_index, evalexpr_grad, pytorch_grad, diff, threshold
        ));
    }
    
    Ok(())
}


fuzz_target!(|data: &[u8]| {
    let config: FuzzConfig = get_fuzz_config();
    
    let ast_config = get_ast_config();
    let num_variables = ast_config.max_variables;

    let input_decoder: GeneralInputDecoder = GeneralInputDecoder { input_length: num_variables };

    let min_data_size = num_variables * 8;

    if data.len() < min_data_size {
        return;
    }

    let inputs: Vec<f64> = match input_decoder.decode(&data[0..min_data_size]) {
        Ok(inputs) => inputs,
        Err(_) => return,
    };
    
    for &val in &inputs {
        if !val.is_finite() || val.abs() > 1e10 {
            return;
        }
    }
    
    let ast_data = &data[min_data_size..];
    
    let mut evalexpr_evaluators = Vec::new();
    let mut pytorch_evaluators = Vec::new();
    let mut used_vars_list = Vec::new();
    
    for i in 0..config.num_generated_tests {
        let offset = i * 32;
        let test_data = if offset < ast_data.len() {
            &ast_data[offset..]
        } else {
            ast_data
        };
        
        let generated_expr = match generate_from_bytes(test_data, ast_config.clone()) {
            Ok(generated_expr) => generated_expr,
            Err(_) => continue,
        };
        
        let evalexpr_eval = match EvalexprEvaluator::new(generated_expr.expr.clone(), generated_expr.num_inputs) {
            Ok(eval) => eval,
            Err(e) => { println!("EvalexprEvaluator creation failed: {}", e); continue; },
        };
        
        let pytorch_eval = PyTorchEvaluator {
            expr: generated_expr.expr,
            num_inputs: generated_expr.num_inputs,
            num_outputs: 1,
        };
        
        used_vars_list.push(generated_expr.num_inputs);
        evalexpr_evaluators.push(evalexpr_eval);
        pytorch_evaluators.push(pytorch_eval);
    }
    
    if evalexpr_evaluators.is_empty() {
        return;
    }
    
    for (idx, ((evalexpr_eval, pytorch_eval), num_inputs)) in 
        evalexpr_evaluators.iter()
            .zip(pytorch_evaluators.iter())
            .zip(used_vars_list.iter())
            .enumerate() 
    {
        if *num_inputs == 0 {
            continue;
        }
        
        let test_inputs = &inputs[..*num_inputs];
        
        let pytorch_jacobian = match compute_pytorch_gradients(pytorch_eval, test_inputs) {
            Ok(jac) => jac,
            Err(_) => continue, 
        };
        println!(": Computed PyTorch Jacobian: {:?}", InfixPrinter::print(&evalexpr_eval.expr, *num_inputs));
        
        for var_idx in 0..*num_inputs {
            let evalexpr_grad = match evalexpr_eval.derivative(var_idx) {
                Ok(deriv_fn) => deriv_fn(test_inputs),
                Err(_) => continue,
            };
            
            let pytorch_grad = pytorch_jacobian[var_idx];
            
            if let Err(e) = compare_derivatives(evalexpr_grad, pytorch_grad, var_idx) {
                let expr_str = InfixPrinter::print(&evalexpr_eval.expr, *num_inputs);
                eprintln!("\n=== CRASH DETECTED ===");
                eprintln!("Expression that caused the mismatch:");
                eprintln!("  {}", expr_str);
                eprintln!("\nInputs:");
                print_vec(test_inputs);
                eprintln!("\nError: {}", e);
                eprintln!("======================\n");
                panic!("Derivative mismatch: {}", e);
            }
        }
    }
});
