// src/ast_evaluator/ad_backend.rs

// AST -> AD trait

use ad_trait::AD;
use crate::ast_expr::Expr;
use crate::fuzz_harness::Calculator;
use super::{MainBackend, evaluate};
use std::collections::HashMap;
use std::fmt::format;

macro_rules! impl_forwarding_ops {
    () => {
        fn neg(self) -> Self { -self }
        fn sin(self) -> Self { self.sin() }
        fn cos(self) -> Self { self.cos() }
        fn tan(self) -> Self { self.tan() }
        fn exp(self) -> Self { self.exp() }
        fn log(self) -> Self { self.ln() }  // Note: AD trait uses ln(), not log()
        fn sqrt(self) -> Self { self.sqrt() }
        fn abs(self) -> Self { self.abs() }
        
        fn add(self, other: Self) -> Self { self + other }
        fn sub(self, other: Self) -> Self { self - other }
        fn mul(self, other: Self) -> Self { self * other }
        fn div(self, other: Self) -> Self { self / other }
        
        fn pow(self, other: Self) -> Self { self.powf(other) }
    };
}


impl<T: AD> MainBackend for T {
    fn from_f64(val: f64) -> Self {
        T::from_f64(val).unwrap_or_else(|| T::zero())
    }
    
    fn zero() -> Self { T::zero() }
    fn one() -> Self { T::one() }
    
    impl_forwarding_ops!();
}

/// Evaluator that uses AD types
#[derive(Clone)]
pub struct AdEvaluator<Tag: Clone> {
    pub expr: Expr<Tag>,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

// specific eval for AD
impl<Tag: Clone> Calculator for AdEvaluator<Tag> {
    fn eval_expr<T: AD>(&self, inputs: &[T]) -> T {
        let mut env = HashMap::new();
        for (i, e) in inputs.iter().enumerate()
        {
            env.insert(format(format_args!("x_{}", i)), *e);
        }
        
        match evaluate(&self.expr, &env) {
            Ok(result) => result,
            Err(e) => {panic!("Error during AD evaluation: {}", e)}
        }
    }
    fn num_inputs(&self) -> usize
    {
        self.num_inputs
    }
}
