// src/oracles/ad_vs_pytorch.rs

use super::{EngineResults, Oracle, GroundTruth};
use std::error::Error;

/// Defines which AD type should be compared against the ground truth.
#[derive(Clone)]
pub enum ADType {
    Reverse,
    Forward,
}

/// ADVsGroundTruthCheck: Checks if an AD result (Reverse or Forward) matches the external 
/// ground truth (e.g., PyTorch), also using a robust **hybrid tolerance model**.
#[derive(Clone)]
pub struct ADVsGroundTruthCheck {
    pub ad_type: ADType, 
}

impl Oracle for ADVsGroundTruthCheck {
    /// Tolerance constant for trait satisfaction. The actual tolerances are defined below.
    const TOLERANCE: f64 = 1e-6; 
    
    fn check(&self, engine: &EngineResults, gt: Option<&GroundTruth>, i: usize) -> Result<(), Box<dyn Error>> {
        
        // Define tolerances as local constants inside the function scope.
        const ABS_TOLERANCE: f64 = 1e-12; // Absolute threshold, used when ground truth is near zero.
        const REL_TOLERANCE: f64 = 1e-9;  // Relative threshold, 1 part per billion.
        
        // Ensure a Ground Truth value was provided for this check
        let gt = gt.ok_or("AD vs Ground Truth check requires a ground truth input.")?;

        let (ad_val, ad_name) = match self.ad_type {
            ADType::Reverse => (engine.reverse[i], "Reverse AD"),
            ADType::Forward => (engine.forward[i], "Forward AD"),
        };
        
        let gt_val = gt.jacobian[i];
        let gt_name = gt.name;

        // Skip check if ground truth is not finite (e.g., NaN, Inf)
        if !gt_val.is_finite() {
            return Ok(());
        }

        let diff = (ad_val - gt_val).abs();
        
        // 1. Calculate the scaled threshold: max(ABS_TOLERANCE, |GT| * REL_TOLERANCE)
        let scaled_rel_threshold = gt_val.abs() * REL_TOLERANCE;
        let threshold = ABS_TOLERANCE.max(scaled_rel_threshold);

        // 2. Perform the Hybrid check: Fail only if difference is greater than the threshold
        if diff > threshold {
            let relative_diff = diff / gt_val.abs();
            let percent_diff = (relative_diff * 100.0).min(100.0);
            
            Err(format!(
                "{} vs {} failed! (Hybrid Tolerance Check)\n\
                {}: {:.10e}, {}: {:.10e}\n\
                Absolute Diff: {:.10e}\n\
                Relative Diff: {:.10e} ({}%)\n\
                Tolerance Threshold: {:.10e} (max of Abs:{:.10e} or Rel:{:.10e})",
                ad_name, gt_name,
                ad_name, ad_val, gt_name, gt_val,
                diff, 
                relative_diff, percent_diff,
                threshold, ABS_TOLERANCE, scaled_rel_threshold
            ).into())
        } else {
            Ok(())
        }
    }
}