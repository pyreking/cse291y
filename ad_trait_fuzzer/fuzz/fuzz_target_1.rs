#![no_main]
use libfuzzer_sys::fuzz_target;
use tch::{Tensor, Kind};

use ad_trait::AD;
use ad_trait::function_engine::FunctionEngine;
use ad_trait::differentiable_function::{DifferentiableFunctionTrait, ForwardAD, ReverseAD};
use ad_trait::forward_ad::adfn::adfn;
use ad_trait::reverse_ad::adr::adr;
use core::convert::TryInto;

/// Trait for functions that can be computed using PyTorch tensors for ground truth
pub trait PyTorchComputable {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn std::error::Error>>;
    fn num_inputs(&self) -> usize;
    fn num_outputs(&self) -> usize;
}


/// AD trait where z = (sin(x) * exp(y))^2 + sqrt(x)
macro_rules! compute_expression_ad {
    ($x:expr, $y:expr) => {
        ($x.sin() * $y.exp()).powi(2) + $x.sqrt()
    };
}

trait Calculator
{
    fn eval_expr<T:AD>(&self, x: T, y: T) -> T;
}

#[derive(Clone)]
struct Jacobian;

impl Calculator for Jacobian
{
    fn eval_expr<T:AD>(&self, x: T, y: T) -> T
    {
	(x.sin() * y.exp()).powi(2) + x.sqrt()
    }
}

impl PyTorchComputable for Jacobian
{
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn std::error::Error>> {
        if inputs.len() < 2 {
            return Err("Need at least 2 inputs".into());
        }
        
        let x_t = &inputs[0];
        let y_t = &inputs[1];
        
        let z = (x_t.sin() * y_t.exp()).pow_tensor_scalar(2.0) + x_t.sqrt();
        
        Ok(vec![z])
    }
    
    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}

/// tch-rs
fn pytorch_ground_truth<F: PyTorchComputable>(func: &F, input_values: &[f64]) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    // Create tensors with requires_grad=true
    let mut tensors: Vec<Tensor> = Vec::new();
    for &val in input_values {
        tensors.push(
            Tensor::from(val)
                .set_requires_grad(true)
                .to_kind(Kind::Double)
        );
    }
    
    let outputs = func.compute_pytorch(&tensors)?;
    if outputs.is_empty() {
        return Err("Function returned no outputs".into());
    }
    
    outputs[0].backward();
    
    let mut gradients = Vec::new();
    for tensor in &tensors {
        let grad = f64::try_from(tensor.grad().double_value(&[]))?;
        gradients.push(grad);
    }
    
    Ok(gradients)
}

fn check_expression_ad_results<G: Calculator + Clone + PyTorchComputable>(x: f64, y: f64, calc: G) -> Vec<f64>
{
    #[derive(Clone)]
    pub struct SimpleADFunction<T: AD, G: Calculator + Clone + PyTorchComputable>
    {
	placeholder : T,
	m_num_inputs: usize,
	m_num_outputs: usize,
	expression: G
	    
    }
    impl<T: AD, G: Calculator + Clone + PyTorchComputable> DifferentiableFunctionTrait<T> for SimpleADFunction<T, G>
    {
	const NAME: &'static str = "SimpleFunc";
	fn call(&self, inputs: &[T], _freeze: bool) -> Vec<T>
	{
	    vec![self.expression.eval_expr(inputs[0], inputs[1])]
	}

	fn num_inputs(&self) -> usize { self.m_num_inputs }
	fn num_outputs(&self) -> usize { self.m_num_outputs }
    }
    impl<T: AD, G: Calculator + Clone + PyTorchComputable> SimpleADFunction<T, G> {
	pub fn to_other_ad_type<T2: AD>(&self) -> SimpleADFunction<T2, G> {
            SimpleADFunction { placeholder: self.placeholder.to_other_ad_type::<T2>(),
			       m_num_inputs: self.m_num_inputs,
			       m_num_outputs: self.m_num_outputs,
			       expression: self.expression.clone() }
	}
    }
    impl<T: AD, G: Calculator + Clone + PyTorchComputable> PyTorchComputable for SimpleADFunction<T, G>
    {
	fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn std::error::Error>>
	{
	    return self.expression.compute_pytorch(inputs);
	}
	
	fn num_inputs(&self) -> usize { self.expression.num_inputs() }
	fn num_outputs(&self) -> usize { self.expression.num_outputs() }
    }
    let func_standard = SimpleADFunction { placeholder: 0.0,
					   m_num_inputs: 2,
					   m_num_outputs: 1,
					   expression: calc };
    let inputs = vec![x, y];

    // Reverse AD
    let func_rev_derivative = func_standard.to_other_ad_type::<adr>();
    let rev_engine = FunctionEngine::new(func_standard.clone(), func_rev_derivative, ReverseAD::new());
    let (_f_res_rev, derivative_result_rev) = rev_engine.derivative(&inputs);

    // Forward AD
    let func_fwd_derivative = func_standard.to_other_ad_type::<adfn<1>>();
    let fwd_engine = FunctionEngine::new(func_standard.clone(), func_fwd_derivative, ForwardAD::new());
    let (_f_res_fwd, derivative_result_fwd) = fwd_engine.derivative(&inputs);

    // PyTorch AD
    let pytorch_derivative = match pytorch_ground_truth(&func_standard, &inputs) {
        Ok(grad) => grad,
        Err(_) => return vec![], // Skip if PyTorch fails
    };

    if (derivative_result_rev.len() != derivative_result_fwd.len()) || (pytorch_derivative.len() != derivative_result_rev.len())
    {
        panic!("Derivative dimensions mismatch!");
    }
    let mut return_val: Vec<f64> = vec![];
    for i in 0..derivative_result_rev.len()
    {
	let tolerance = 1e-6;
	let rev_result: f64 = derivative_result_rev[i].into();
	let fwd_result: f64 = derivative_result_fwd[i].into();
	let pytorch_result: f64 = pytorch_derivative[i];
	if (derivative_result_rev[i] - derivative_result_fwd[i]).abs() > tolerance
	{
	    println!("Reverse and forward derivative results comparison:");
	    println!("Results differ greater than tolerance at {}.", i);
	    println!("Reverse derivative value: {}\nForward derivative value: {}", rev_result, fwd_result);
	    println!("(x,y): ({}, {})", x, y);
	    println!("Percent Difference: {}%", ((rev_result - fwd_result).abs()) / rev_result * 100.0);
	}
	if (rev_result - pytorch_result).abs() > tolerance
	{
	    println!("Reverse and PyTorch derivative results comparison:");
	    println!("Results differ greater than tolerance at {}.", i);
	    println!("Reverse derivative value: {}\nPyTorch derivative value: {}", rev_result, pytorch_result);
	    println!("(x,y): ({}, {})", x, y);
	    println!("Percent Difference: {}%", ((rev_result - pytorch_result).abs()) / rev_result * 100.0);
	}
	if (fwd_result - pytorch_result).abs() > tolerance
	{
	    println!("PyTorch derivatve and forward derivative results comparison:");
	    println!("Results differ greater than tolerance at {}.", i);
	    println!("PyTorch derivative value: {}\nForward derivative value: {}", pytorch_result, fwd_result);
	    println!("(x,y): ({}, {})", x, y);
	    println!("Percent Difference: {}%", ((pytorch_result - fwd_result).abs()) / pytorch_result * 100.0);
	}
	return_val.push(rev_result);
    }
    return return_val
}

/// PyTorch
macro_rules! compute_expression_pytorch {
    ($x:expr, $y:expr) => {
        ($x.sin() * $y.exp()).pow_tensor_scalar(2.0) + $x.sqrt()
    };
}

#[derive(Clone)]
pub struct SimpleADFunction<T: AD> {
    placeholder: T
}

impl<T: AD> DifferentiableFunctionTrait<T> for SimpleADFunction<T> {
    const NAME: &'static str = "SimpleFunc";

    fn call(&self, inputs: &[T], _freeze: bool) -> Vec<T> {
        if inputs.len() < 2 {
            return vec![T::zero()];
        }

        let x = inputs[0];
        let y = inputs[1];

        let z = compute_expression_ad!(x, y);

        vec![z]
    }

    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}

impl<T: AD> SimpleADFunction<T> {
    pub fn to_other_ad_type<T2: AD>(&self) -> SimpleADFunction<T2> {
        SimpleADFunction { placeholder: self.placeholder.to_other_ad_type::<T2>() }
    }
}

impl<T: AD> PyTorchComputable for SimpleADFunction<T> {
    fn compute_pytorch(&self, inputs: &[Tensor]) -> Result<Vec<Tensor>, Box<dyn std::error::Error>> {
        if inputs.len() < 2 {
            return Err("Need at least 2 inputs".into());
        }
        
        let x_t = &inputs[0];
        let y_t = &inputs[1];
        
        let z = compute_expression_pytorch!(x_t, y_t);
        
        Ok(vec![z])
    }
    
    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}


fuzz_target!(|data: &[u8]| {
    // 1. Input Processing / Filtering
    if data.len() < 16 {
        return;
    }

    let x_bytes: [u8; 8] = match data[0..8].try_into() { Ok(arr) => arr, Err(_) => return, };
    let y_bytes: [u8; 8] = match data[8..16].try_into() { Ok(arr) => arr, Err(_) => return, };
    let x = f64::from_le_bytes(x_bytes);
    let y = f64::from_le_bytes(y_bytes);
    
    if !x.is_finite() || !y.is_finite() || x <= 0.0 {
        return;
    }
    
    if x.abs() > 1e10 || y.abs() > 100.0 {
        return;
    }

    let jacob = Jacobian{};
    check_expression_ad_results(x, y, jacob);
});

