// examples/build_custom_ast.rs
//
// Examples of building various AST expressions manually
// Run with: cargo run --example build_custom_ast

use fuzz_core::ast_expr::{SimpleExpr, Expr};
use fuzz_core::ast_evaluator::unified::AllEvaluators;
use fuzz_core::ast_evaluator::{SExprPrinter, InfixPrinter, SSAPrinter};
use fuzz_core::fuzz_harness::run_custom_test;
use fuzz_core::gt_calculators::PyTorchGroundTruthCalculator;

fn print_and_test(name: &str, expr: SimpleExpr, num_inputs: usize, inputs: Vec<f64>) {
    println!("\n=== {} ===", name);
    println!("S-expr: {}", SExprPrinter::print(&expr, num_inputs));
    println!("Infix:  {}", InfixPrinter::print(&expr, num_inputs));
    println!("SSA:\n{}", SSAPrinter::print(&expr));
    
    let evaluator = AllEvaluators::new(expr, num_inputs, 1);
    let gt_calculators = [PyTorchGroundTruthCalculator];
    
    println!("Testing with inputs {:?}:", inputs);
    let _ = run_custom_test(inputs, evaluator, &gt_calculators);
}

fn main() {
    // Example 1: -0.1 * x_1
    let expr1 = SimpleExpr::mul(
        SimpleExpr::num(-0.1),
        SimpleExpr::var("x_0")
    );
    print_and_test("-0.1 * x_0", expr1, 2, vec![1.0, 2.0]);
    
    // Example 2: x_0 + x_1
    let expr2 = SimpleExpr::add(
        SimpleExpr::var("x_0"),
        SimpleExpr::var("x_1")
    );
    print_and_test("x_0 + x_1", expr2, 2, vec![3.0, 4.0]);
    
    // Example 3: sin(x_0) * cos(x_1)
    let expr3 = SimpleExpr::mul(
        SimpleExpr::sin(SimpleExpr::var("x_0")),
        SimpleExpr::cos(SimpleExpr::var("x_1"))
    );
    print_and_test("sin(x_0) * cos(x_1)", expr3, 2, vec![0.5, 1.0]);
    
    // Example 4: (x_0 + x_1) * (x_0 - x_1)
    let expr4 = SimpleExpr::mul(
        SimpleExpr::add(
            SimpleExpr::var("x_0"),
            SimpleExpr::var("x_1")
        ),
        SimpleExpr::sub(
            SimpleExpr::var("x_0"),
            SimpleExpr::var("x_1")
        )
    );
    print_and_test("(x_0 + x_1) * (x_0 - x_1)", expr4, 2, vec![5.0, 3.0]);
    
    // Example 5: exp(x_0 / 10.0)
    let expr5 = SimpleExpr::exp(
        SimpleExpr::div(
            SimpleExpr::var("x_0"),
            SimpleExpr::num(10.0)
        )
    );
    print_and_test("exp(x_0 / 10.0)", expr5, 2, vec![2.0, 3.0]);
    
    // Example 6: x_0^2 + 2*x_1 + 3
    let expr6 = SimpleExpr::add(
        SimpleExpr::add(
            SimpleExpr::pow(
                SimpleExpr::var("x_0"),
                SimpleExpr::num(2.0)
            ),
            SimpleExpr::mul(
                SimpleExpr::num(2.0),
                SimpleExpr::var("x_1")
            )
        ),
        SimpleExpr::num(3.0)
    );
    print_and_test("x_0^2 + 2*x_1 + 3", expr6, 2, vec![2.0, 1.0]);
    

}
