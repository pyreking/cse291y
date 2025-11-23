// fuzz/fuzz_target_ast.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::env;

use fuzz_core::input_decoder::{FuzzInputDecoder, TwoInputDecoder, GeneralInputDecoder};
use fuzz_core::fuzz_harness::{run_ad_tests, HarnessMode, FuzzConfig}; 
use fuzz_core::oracles::FuzzingOracles; 
use fuzz_core::gt_calculators::PyTorchGroundTruthCalculator; 
use fuzz_core::ast_evaluator::unified::AllEvaluators;
use fuzz_core::ast_generator::{generate_from_bytes, AstGenConfig};

const NUM_GENERATED_TESTS: usize = 1; 

// Print utility function:
fn print_vec(vec: &Vec<f64>)
{
    for (i, e) in vec.iter().enumerate()
    {
        println!("x_{}: {}", i, e);
    }
}

// --- Configuration Reader (Reads Environment Variables) ---

fn get_fuzz_config() -> FuzzConfig {
    // 1. Harness Mode
    let mode = match env::var("FUZZ_MODE") {
        Ok(val) if val.eq_ignore_ascii_case("continuous") => HarnessMode::Continuous,
        _ => HarnessMode::PanicOnFirstError,
    };

    // 2. Number of Tests
    let num_generated_tests = match env::var("FUZZ_TESTS") {
        Ok(val) => val.parse::<usize>().unwrap_or(NUM_GENERATED_TESTS),
        _ => NUM_GENERATED_TESTS, 
    };

    // 3. Oracle Selection
    let oracle_selection = env::var("FUZZ_ORACLE").unwrap_or_else(|_| "all".to_string());

    FuzzConfig {
        mode,
        num_generated_tests,
        oracle_selection,
    }
}

// --- AST Generation Config ---

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
        .unwrap_or(false);  // Disable by def

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

// --- Fuzz Target Implementation ---

fuzz_target!(|data: &[u8]| {
    let config: FuzzConfig = get_fuzz_config();
    
    let ast_config = get_ast_config();
    let num_variables = ast_config.max_variables;

    let input_decoder: GeneralInputDecoder = GeneralInputDecoder{ input_length: num_variables };

    let min_data_size = num_variables * 8;

    if data.len() < min_data_size
    {
        return;
    }

    let inputs: Vec<f64> = match input_decoder.decode(&data[0..min_data_size]) {
        Ok(inputs) => inputs,
        Err(_) => return,
    };
    
    let x: f64 = inputs[0];
    let y: f64 = inputs[1];
    if !x.is_finite() || !y.is_finite() || x <= 0.0 || x.abs() > 1e10 || y.abs() > 100.0 {
        return;
    }
    
    let ast_data = &data[min_data_size..];
    
    // Generate AST using arbitrary
    let mut evaluators = Vec::new();
    
    for i in 0..config.num_generated_tests {
        let offset = i * 32;
        let test_data = if offset < ast_data.len() {
            &ast_data[offset..]
        } else {
            ast_data
        };
        
        let expr = match generate_from_bytes(test_data, ast_config.clone()) {
            Ok(expr) => expr,
            Err(_) => continue,
        };
        
        let evaluator = AllEvaluators::new(expr, num_variables, 1);
        evaluators.push(evaluator);
    }
    
    if evaluators.is_empty() {
        return;
    }
    
    let oracles = FuzzingOracles::new(config.oracle_selection.clone());
    
    let gt_calculators = [
        PyTorchGroundTruthCalculator,
    ];
    
    for (idx, evaluator) in evaluators.iter().enumerate() {
        if let Err(e) = run_ad_tests(inputs.clone(), evaluator.clone(), &oracles, &gt_calculators, config.mode) {
            eprintln!("\n=== CRASH DETECTED ===");
            eprintln!("Expression that caused the crash:");
            eprintln!("{:#?}", evaluator.get_expr());
            eprintln!("Inputs:");
            print_vec(&inputs);
            eprintln!("Error: {}", e);
            eprintln!("======================\n");
            
            // Panic so libfuzzer can capture it
            panic!("Oracle check failed: {}", e);
        }
    }
});
