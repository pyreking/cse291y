// src/test_generator.rs

use rand::prelude::*;
use crate::test_definition::TestDefinition;

const MAX_TOKENS: usize = 7;
const TERMINAL_TOKENS: &[&str] = &["x", "y", "2"];
const UNARY_OPS: &[&str] = &["sin", "exp", "sqrt"];
const BINARY_OPS: &[&str] = &["+", "*", "pow"];

/// Generates a valid, randomized RPN expression and wraps it in a TestDefinition.
///
/// This function ensures the generated RPN sequence is syntactically valid by
/// tracking the stack depth during generation.
pub fn generate_random_test(rng: &mut impl Rng) -> TestDefinition {
    let mut tokens: Vec<String> = Vec::new();
    let mut stack_depth: i32 = 0; // Tracks the number of available operands

    // 1. Initial terminal (must start with an operand)
    let first_terminal = TERMINAL_TOKENS.choose(rng).unwrap();
    tokens.push(first_terminal.to_string());
    stack_depth += 1;

    // 2. Generate remaining tokens to build the expression
    let num_tokens = rng.gen_range(2..=MAX_TOKENS);
    for _ in 0..num_tokens {
        // Logic to maintain a balanced RPN stack: prioritize operators if stack is deep,
        // or operands if stack is shallow.
        let push_operand = (stack_depth < 2) || rng.gen_bool(0.4);

        if push_operand {
            // Push Operand
            let token = TERMINAL_TOKENS.choose(rng).unwrap();
            tokens.push(token.to_string());
            stack_depth += 1;
        } else {
            // Push Operator
            if stack_depth >= 2 && rng.gen_bool(0.7) {
                // Binary Op (70% chance if 2+ operands available)
                let op = BINARY_OPS.choose(rng).unwrap();
                tokens.push(op.to_string());
                stack_depth -= 1;
            } else if stack_depth >= 1 {
                // Unary Op
                let op = UNARY_OPS.choose(rng).unwrap();
                tokens.push(op.to_string());
                // Stack depth remains the same (1 consumed, 1 result pushed)
            } else {
                // Fallback: push an operand if we cannot push an operator
                let token = TERMINAL_TOKENS.choose(rng).unwrap();
                tokens.push(token.to_string());
                stack_depth += 1;
            }
        }
    }

    // 3. Ensure final stack depth is exactly 1 (a single, complete result)
    while stack_depth > 1 {
        // Use a binary operator to combine the top two operands
        let op = BINARY_OPS.choose(rng).unwrap();
        tokens.push(op.to_string());
        stack_depth -= 1;
    }
    
    // Create the definition struct
    TestDefinition {
        name: format!("AutoTest_{}", tokens.join("_")),
        description: "Randomly generated RPN expression.".to_string(),
        expression_rpn: tokens,
        num_inputs: 2,
        num_outputs: 1,
    }
}