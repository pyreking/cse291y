// src/rpn_evaluator.rs

use ad_trait::AD;
use tch::Tensor;
use std::error::Error;
use crate::fuzz_harness::{Calculator, PyTorchComputable}; 
use crate::test_definition::TestDefinition;

// --- RpnTensor Newtype ---
/// Wrapper for tch::Tensor to avoid coherence errors with the generic impl<T: AD>.
#[derive(PartialEq)]
struct RpnTensor(Tensor);

impl Clone for RpnTensor {
    fn clone(&self) -> Self {
        RpnTensor(self.0.shallow_clone())
    }
}


// --- Trait for RPN Operations ---

/// Trait to unify the syntax for AD types and Tensor types.
pub trait RpnEvalType: Sized + Clone + PartialEq {
    // Constants
    fn const_2() -> Self;

    // Unary Functions
    fn fun_sin(self) -> Self;
    fn fun_exp(self) -> Self;
    fn fun_sqrt(self) -> Self;

    // Binary Operations
    fn op_add(self, other: Self) -> Self;
    fn op_mul(self, other: Self) -> Self;

    // Special Operations (Power 2)
    fn op_pow2(self) -> Self;
}

// --- Implementation for T: AD (Generic AD types) ---

impl<T: AD + PartialEq> RpnEvalType for T {
    fn const_2() -> T { T::from_f64(2.0).unwrap() }

    fn fun_sin(self) -> T { self.sin() }
    fn fun_exp(self) -> T { self.exp() }
    fn fun_sqrt(self) -> T { self.sqrt() }

    fn op_add(self, other: T) -> T { self + other }
    fn op_mul(self, other: T) -> T { self * other }

    fn op_pow2(self) -> T { self.powi(2) }
}

// --- Implementation for RpnTensor (PyTorch Ground Truth) ---

impl RpnEvalType for RpnTensor {
    fn const_2() -> RpnTensor { RpnTensor(Tensor::from(2.0).to_kind(tch::Kind::Double)) } 

    fn fun_sin(self) -> RpnTensor { RpnTensor(self.0.sin()) }
    fn fun_exp(self) -> RpnTensor { RpnTensor(self.0.exp()) }
    fn fun_sqrt(self) -> RpnTensor { RpnTensor(self.0.sqrt()) }

    fn op_add(self, other: RpnTensor) -> RpnTensor { RpnTensor(self.0 + other.0) }
    fn op_mul(self, other: RpnTensor) -> RpnTensor { RpnTensor(self.0 * other.0) }

    fn op_pow2(self) -> RpnTensor { RpnTensor(self.0.pow_tensor_scalar(2.0)) }
}


// --- Single, Generic RPN Evaluator Function ---

fn evaluate_rpn<T: RpnEvalType>(rpn_tokens: &[String], inputs: &[T]) -> Result<T, &'static str> {
    if inputs.len() < 2 { return Err("Insufficient inputs"); }
    
    let mut stack: Vec<T> = Vec::new();

    let x = inputs[0].clone();
    let y = inputs[1].clone();

    for token in rpn_tokens {
        match token.as_str() {
            "x" => stack.push(x.clone()),
            "y" => stack.push(y.clone()),
            "2" => stack.push(T::const_2()), 
            
            "sin" => stack.pop().ok_or("RPN: Missing operand for sin")
                         .map(|a| stack.push(a.fun_sin()))?,
            "exp" => stack.pop().ok_or("RPN: Missing operand for exp")
                         .map(|a| stack.push(a.fun_exp()))?,
            "sqrt" => stack.pop().ok_or("RPN: Missing operand for sqrt")
                           .map(|a| stack.push(a.fun_sqrt()))?,
            
            "+" => {
                let b = stack.pop().ok_or("RPN: Missing operand for +")?;
                let a = stack.pop().ok_or("RPN: Missing operand for +")?;
                stack.push(a.op_add(b));
            },
            "*" => {
                let b = stack.pop().ok_or("RPN: Missing operand for *")?;
                let a = stack.pop().ok_or("RPN: Missing operand for *")?;
                stack.push(a.op_mul(b));
            },
            "pow" => {
                let power = stack.pop().ok_or("RPN: Missing exponent for pow")?;
                let base = stack.pop().ok_or("RPN: Missing base for pow")?;
                
                let two: T = T::const_2();
                
                if power == two { 
                    stack.push(base.op_pow2());
                } else {
                    return Err("RPN: Only integer power 2 supported for pow token.");
                }
            },
            _ => return Err("RPN: Unknown token"),
        }
    }

    stack.pop().ok_or("RPN: Stack length is not 1 at end of evaluation")
}


// --- RpnEvaluator Struct and Trait Implementations ---

/// The single struct that evaluates the expression defined by RPN tokens.
#[derive(Clone)]
pub struct RpnEvaluator {
    pub definition: TestDefinition,
}

impl Calculator for RpnEvaluator
{
    fn eval_expr<T: AD + PartialEq>(&self, x: T, y: T) -> T
    {
        match evaluate_rpn(&self.definition.expression_rpn, &[x, y]) {
            Ok(result) => result,
            Err(e) => {
                //eprintln!("RPN AD Evaluation Error: {}", e);
                T::zero() 
            }
        }
    }
    
}

impl PyTorchComputable for RpnEvaluator
{
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn Error>> {
        let rpn_inputs: Vec<RpnTensor> = inputs.iter()
            .map(|t| RpnTensor(t.shallow_clone())) 
            .collect();
        
        match evaluate_rpn(&self.definition.expression_rpn, &rpn_inputs) {
            Ok(RpnTensor(result_tensor)) => Ok(vec![result_tensor]),
            Err(e) => Err(Box::from(e)), 
        }
    }
    
    fn num_inputs(&self) -> usize { self.definition.num_inputs }
    fn num_outputs(&self) -> usize { self.definition.num_outputs }
}