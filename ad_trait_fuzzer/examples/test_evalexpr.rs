// examples/test_evalexpr.rs
//
// Test evalexpr-jit for JIT-compiled evaluation and derivatives
// Run with: cargo +nightly run --example test_evalexpr

use fuzz_core::ast_expr::SimpleExpr;
use fuzz_core::ast_evaluator::EvalexprEvaluator;
use fuzz_core::ast_evaluator::unified::AllEvaluators;
use fuzz_core::fuzz_harness::run_custom_test;
use fuzz_core::gt_calculators::PyTorchGroundTruthCalculator;

fn test_evalexpr_vs_ad_trait<const N: usize>(name: &str, expr: SimpleExpr, inputs: [f64; N]) {
    println!("\n=== {} ===", name);
    
    // Test evalexpr-jit
    match EvalexprEvaluator::new(expr.clone(), N) {
        Ok(eval) => {
            println!("  Expression: {}", eval.expr_string());
            match eval.eval(&inputs) {
                Ok(result) => println!("   f({:?}) = {}", inputs, result),
                Err(e) => println!("   Eval error: {}", e),
            }
            
            // Compute derivs
            for i in 0..N {
                match eval.derivative(i) {
                    Ok(deriv) => {
                        let grad = deriv(&inputs);
                        println!("   d/dx_{} = {}", i, grad);
                    },
                    Err(e) => println!(" ERROR: {}", e),
                }
            }
        },
        Err(e) => println!("  Failed to create evals: {}", e),
    }
    
    // Test with AD trait for comparison
    println!("\nUsing AD trait:");
    let evaluator = AllEvaluators::new(expr, N, 1);
    let gt_calculators = [PyTorchGroundTruthCalculator];
    let _ = run_custom_test(inputs.to_vec(), evaluator, &gt_calculators);
}

fn main() {
    
    // Example 1: -0.1 * x_0
    test_evalexpr_vs_ad_trait(
        "-0.1 * x_0",
        SimpleExpr::mul(
            SimpleExpr::num(-0.1),
            SimpleExpr::var("x_0")
        ),
        [1.0, 2.0]
    );
    
    // Example 2: x_0^2 + x_1^2
    test_evalexpr_vs_ad_trait(
        "x_0^2 + x_1^2",
        SimpleExpr::add(
            SimpleExpr::pow(SimpleExpr::var("x_0"), SimpleExpr::num(2.0)),
            SimpleExpr::pow(SimpleExpr::var("x_1"), SimpleExpr::num(2.0))
        ),
        [3.0, 4.0]
    );
    
    // Example 3: sin(x_0) * cos(x_1)
    test_evalexpr_vs_ad_trait(
        "sin(x_0) * cos(x_1)",
        SimpleExpr::mul(
            SimpleExpr::sin(SimpleExpr::var("x_0")),
            SimpleExpr::cos(SimpleExpr::var("x_1"))
        ),
        [0.5, 1.0]
    );
}
