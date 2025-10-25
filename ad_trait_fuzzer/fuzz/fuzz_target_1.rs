#![no_main]
use libfuzzer_sys::fuzz_target;

use ad_trait::AD;
use ad_trait::function_engine::FunctionEngine;
use ad_trait::differentiable_function::{DifferentiableFunctionTrait, ForwardAD, ReverseAD};
use ad_trait::forward_ad::adfn::adfn;
use ad_trait::reverse_ad::adr::adr;
use core::convert::TryInto;

/// A simple, custom differentiable function used as the target for the fuzzer.
///
/// The function calculates: `z = (x.sin() * y.exp())^2 + x.sqrt()`, where
/// `inputs[0]` is `x` and `inputs[1]` is `y`.
#[derive(Clone)]
pub struct SimpleADFunction<T: AD> {
    /// A placeholder field required by `ad-trait` for type context.
    placeholder: T
}

impl<T: AD> DifferentiableFunctionTrait<T> for SimpleADFunction<T> {
    const NAME: &'static str = "SimpleFunc";

    /// The core function logic to be differentiated.
    ///
    /// The function takes two inputs (`x` and `y`) and produces one output (`z`).
    ///
    /// # Arguments
    /// * `inputs`: A slice containing the function arguments [x, y].
    /// * `_freeze`: Unused in this implementation.
    ///
    /// # Returns
    /// A `Vec<T>` containing the single output `z`.
    fn call(&self, inputs: &[T], _freeze: bool) -> Vec<T> {
        // Safe check to prevent panics on undersized inputs from the fuzzer.
        if inputs.len() < 2 {
            return vec![T::zero()];
        }
        
        let x = inputs[0];
        let y = inputs[1];

        // The function: z = (x.sin() * y.exp())^2 + x.sqrt()
        let z = (x.sin() * y.exp()).powi(2) + x.sqrt();

        vec![z]
    }

    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}

impl<T: AD> SimpleADFunction<T> {
    /// Utility function to convert the function definition to use a different
    /// AD type (e.g., from `f64` to `adr` or `adfn`).
    pub fn to_other_ad_type<T2: AD>(&self) -> SimpleADFunction<T2> {
        SimpleADFunction { placeholder: self.placeholder.to_other_ad_type::<T2>() }
    }
}

/// The main fuzz target function.
///
/// It takes an arbitrary slice of bytes, converts the first 16 bytes into two
/// `f64` values (`x` and `y`), and then performs differential fuzzing.
///
/// It compares the Jacobian (gradient) calculated by the **ReverseAD** mode
/// against the Jacobian calculated by the **ForwardAD** mode. If the results
/// differ by more than the tolerance (`1e-9`), it panics, indicating a bug
/// in one of the AD modes for that specific input.
fuzz_target!(|data: &[u8]| {
    // 1. Input Processing and Filtering (Needs 16 bytes for two f64s)
    if data.len() < 16 {
        return;
    }

    // Convert the first 16 bytes into two f64 values (x and y)
    let x_bytes: [u8; 8] = match data[0..8].try_into() { Ok(arr) => arr, Err(_) => return, };
    let y_bytes: [u8; 8] = match data[8..16].try_into() { Ok(arr) => arr, Err(_) => return, };
    let inputs = vec![f64::from_le_bytes(x_bytes), f64::from_le_bytes(y_bytes)];

    // 2. Reverse AD Calculation Setup
    let func_standard = SimpleADFunction { placeholder: 0.0 };
    // Clone the structure and convert types for ReverseAD (adr)
    let func_rev_derivative = func_standard.to_other_ad_type::<adr>();
    let rev_engine = FunctionEngine::new(func_standard.clone(), func_rev_derivative, ReverseAD::new());
    
    let (_f_res_rev, reverse_jacobian) = rev_engine.derivative(&inputs);

    // 3. Forward AD Calculation Setup
    let func_standard_fwd = SimpleADFunction { placeholder: 0.0 };
    // Clone the structure and convert types for ForwardAD (adfn)
    // Note: <adfn<1>> is used because we are calculating the Jacobian (d/dx and d/dy) of a single output function.
    let func_fwd_derivative = func_standard_fwd.to_other_ad_type::<adfn<1>>();
    let fwd_engine = FunctionEngine::new(func_standard_fwd, func_fwd_derivative, ForwardAD::new());
    
    let (_f_res_fwd, forward_jacobian) = fwd_engine.derivative(&inputs);

    // 4. Differential Oracle (Comparison)
    if reverse_jacobian.len() != forward_jacobian.len() {
        panic!("AD mode Jacobians have different dimensions!");
    }
    
    // Iterate through the gradients (Jacobian entries) and compare them.
    for (rev_grad, for_grad) in reverse_jacobian.iter().zip(forward_jacobian.iter()) {
        
        let rev_val = *rev_grad;
        let for_val = *for_grad;
        
        // Check if the absolute difference exceeds the floating-point tolerance.
        if (rev_val - for_val).abs() > 1e-9 {
            panic!("Differential Fuzzing Failure: Gradients differ greatly! Reverse: {}, Forward: {}", rev_val, for_val);
        }
    }
});