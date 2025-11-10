// src/gt_calculators.rs

use tch::{Tensor, Kind};
use std::error::Error;
use core::convert::TryFrom; 
use crate::fuzz_harness::{GroundTruthCalculator, PyTorchComputable, Calculator}; 

/// Concrete implementation for calculating Ground Truth via PyTorch.
#[derive(Clone)]
pub struct PyTorchGroundTruthCalculator;

impl GroundTruthCalculator for PyTorchGroundTruthCalculator {
    fn name(&self) -> &'static str { "PyTorch" }

    // G is a generic type for the function (e.g., RpnEvaluator)
    fn calculate<G: Calculator + PyTorchComputable>(&self, calc: &G, inputs: &[f64]) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut tensors: Vec<Tensor> = Vec::new();
        for &val in inputs {
            tensors.push(
                Tensor::from(val)
                    .set_requires_grad(true) // Inputs always require grad
                    .to_kind(Kind::Double)
            );
        }
        
        // 1. Compute PyTorch output
        let outputs = calc.compute_pytorch(&tensors)?; 
        if outputs.is_empty() { return Err("PyTorch function returned no output.".into()); }
        
        // Assuming scalar output
        if outputs[0].numel() != 1 {
            return Err("PyTorch output is not a scalar, skipping derivative calculation.".into());
        }

        // Check if the output requires a gradient. If not, the function evaluated
        // to a constant (derivative must be zero). This prevents the E0599 panic.
        if !outputs[0].requires_grad() {
            let zero_gradients = vec![0.0; inputs.len()];
            return Ok(zero_gradients);
        }

        // 2. Run backpropagation
        outputs[0].backward(); 
        
        // 3. Extract gradients
        let mut gradients = Vec::new();
        for tensor in &tensors {
            let grad_tensor = tensor.grad();

            // Use numel() > 0 to check if a gradient was actually computed.
            let grad = if grad_tensor.numel() > 0 {
                
                // Convert the scalar tensor value to f64
                match f64::try_from(grad_tensor.double_value(&[])) {
                    Ok(val) => val,
                    Err(_) => {
                        //eprintln!("Warning: PyTorch gradient conversion failed unexpectedly. Assuming 0.0");
                        0.0
                    }
                }
            } else {
                // Derivative is 0 if no gradient was stored
                0.0
            };
            gradients.push(grad);
        }
        
        Ok(gradients)
    }
}