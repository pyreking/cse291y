// src/oracles/evalexpr_vs_pytorch.rs

use super::GroundTruth;
use std::error::Error;
use crate::ast_evaluator::EvalexprEvaluator;

pub struct EvalexprVsPyTorchCheck {
    abs_tolerance: f64,
    rel_tolerance: f64,
}

impl EvalexprVsPyTorchCheck {
    pub fn new() -> Self {
        EvalexprVsPyTorchCheck {
            abs_tolerance: 1e-12,
            rel_tolerance: 1e-9,
        }
    }
    
    pub fn check_derivative(
        &self,
        evalexpr_eval: &EvalexprEvaluator<()>,
        inputs: &[f64],
        gt: &GroundTruth,
        var_index: usize,
    ) -> Result<(), Box<dyn Error>> {
        let gt_val = gt.jacobian[var_index];
        
        if !gt_val.is_finite() {
            return Ok(());
        }
        
        // evalexpr-jit derivative
        let deriv_fn = evalexpr_eval.derivative(var_index)?;
        let evalexpr_val = deriv_fn(inputs);
        
        let diff = (evalexpr_val - gt_val).abs();
        
        // Calculate thresh
        let scaled_rel_threshold = gt_val.abs() * self.rel_tolerance;
        let threshold = self.abs_tolerance.max(scaled_rel_threshold);
        
        if diff > threshold {
            return Err(format!(
                "evalexpr-jit vs {} derivative mismatch for x_{}: evalexpr-jit = {}, {} = {}, diff = {} (threshold = {})",
                gt.name, var_index, evalexpr_val, gt.name, gt_val, diff, threshold
            ).into());
        }
        
        Ok(())
    }
    
    pub fn check_all(
        &self,
        evalexpr_eval: &EvalexprEvaluator<()>,
        inputs: &[f64],
        ground_truths: &[GroundTruth],
    ) -> Result<(), Box<dyn Error>> {
        let num_inputs = evalexpr_eval.num_inputs;
        
        for gt in ground_truths {
            for i in 0..num_inputs {
                self.check_derivative(evalexpr_eval, inputs, gt, i)?;
            }
        }
        
        Ok(())
    }
}

impl Default for EvalexprVsPyTorchCheck {
    fn default() -> Self {
        Self::new()
    }
}
