// src/oracles/reverse_vs_forward.rs

use super::{EngineResults, Oracle, GroundTruth};
use std::error::Error;

/// ReverseVsForwardCheck: Ensures that the Jacobians calculated by Reverse AD and 
/// Forward AD are nearly identical, checking for internal consistency in the AD engine.
#[derive(Clone)]
pub struct ReverseVsForwardCheck;

impl Oracle for ReverseVsForwardCheck {
    /// Tolerance constant for trait satisfaction. The actual tolerances are defined below.
    const TOLERANCE: f64 = 1e-9; 

    /// Executes the check for a single partial derivative.
    /// Uses a hybrid tolerance model to handle results near zero and large results robustly.
    fn check(&self, engine: &EngineResults, _gt: Option<&GroundTruth>, i: usize) -> Result<(), Box<dyn Error>> {
        
        // Define tolerances as local constants for the hybrid check.
        const ABS_TOLERANCE: f64 = 1e-12; // Absolute threshold (for results near zero)
        const REL_TOLERANCE: f64 = 1e-9;  // Relative threshold (1 part per billion)

        let rev_result = engine.reverse[i];
        let fwd_result = engine.forward[i];
        
        // // Skip check if either result is not finite (NaN, Inf)
        // if !rev_result.is_finite() || !fwd_result.is_finite() {
        //     return Ok(());
        // }

        let diff = (rev_result - fwd_result).abs();

        // 1. Calculate the scaled threshold: max(ABS_TOLERANCE, |Fwd Result| * REL_TOLERANCE)
        let scaled_rel_threshold = fwd_result.abs() * REL_TOLERANCE;
        let threshold = ABS_TOLERANCE.max(scaled_rel_threshold);
        
        // 2. Perform the Hybrid check: Fail only if difference is greater than the threshold
        if diff > threshold || (rev_result.is_nan() != fwd_result.is_nan()) {
            
            // Calculate relative difference, safely handling division by zero for presentation
            let relative_diff = if fwd_result.abs() > ABS_TOLERANCE {
                diff / fwd_result.abs()
            } else {
                // If result is near zero, the absolute difference is the most meaningful error metric.
                diff 
            };

            let percent_diff = (relative_diff * 100.0).min(100.0);
            
            Err(format!(
                "Reverse vs Forward failed! Gradients differ. (Hybrid Tolerance Check)\n\
                Rev: {:.10e}, Fwd: {:.10e}\n\
                Absolute Diff: {:.10e}\n\
                Relative Diff: {:.10e} ({}%)\n\
                Tolerance Threshold: {:.10e} (max of Abs:{:.10e} or Rel:{:.10e})",
                rev_result, fwd_result, 
                diff, 
                relative_diff, percent_diff,
                threshold, ABS_TOLERANCE, scaled_rel_threshold
            ).into())
        } else {
            Ok(())
        }
    }
}
