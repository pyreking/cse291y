// src/ast_evaluator/evalexpr_backend.rs

use super::print_backend::InfixPrinter;
use crate::ast_expr::Expr;
use crate::fuzz_harness::PyTorchComputable;
use evalexpr_jit::{Equation, backends::vector::Vector};
use std::error::Error;
use std::sync::Arc;
use tch::Tensor;

#[derive(Clone)]
pub struct EvalexprEvaluator<Tag: Clone> {
    pub expr: Expr<Tag>,
    pub num_inputs: usize,
    equation: Option<Equation>,
}

impl<Tag: Clone> EvalexprEvaluator<Tag> {
    pub fn new(expr: Expr<Tag>, num_inputs: usize) -> Result<Self, Box<dyn Error>> {
        let expr_str = InfixPrinter::print(&expr, num_inputs);
        let equation = Equation::new(expr_str)?;
        
        Ok(EvalexprEvaluator {
            expr,
            num_inputs,
            equation: Some(equation),
        })
    }
    
    // fixed-size array issues are fixed
    pub fn eval<V: Vector>(&self, inputs: &V) -> Result<f64, Box<dyn Error>> {
        match &self.equation {
            Some(eq) => Ok(eq.eval(inputs)?),
            None => Err("Equation not init".into()),
        }
    }
    
    /// Compute der with respect to var i
    pub fn derivative(&self, var_index: usize) -> Result<Arc<dyn Fn(&[f64]) -> f64 + Send + Sync>, Box<dyn Error>> {
        match &self.equation {
            Some(eq) => {
                let var_name = format!("x_{}", var_index);
                Ok(eq.derivative(&var_name)?.clone())
            },
            None => Err("Equation not init".into()),
        }
    }
    
    pub fn expr_string(&self) -> String {
        InfixPrinter::print(&self.expr, self.num_inputs)
    }
}
