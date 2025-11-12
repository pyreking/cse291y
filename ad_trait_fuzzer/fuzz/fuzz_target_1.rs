// fuzz/fuzz_target_1.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use rand::thread_rng; 
use std::env;

// --- Imports from your library modules ---
use fuzz_core::input_decoder::{FuzzInputDecoder, TwoInputDecoder}; 
use fuzz_core::fuzz_harness::{run_ad_tests, HarnessMode, FuzzConfig}; 
use fuzz_core::oracles::{FuzzingOracles}; 
use fuzz_core::gt_calculators::PyTorchGroundTruthCalculator; 
use fuzz_core::rpn_evaluator::RpnEvaluator; 
use fuzz_core::test_generator; 

const NUM_GENERATED_TESTS: usize = 1; 

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

// --- Fuzz Target Implementation ---

fuzz_target!(|data: &[u8]| {
    
    let config: FuzzConfig = get_fuzz_config();
    
    let inputs: Vec<f64> = match TwoInputDecoder::decode(data) {
        Ok(inputs) => inputs,
        Err(_) => return,
    };
    
    // Input Sanitization
    let x: f64 = inputs[0];
    let y: f64 = inputs[1];
    if !x.is_finite() || !y.is_finite() || x <= 0.0 || x.abs() > 1e10 || y.abs() > 100.0 {
        return;
    }
    
    // --- Test Setup ---
    let mut rng = thread_rng(); 
    
    let mut test_definitions = Vec::new();
    for _ in 0..config.num_generated_tests {
        let test_def = test_generator::generate_random_test(&mut rng);
        test_definitions.push(test_def);
    }

    // Pass the configuration to the oracle constructor
    let oracles = FuzzingOracles::new(config.oracle_selection.clone());
    
    let gt_calculators = [
        PyTorchGroundTruthCalculator,
    ];
    
    for test_def in test_definitions {
        let evaluator = RpnEvaluator { definition: test_def };
        
        run_ad_tests(inputs.clone(), evaluator, &oracles, &gt_calculators, config.mode); 
    }
});