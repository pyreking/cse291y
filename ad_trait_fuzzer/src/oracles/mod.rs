// src/oracles/mod.rs

use std::error::Error;
use crate::fuzz_harness::HarnessMode; 

mod reverse_vs_forward;
mod ad_vs_pytorch;

pub use reverse_vs_forward::ReverseVsForwardCheck;
pub use ad_vs_pytorch::{ADVsGroundTruthCheck, ADType}; 

// --- Structs for Data Transport ---

/// Container for any single ground truth derivative (name and value).
#[derive(Debug, Clone)]
pub struct GroundTruth {
    pub name: &'static str,
    pub jacobian: Vec<f64>,
}

/// A struct to hold ONLY the AD engine results and contextual input data.
#[derive(Debug, Clone)]
pub struct EngineResults {
    pub inputs: Vec<f64>,
    pub reverse: Vec<f64>,
    pub forward: Vec<f64>,
}

// --- Oracle Trait and Master Struct ---

/// The core trait for any comparison logic.
pub trait Oracle {
    const TOLERANCE: f64;
    /// The check verifies AD engine results against an optional ground truth (for Rev vs GT or Fwd vs GT)
    /// or against None (for Rev vs Fwd).
    fn check(&self, engine: &EngineResults, ground_truth: Option<&GroundTruth>, index: usize) -> Result<(), Box<dyn Error>>;
}

/// The master struct holding all configurable oracle checks.
#[derive(Clone)]
pub struct FuzzingOracles {
    pub reverse_vs_forward: ReverseVsForwardCheck, 
    pub reverse_vs_gt: ADVsGroundTruthCheck,
    pub forward_vs_gt: ADVsGroundTruthCheck,
    pub check_mode: String,
}

impl FuzzingOracles {
    pub fn new(selection: String) -> Self {
        FuzzingOracles {
            reverse_vs_forward: ReverseVsForwardCheck, 
            reverse_vs_gt: ADVsGroundTruthCheck { ad_type: ADType::Reverse },
            forward_vs_gt: ADVsGroundTruthCheck { ad_type: ADType::Forward },
            check_mode: selection, // Store the configured mode
        }
    }
    
    /// Executes all contained oracle checks against the computed results, respecting the harness mode.
    /// Returns an error if any oracle check fails.
    pub fn check_all(&self, engine: &EngineResults, ground_truths: &[GroundTruth], mode: HarnessMode) -> Result<(), Box<dyn Error>> {
        if engine.reverse.len() != engine.forward.len() {
            return Err("Engine error: AD derivative dimension mismatch!".into());
        }

        for i in 0..engine.reverse.len() {
            // 1. Run Internal AD vs AD check (rev_fwd)
            if self.check_mode.eq_ignore_ascii_case("all") || self.check_mode.eq_ignore_ascii_case("rev_fwd") {
                if let Err(e) = self.reverse_vs_forward.check(engine, None, i) {
                    return Err(format!("Oracle check failed for inputs {:?}:\n{}", engine.inputs, e).into());
                }
            }

            // 2. Run all AD vs Ground Truth checks (rev_gt and fwd_gt)
            for gt in ground_truths {
                // Run Reverse AD vs GT
                if self.check_mode.eq_ignore_ascii_case("all") || self.check_mode.eq_ignore_ascii_case("rev_gt") {
                    if let Err(e) = self.reverse_vs_gt.check(engine, Some(gt), i) {
                        return Err(format!("Oracle check failed for inputs {:?} (Rev vs {}):\n{}", engine.inputs, gt.name, e).into());
                    }
                }
                
                // Run Forward AD vs GT
                if self.check_mode.eq_ignore_ascii_case("all") || self.check_mode.eq_ignore_ascii_case("fwd_gt") {
                    if let Err(e) = self.forward_vs_gt.check(engine, Some(gt), i) {
                        return Err(format!("Oracle check failed for inputs {:?} (Fwd vs {}):\n{}", engine.inputs, gt.name, e).into());
                    }
                }
            }
        }
        
        Ok(())
    }
}