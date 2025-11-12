// src/ast_generator.rs

use crate::ast_expr::{Expr, Op1, Op2};
use arbitrary::{Arbitrary, Unstructured, Error as ArbitraryError};

/// Config for AST
#[derive(Debug, Clone)]
pub struct AstGenConfig {
    pub max_depth: usize,
    pub max_variables: usize,
    pub allow_division: bool,
    pub allow_power: bool,
    pub allow_log: bool,
}

impl Default for AstGenConfig {
    fn default() -> Self {
        AstGenConfig {
            max_depth: 5,
            max_variables: 2,
            allow_division: true,
            allow_power: true,
            allow_log: false,
        }
    }
}

impl<'a> Arbitrary<'a> for Op1 {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, ArbitraryError> {
        Ok(match u.int_in_range(0..=6)? {
            0 => Op1::Neg,
            1 => Op1::Sin,
            2 => Op1::Cos,
            3 => Op1::Exp,
            4 => Op1::Sqrt,
            5 => Op1::Abs,
            _ => Op1::Tan,
        })
    }
}

impl<'a> Arbitrary<'a> for Op2 {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, ArbitraryError> {
        Ok(match u.int_in_range(0..=4)? {
            0 => Op2::Add,
            1 => Op2::Sub,
            2 => Op2::Mul,
            3 => Op2::Div,
            _ => Op2::Pow,
        })
    }
}

/// Generate AST expr with arbitrary
pub fn generate_expr_arbitrary(
    u: &mut Unstructured,
    config: &AstGenConfig,
    depth: usize,
) -> Result<Expr<()>, ArbitraryError> {
    // At max depth, only generate terminals
    if depth >= config.max_depth {
        return generate_terminal(u, config);
    }

    // Choose between terminal, unary, or binary
    match u.int_in_range(0..=2)? {
        0 => generate_terminal(u, config),
        1 => generate_unary(u, config, depth),
        _ => generate_binary(u, config, depth),
    }
}

fn generate_terminal(
    u: &mut Unstructured,
    config: &AstGenConfig,
) -> Result<Expr<()>, ArbitraryError> {
    if u.ratio(2, 5)? {
        // Gen a var
        let var_idx = u.int_in_range(0..=config.max_variables.saturating_sub(1))?;
        let name = match var_idx {
            0 => "x".to_string(),
            1 => "y".to_string(),
            _ => format!("v{}", var_idx),
        };
        Ok(Expr::Id((), name))
    } else {
        // Gen a number
        let val = match u.int_in_range(0..=4)? {
            0 => 0.0,
            1 => 1.0,
            2 => 2.0,
            3 => u.arbitrary::<f64>()?.clamp(-10.0, 10.0),
            _ => u.arbitrary::<f64>()?.abs().clamp(0.1, 5.0),
        };
        Ok(Expr::Number((), val))
    }
}

fn generate_unary(
    u: &mut Unstructured,
    config: &AstGenConfig,
    depth: usize,
) -> Result<Expr<()>, ArbitraryError> {
    let sub_expr = generate_expr_arbitrary(u, config, depth + 1)?;
    
    let mut op_choice = u.int_in_range(0..=5)?;
    
    // Skip Log if not allowed
    if !config.allow_log && op_choice >= 5 {
        op_choice = 4;
    }
    
    let op = match op_choice {
        0 => Op1::Neg,
        1 => Op1::Sin,
        2 => Op1::Cos,
        3 => Op1::Exp,
        4 => Op1::Sqrt,
        5 => Op1::Log,
        _ => Op1::Abs,
    };
    
    Ok(Expr::UnOp((), op, Box::new(sub_expr)))
}

fn generate_binary(
    u: &mut Unstructured,
    config: &AstGenConfig,
    depth: usize,
) -> Result<Expr<()>, ArbitraryError> {
    let left = generate_expr_arbitrary(u, config, depth + 1)?;
    let right = generate_expr_arbitrary(u, config, depth + 1)?;
    
    let mut num_ops = 3; // Add, Sub, Mul
    if config.allow_division {
        num_ops += 1;
    }
    if config.allow_power {
        num_ops += 1;
    }
    
    let op_choice = u.int_in_range(0..=(num_ops - 1))?;
    
    let op = match op_choice {
        0 => Op2::Add,
        1 => Op2::Sub,
        2 => Op2::Mul,
        3 if config.allow_division => Op2::Div,
        4 if config.allow_power => Op2::Pow,
        _ => Op2::Add, // Default fallback
    };
    
    Ok(Expr::BinOp((), op, Box::new(left), Box::new(right)))
}

/// Generate from fuzzer bytes using arbitrary
pub fn generate_from_bytes(data: &[u8], config: AstGenConfig) -> Result<Expr<()>, ArbitraryError> {
    let mut u = Unstructured::new(data);
    generate_expr_arbitrary(&mut u, &config, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple() {
        let config = AstGenConfig {
            max_depth: 3,
            max_variables: 2,
            ..Default::default()
        };
        
        // Test with deterministic bytes
        let data = b"test data for fuzzing expressions";
        
        match generate_from_bytes(data, config) {
            Ok(expr) => println!("Generated expression: {:?}", expr),
            Err(e) => println!("Generation failed: {:?}", e),
        }
    }
    
    #[test]
    fn test_generate_multiple() {
        let config = AstGenConfig::default();
        
        // gen multiple expressions to test variety
        for i in 0..5 {
            let data = format!("test data {}", i).into_bytes();
            if let Ok(expr) = generate_from_bytes(&data, config.clone()) {
                println!("Expression {}: {:?}", i, expr);
            }
        }
    }
}
