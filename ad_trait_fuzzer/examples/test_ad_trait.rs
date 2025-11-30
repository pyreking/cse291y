// examples/test_ad_trait.rs
//
// Direct test of ad_trait to verify it's working correctly
// Run with: cargo +nightly run --example test_ad_trait

use ad_trait::AD;
use ad_trait::reverse_ad::adr::adr;
use ad_trait::forward_ad::adfn::adfn;
use ad_trait::function_engine::FunctionEngine;
use ad_trait::differentiable_function::{ForwardAD, ReverseAD, DifferentiableFunctionTrait};

// Define a simple function: f(x, y) = -0.1 * x
#[derive(Clone)]
struct SimpleFunc<T: AD> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: AD> DifferentiableFunctionTrait<T> for SimpleFunc<T> {
    const NAME: &'static str = "SimpleFunc";
    
    fn call(&self, inputs: &[T], _freeze: bool) -> Vec<T> {
        let neg_point_one = T::constant(-0.1);
        let result = neg_point_one * inputs[0];
        vec![result]
    }
    
    fn num_inputs(&self) -> usize { 2 }
    fn num_outputs(&self) -> usize { 1 }
}

impl<T: AD> SimpleFunc<T> {
    fn new() -> Self {
        SimpleFunc { phantom: std::marker::PhantomData }
    }
    
    fn to_other_ad_type<T2: AD>(&self) -> SimpleFunc<T2> {
        SimpleFunc { phantom: std::marker::PhantomData }
    }
}

fn main() {

    let inputs = vec![1.0, 2.0];
    
    // test reverse
    println!("=== Reverse AD ===");
    let func_f64 = SimpleFunc::<f64>::new();
    let func_rev = func_f64.to_other_ad_type::<adr>();
    let rev_engine = FunctionEngine::new(func_f64.clone(), func_rev, ReverseAD::new());
    let (f_res_rev, reverse_jacobian) = rev_engine.derivative(&inputs);
    
    println!("Function value: {:?}", f_res_rev);
    println!("Reverse Jacobian: {:?}", reverse_jacobian);
    println!("  df/dx = {}", reverse_jacobian[0]);
    println!("  df/dy = {}", reverse_jacobian[1]);
    
    // test forward
    println!("\n=== Forward AD ===");
    let func_fwd = func_f64.to_other_ad_type::<adfn<1>>();
    let fwd_engine = FunctionEngine::new(func_f64.clone(), func_fwd, ForwardAD::new());
    let (f_res_fwd, forward_jacobian) = fwd_engine.derivative(&inputs);
    
    println!("Function value: {:?}", f_res_fwd);
    println!("Forward Jacobian: {:?}", forward_jacobian);
    println!("  df/dx = {}", forward_jacobian[0]);
    println!("  df/dy = {}", forward_jacobian[1]);
    
    // eval
    println!("\n=== Direct eval ===");
    let x_rev = adr::constant(1.0);
    let y_rev = adr::constant(2.0);
    let neg_point_one = adr::constant(-0.1);
    let result = neg_point_one * x_rev;
    println!("Direct adr result: value={}, type={}", result.value(), std::any::type_name::<adr>());
}
