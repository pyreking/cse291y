#![no_main]
use libfuzzer_sys::fuzz_target;
use tch::{Tensor, Kind};

use ad_trait::AD;
use ad_trait::function_engine::FunctionEngine;
use ad_trait::differentiable_function::{DifferentiableFunctionTrait, ForwardAD, ReverseAD};
use ad_trait::forward_ad::adfn::adfn;
use ad_trait::reverse_ad::adr::adr;
use core::convert::TryInto;

/// Trait for functions that can be computed using PyTorch tensors for ground truth
pub trait PyTorchComputable {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn std::error::Error>>;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
}


/// AD trait where z = (sin(x) * exp(y))^2 + sqrt(x)
macro_rules! compute_expression_ad {
    ($x:expr, $y:expr) => {
        ($x.sin() * $y.exp()).powi(2) + $x.sqrt()
    };
}

/// PyTorch
macro_rules! compute_expression_pytorch {
    ($x:expr, $y:expr) => {
        ($x.sin() * $y.exp()).pow_tensor_scalar(2.0) + $x.sqrt()
    };
}

#[derive(Clone)]
pub struct SimpleADFunction<T: AD> {
    placeholder: T
}

impl<T: AD> DifferentiableFunctionTrait<T> for SimpleADFunction<T> {
    const NAME: &'static str = "SimpleFunc";

    fn call(&self, inputs: &[T], _freeze: bool) -> Vec<T> {
        if inputs.len() < 2 {
            return vec![T::zero()];
        }

        let x = inputs[0];
        let y = inputs[1];

        let z = compute_expression_ad!(x, y);

        vec![z]
    }

    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}

impl<T: AD> SimpleADFunction<T> {
    pub fn to_other_ad_type<T2: AD>(&self) -> SimpleADFunction<T2> {
        SimpleADFunction { placeholder: self.placeholder.to_other_ad_type::<T2>() }
    }
}

impl<T: AD> PyTorchComputable for SimpleADFunction<T> {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn std::error::Error>> {
        if inputs.len() < 2 {
            return Err("Need at least 2 inputs".into());
        }
        
        let x_t = &inputs[0];
        let y_t = &inputs[1];
        
        let z = compute_expression_pytorch!(x_t, y_t);
        
        Ok(vec![z])
    }
    
    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}

/// tch-rs
fn pytorch_ground_truth<F: PyTorchComputable>(func: &F, input_values: &[f64]) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    // Create tensors with requires_grad=true
    let mut tensors: Vec<Tensor> = Vec::new();
    for &val in input_values {
        tensors.push(
            Tensor::from(val)
                .set_requires_grad(true)
                .to_kind(Kind::Double)
        );
    }
    
    let outputs = func.compute_pytorch(&tensors)?;
    if outputs.is_empty() {
        return Err("Function returned no outputs".into());
    }
    
    outputs[0].backward();
    
    let mut gradients = Vec::new();
    for tensor in &tensors {
        let grad = f64::try_from(tensor.grad().double_value(&[]))?;
        gradients.push(grad);
    }
    
    Ok(gradients)
}

fuzz_target!(|data: &[u8]| {
    // 1. Input Processing / Filtering
    if data.len() < 16 {
        return;
    }

    let x_bytes: [u8; 8] = match data[0..8].try_into() { Ok(arr) => arr, Err(_) => return, };
    let y_bytes: [u8; 8] = match data[8..16].try_into() { Ok(arr) => arr, Err(_) => return, };
    let x = f64::from_le_bytes(x_bytes);
    let y = f64::from_le_bytes(y_bytes);
    
    if !x.is_finite() || !y.is_finite() || x <= 0.0 {
        return;
    }
    
    if x.abs() > 1e10 || y.abs() > 100.0 {
        return;
    }
    
    let inputs = vec![x, y];

    // SimpleADFunction instance
    let func_standard = SimpleADFunction { placeholder: 0.0 };

    // PyTorch Ground Truth
    let pytorch_jacobian = match pytorch_ground_truth(&func_standard, &inputs) {
        Ok(grad) => grad,
        Err(_) => return, // Skip if PyTorch fails
    };

    // Reverse AD
    let func_rev_derivative = func_standard.to_other_ad_type::<adr>();
    let rev_engine = FunctionEngine::new(func_standard.clone(), func_rev_derivative, ReverseAD::new());
    let (_f_res_rev, reverse_jacobian) = rev_engine.derivative(&inputs);

    // Forward AD
    let func_fwd_derivative = func_standard.to_other_ad_type::<adfn<1>>();
    let fwd_engine = FunctionEngine::new(func_standard.clone(), func_fwd_derivative, ForwardAD::new());
    let (_f_res_fwd, forward_jacobian) = fwd_engine.derivative(&inputs);

    // Triple differential oracle lol
    let tolerance = 1e-6;
    
    if reverse_jacobian.len() != pytorch_jacobian.len() || forward_jacobian.len() != pytorch_jacobian.len() {
        panic!("Jacobian dimension mismatch!");
    }
    
    for i in 0..reverse_jacobian.len() {
        let rev_val = reverse_jacobian[i];
        let fwd_val = forward_jacobian[i];
        let pytorch_val = pytorch_jacobian[i];
        
        // Skip if PyTorch returned NaN / Inf
        if !pytorch_val.is_finite() {
            return;
        }
        
        // Reverse vs PyTorch
        let rev_diff = (rev_val - pytorch_val).abs();
        let rev_rel_error = if pytorch_val.abs() > 1e-10 {
            rev_diff / pytorch_val.abs()
        } else {
            rev_diff
        };
        
        if rev_diff > tolerance && rev_rel_error > tolerance {
            panic!(
                "ReverseAD differs from PyTorch ground truth!\n\
                Input: x={:.6e}, y={:.6e}\n\
                Gradient[{}]: Reverse={:.10e}, PyTorch={:.10e}\n\
                Absolute Error: {:.10e}, Relative Error: {:.10e}", 
                x, y, i, rev_val, pytorch_val, rev_diff, rev_rel_error
            );
        }
        
        // Forward vs PyTorch
        let fwd_diff = (fwd_val - pytorch_val).abs();
        let fwd_rel_error = if pytorch_val.abs() > 1e-10 {
            fwd_diff / pytorch_val.abs()
        } else {
            fwd_diff
        };
        
        if fwd_diff > tolerance && fwd_rel_error > tolerance {
            panic!(
                "ForwardAD differs from PyTorch ground truth!\n\
                Input: x={:.6e}, y={:.6e}\n\
                Gradient[{}]: Forward={:.10e}, PyTorch={:.10e}\n\
                Absolute Error: {:.10e}, Relative Error: {:.10e}", 
                x, y, i, fwd_val, pytorch_val, fwd_diff, fwd_rel_error
            );
        }
        
        // Forward vs Reverse
        if (rev_val - fwd_val).abs() > tolerance {
            println!("Diff={:.9e}\n", (rev_val - fwd_val).abs()); // --- IGNORE ---
            panic!(
                "ForwardAD and ReverseAD differ from each other!\n\
                Input: x={:.32e}, y={:.32e}\n\
                Gradient[{}]: Reverse={:.64e}, Forward={:.64e}, PyTorch={:.64e}", 
                x, y, i, rev_val, fwd_val, pytorch_val
            );
        }
    }
});

