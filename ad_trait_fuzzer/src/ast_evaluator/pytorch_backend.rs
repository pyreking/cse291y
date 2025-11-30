// src/ast_evaluator/pytorch_backend.rs

// AST -> PyTorch

use tch::Tensor;
use crate::ast_expr::Expr;
use crate::fuzz_harness::PyTorchComputable;
use super::{MainBackend, evaluate};
use std::collections::HashMap;
use std::error::Error;

macro_rules! impl_unary_ops {
    ($wrapper:ty, .$field:tt) => {
        fn neg(self) -> Self { Self(-self.$field) }
        fn sin(self) -> Self { Self(self.$field.sin()) }
        fn cos(self) -> Self { Self(self.$field.cos()) }
        fn tan(self) -> Self { Self(self.$field.tan()) }
        fn exp(self) -> Self { Self(self.$field.exp()) }
        fn log(self) -> Self { Self(self.$field.log()) }
        fn sqrt(self) -> Self { Self(self.$field.sqrt()) }
        fn abs(self) -> Self { Self(self.$field.abs()) }
    };
}


macro_rules! impl_binary_ops {
    ($wrapper:ty, .$field:tt) => {
        fn add(self, other: Self) -> Self { Self(self.$field + other.$field) }
        fn sub(self, other: Self) -> Self { Self(self.$field - other.$field) }
        fn mul(self, other: Self) -> Self { Self(self.$field * other.$field) }
        fn div(self, other: Self) -> Self { Self(self.$field / other.$field) }
        
        fn pow(self, other: Self) -> Self {
            Self(tch::Tensor::pow(&self.$field, &other.$field))
        }
    };
}

pub struct PyTorchTensor(pub Tensor);

impl Clone for PyTorchTensor {
    fn clone(&self) -> Self {
        PyTorchTensor(self.0.shallow_clone())
    }
}

impl MainBackend for PyTorchTensor {
    fn from_f64(val: f64) -> Self {
        PyTorchTensor(Tensor::from(val).to_kind(tch::Kind::Double))
    }
    
    fn zero() -> Self {
        PyTorchTensor(Tensor::from(0.0).to_kind(tch::Kind::Double))
    }
    
    fn one() -> Self {
        PyTorchTensor(Tensor::from(1.0).to_kind(tch::Kind::Double))
    }
    
    impl_unary_ops!(PyTorchTensor, .0);
    impl_binary_ops!(PyTorchTensor, .0);
}

#[derive(Clone)]
pub struct PyTorchEvaluator<Tag: Clone> {
    pub expr: Expr<Tag>,
    pub num_inputs: usize,
    pub num_outputs: usize,
}

// specific eval for PyTorch
impl<Tag: Clone> PyTorchComputable for PyTorchEvaluator<Tag> {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn Error>> {
        if inputs.len() < self.num_inputs {
            return Err("Insufficient inputs".into());
        }
        
        let mut env = HashMap::new();
        for (i, input) in inputs.iter().enumerate() {
            env.insert(format!("x_{}", i), PyTorchTensor(input.shallow_clone()));
        }
        
        match evaluate(&self.expr, &env) {
            Ok(PyTorchTensor(result)) => Ok(vec![result]),
            Err(e) => Err(e.into()),
        }
    }
    
    fn num_inputs(&self) -> usize { self.num_inputs }
    fn num_outputs(&self) -> usize { self.num_outputs }
}
