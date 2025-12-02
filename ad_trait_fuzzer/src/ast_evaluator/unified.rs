// src/ast_evaluator/unified.rs


use crate::ast_expr::Expr;
use crate::fuzz_harness::{Calculator, PyTorchComputable};
use super::{AdEvaluator, PyTorchEvaluator, EvalexprEvaluator, InfixPrinter};
use ad_trait::AD;
use tch::Tensor;
use std::error::Error;


/// Unified eval for both AD and PyTorch

#[derive(Clone)]
pub struct AdPyUnified<Tag: Clone> {
    ad_eval: AdEvaluator<Tag>,
    pytorch_eval: PyTorchEvaluator<Tag>,
    num_inputs: usize,
    expr: Expr<Tag>,
}

impl<Tag: Clone + std::fmt::Debug> AdPyUnified<Tag> {
    pub fn new(expr: Expr<Tag>, num_inputs: usize, num_outputs: usize) -> Self {

        AdPyUnified {
            ad_eval: AdEvaluator {
                expr: expr.clone(),
                num_inputs,
                num_outputs,
            },
            pytorch_eval: PyTorchEvaluator {
                expr: expr.clone(),
                num_inputs,
                num_outputs,
            },
            num_inputs: num_inputs,
            expr: expr.clone(),
        }
    }
    
    pub fn get_expr(&self) -> &Expr<Tag> {
        &self.expr
    }

    pub fn num_inputs(&self) -> usize {
        self.ad_eval.num_inputs
    }
}

impl<Tag: Clone> Calculator for AdPyUnified<Tag> {
    fn eval_expr<T: AD>(&self, inputs: &[T]) -> T {
        self.ad_eval.eval_expr(inputs)
    }
    
    fn num_inputs(&self) -> usize {
        self.num_inputs
    }
    
    fn num_outputs(&self) -> usize {
        self.ad_eval.num_outputs
    }
}

impl<Tag: Clone> PyTorchComputable for AdPyUnified<Tag> {
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


// the same as "unified" but with evalexpr-jit and PyTorch
#[derive(Clone)]
pub struct EvalexprPyUnified<Tag: Clone> {
    evalexpr_eval: EvalexprEvaluator<Tag>,
    pytorch_eval: super::PyTorchEvaluator<Tag>,
    num_inputs: usize,
    expr: Expr<Tag>,
}

impl<Tag: Clone> EvalexprPyUnified<Tag> {
    pub fn new(expr: Expr<Tag>, num_inputs: usize) -> Result<Self, Box<dyn Error>> {
        let evalexpr_eval = EvalexprEvaluator::new(expr.clone(), num_inputs)?;
        let pytorch_eval = super::PyTorchEvaluator {
            expr: expr.clone(),
            num_inputs,
            num_outputs: 1,
        };
        
        Ok(EvalexprPyUnified {
            evalexpr_eval,
            pytorch_eval,
            num_inputs,
            expr,
        })
    }
    
    pub fn evalexpr(&self) -> &EvalexprEvaluator<Tag> {
        &self.evalexpr_eval
    }
    
    pub fn get_expr(&self) -> &Expr<Tag> {
        &self.expr
    }
    
    pub fn expr_string(&self) -> String {
        InfixPrinter::print(&self.expr, self.num_inputs)
    }
}

impl<Tag: Clone> PyTorchComputable for EvalexprPyUnified<Tag> {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn Error>> {
        self.pytorch_eval.compute_pytorch(inputs)
    }
    
    fn num_inputs(&self) -> usize {
        self.num_inputs
    }
    
    fn num_outputs(&self) -> usize {
        1
    }
}
