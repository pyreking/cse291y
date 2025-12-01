// src/fuzz_harness.rs

use ad_trait::AD;
use ad_trait::function_engine::FunctionEngine;
use ad_trait::differentiable_function::{ForwardAD, ReverseAD}; 
use ad_trait::differentiable_function::DifferentiableFunctionTrait;
use ad_trait::forward_ad::adfn::adfn;
use ad_trait::reverse_ad::adr::adr;
use core::slice::SlicePattern;
use tch::Tensor; 
use std::error::Error;

use crate::oracles::{FuzzingOracles, EngineResults, GroundTruth};

// --- CORE TRAITS (Defining the Interface for a Test Case) ---

pub trait Calculator: Clone
{
    fn eval_expr<T: AD + PartialEq>(&self, _: &[T]) -> T;
    fn num_inputs(&self) -> usize; 
    fn num_outputs(&self) -> usize;
}

// The methods were likely missing in your local file causing E0407, ensure they are present.
pub trait PyTorchComputable: Clone
{
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn Error>>;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
}

/// Defines the interface for calculating a derivative using an external oracle.
pub trait GroundTruthCalculator {
    fn name(&self) -> &'static str;
    
    fn calculate<G: Calculator + PyTorchComputable>(&self, calc: &G, inputs: &[f64]) -> Result<Vec<f64>, Box<dyn Error>>;
}

#[derive(Debug, Clone, Copy)]
pub enum HarnessMode {
    PanicOnFirstError,
    Continuous,
}

#[derive(Debug, Clone)]
pub struct FuzzConfig {
    pub mode: HarnessMode,
    pub num_generated_tests: usize,
    pub oracle_selection: String,
}

// --- ADAPTER Struct (Connects Calculator to ad-trait) ---

#[derive(Clone)]
pub struct SimpleADFunction<T: AD, G: Calculator>
{
    placeholder : T,
    expression: G
}

impl<T: AD, G: Calculator> DifferentiableFunctionTrait<T> for SimpleADFunction<T, G>
{
    const NAME: &'static str = "SimpleFunc";
    fn call(&self, inputs: &[T], _freeze: bool) -> Vec<T>
    {
        vec![self.expression.eval_expr(inputs.as_slice())]
    }

    fn num_inputs(&self) -> usize { self.expression.num_inputs() }
    fn num_outputs(&self) -> usize { self.expression.num_outputs() } 
}

impl<T: AD, G: Calculator> SimpleADFunction<T, G> {
    pub fn to_other_ad_type<T2: AD>(&self) -> SimpleADFunction<T2, G> {
        SimpleADFunction { placeholder: self.placeholder.to_other_ad_type::<T2>(),
                           expression: self.expression.clone() }
    }
}

// --- ORACLE DRIVER (The Engine) ---

pub fn run_ad_tests<G: Calculator + PyTorchComputable + 'static, T: GroundTruthCalculator>(
    inputs: Vec<f64>,
    calc: G,
    oracles: &FuzzingOracles,
    gt_calculators: &[T],
    mode: HarnessMode, 
) -> Result<(), Box<dyn Error>> {
    // FIX E0034: Disambiguate the num_inputs call by specifying the trait.
    if inputs.len() != PyTorchComputable::num_inputs(&calc) || inputs.len() < 1 {
        print!("Input length mismatch: expected {}, got {}", PyTorchComputable::num_inputs(&calc), inputs.len());
        println!("Exiting due to input error!!");
        return Ok(());
    }

    // 1. Compute AD results
    let func_standard = SimpleADFunction { placeholder: 0.0, expression: calc.clone() };

    let func_rev_derivative = func_standard.to_other_ad_type::<adr>();
    let rev_engine = FunctionEngine::new(func_standard.clone(), func_rev_derivative, ReverseAD::new());
    let (_f_res_rev, reverse_jacobian) = rev_engine.derivative(&inputs); 

    let func_fwd_derivative = func_standard.to_other_ad_type::<adfn<1>>();
    let fwd_engine = FunctionEngine::new(func_standard.clone(), func_fwd_derivative, ForwardAD::new());
    let (_f_res_fwd, forward_jacobian) = fwd_engine.derivative(&inputs); 

    // 2. Compute ALL Ground Truths
    let mut ground_truths = Vec::new();
    for gt_calc in gt_calculators {
        if let Ok(jacobian) = gt_calc.calculate(&calc, &inputs) {
            ground_truths.push(GroundTruth { name: gt_calc.name(), jacobian });
        }
    }

    // 3. Collect Engine Results
    let engine_results = EngineResults {
        inputs: inputs.clone(),
        reverse: reverse_jacobian.into_iter().map(|d| (*d).into()).collect::<Vec<f64>>(), 
        forward: forward_jacobian.into_iter().map(|d| (*d).into()).collect::<Vec<f64>>(), 
    };

    println!("Engine Results: {:?}", engine_results);
    // 4. Run all Oracle Checks and return the result
    oracles.check_all(&engine_results, &ground_truths, mode)
}

pub fn run_custom_test<G: Calculator + PyTorchComputable + 'static, T: GroundTruthCalculator>(
    inputs: Vec<f64>,
    calc: G,
    gt_calculators: &[T],
) -> Result<(), Box<dyn Error>> {
    use crate::oracles::FuzzingOracles;
    
    let oracles = FuzzingOracles::new("all".to_string());
    let result = run_ad_tests(inputs.clone(), calc, &oracles, gt_calculators, HarnessMode::PanicOnFirstError);
    
    // Print result regardless of pass/fail
    match &result {
        Ok(_) => println!("Test PASSED"),
        Err(e) => println!("Test FAILED: {}", e),
    }
    
    result
}
