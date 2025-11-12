// src/ast_evaluator/unified.rs

/// Unified eval for both AD and PyTorch

use crate::ast_expr::Expr;
use crate::fuzz_harness::{Calculator, PyTorchComputable};
use super::{AdEvaluator, PyTorchEvaluator};
use ad_trait::AD;
use tch::Tensor;
use std::error::Error;

#[derive(Clone)]
pub struct UnifiedEvaluator<Tag: Clone> {
    ad_eval: AdEvaluator<Tag>,
    pytorch_eval: PyTorchEvaluator<Tag>,
}

impl<Tag: Clone + std::fmt::Debug> UnifiedEvaluator<Tag> {
    pub fn new(expr: Expr<Tag>, num_inputs: usize, num_outputs: usize) -> Self {

        UnifiedEvaluator {
            ad_eval: AdEvaluator {
                expr: expr.clone(),
                num_inputs,
                num_outputs,
            },
            pytorch_eval: PyTorchEvaluator {
                expr,
                num_inputs,
                num_outputs,
            },
        }
    }
    
    pub fn get_expr(&self) -> &Expr<Tag> {
        &self.ad_eval.expr
    }
}

impl<Tag: Clone> Calculator for UnifiedEvaluator<Tag> {
    fn eval_expr<T: AD>(&self, x: T, y: T) -> T {
        self.ad_eval.eval_expr(x, y)
    }
}

impl<Tag: Clone> PyTorchComputable for UnifiedEvaluator<Tag> {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn Error>> {
        self.pytorch_eval.compute_pytorch(inputs)
    }
    
    fn num_inputs(&self) -> usize {
        self.ad_eval.num_inputs
    }
    
    fn num_outputs(&self) -> usize {
        self.ad_eval.num_outputs
    }
}
