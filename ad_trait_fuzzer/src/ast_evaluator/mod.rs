// src/ast_evaluator/mod.rs

// AST evaluation for different numeric backends
// unified interface for evaluating AST expr

use std::collections::HashMap;
use crate::ast_expr::Expr;

pub mod ad_backend;
pub mod pytorch_backend;
pub mod unified;

pub use ad_backend::AdEvaluator;
pub use pytorch_backend::PyTorchEvaluator;

/// env for var bindings during eval
pub type Env<T> = HashMap<String, T>;

pub trait MainBackend: Sized + Clone {
    fn from_f64(val: f64) -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    
    fn neg(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn exp(self) -> Self;
    fn log(self) -> Self;
    fn sqrt(self) -> Self;
    fn abs(self) -> Self;
    
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn mul(self, other: Self) -> Self;
    fn div(self, other: Self) -> Self;
    
    fn pow(self, other: Self) -> Self;
}

/// Generic eval for MainBackend
pub fn evaluate<T: MainBackend, Tag>(
    expr: &Expr<Tag>,
    env: &Env<T>,
) -> Result<T, String> {
    use crate::ast_expr::{Op1, Op2};
    match expr {
        Expr::Number(_, val) => Ok(T::from_f64(*val)),
        
        Expr::Boolean(_, _) => Err("Boolean not supported in numeric expressions".to_string()),
        
        Expr::Id(_, name) => {
            env.get(name)
                .cloned()
                .ok_or_else(|| format!("Variable '{}' not found", name))
        }
        
        Expr::UnOp(_, op, sub_expr) => {
            let val = evaluate(sub_expr, env)?;
            Ok(match op {
                Op1::Neg => val.neg(),
                Op1::Sin => val.sin(),
                Op1::Cos => val.cos(),
                Op1::Tan => val.tan(),
                Op1::Exp => val.exp(),
                Op1::Log => val.log(),
                Op1::Sqrt => val.sqrt(),
                Op1::Abs => val.abs(),
            })
        }
        
        Expr::BinOp(_, op, left, right) => {
            let left_val = evaluate(left, env)?;
            let right_val = evaluate(right, env)?;
            Ok(match op {
                Op2::Add => left_val.add(right_val),
                Op2::Sub => left_val.sub(right_val),
                Op2::Mul => left_val.mul(right_val),
                Op2::Div => left_val.div(right_val),
                Op2::Pow => left_val.pow(right_val),
            })
        }
        
        Expr::Let(_, bindings, body) => {
            let mut new_env = env.clone();
            for (name, expr) in bindings {
                let val = evaluate(expr, env)?;
                new_env.insert(name.clone(), val);
            }
            evaluate(body, &new_env)
        }
        
        Expr::Block(_, exprs) => {
            if exprs.is_empty() {
                return Ok(T::zero());
            }
            let mut result = T::zero();
            for expr in exprs {
                result = evaluate(expr, env)?;
            }
            Ok(result)
        }
        
        _ => Err("Unsupported expression type".to_string()),
    }
}
