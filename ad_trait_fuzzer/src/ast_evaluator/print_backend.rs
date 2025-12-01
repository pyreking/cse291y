// src/ast_evaluator/print_backend.rs

/// Pretty print AST using MainBackend

use crate::ast_expr::{Expr, Op1, Op2, Type};
use super::{MainBackend, evaluate, Env};

#[derive(Clone)]
pub struct SExprString(String);

impl MainBackend for SExprString {
    fn from_f64(val: f64) -> Self { 
        SExprString(format!("{}", val))
    }
    fn zero() -> Self { SExprString("0".to_string()) }
    fn one() -> Self { SExprString("1".to_string()) }
    
    fn neg(self) -> Self { SExprString(format!("(neg {})", self.0)) }
    fn sin(self) -> Self { SExprString(format!("(sin {})", self.0)) }
    fn cos(self) -> Self { SExprString(format!("(cos {})", self.0)) }
    fn tan(self) -> Self { SExprString(format!("(tan {})", self.0)) }
    fn exp(self) -> Self { SExprString(format!("(exp {})", self.0)) }
    fn log(self) -> Self { SExprString(format!("(log {})", self.0)) }
    fn sqrt(self) -> Self { SExprString(format!("(sqrt {})", self.0)) }
    fn abs(self) -> Self { SExprString(format!("(abs {})", self.0)) }
    
    fn add(self, other: Self) -> Self { SExprString(format!("(+ {} {})", self.0, other.0)) }
    fn sub(self, other: Self) -> Self { SExprString(format!("(- {} {})", self.0, other.0)) }
    fn mul(self, other: Self) -> Self { SExprString(format!("(* {} {})", self.0, other.0)) }
    fn div(self, other: Self) -> Self { SExprString(format!("(/ {} {})", self.0, other.0)) }
    fn pow(self, other: Self) -> Self { SExprString(format!("(pow {} {})", self.0, other.0)) }
}

#[derive(Clone)]
pub struct InfixString(String);

impl MainBackend for InfixString {
    fn from_f64(val: f64) -> Self { 
        InfixString(format!("{}", val))
    }
    fn zero() -> Self { InfixString("0".to_string()) }
    fn one() -> Self { InfixString("1".to_string()) }
    
    fn neg(self) -> Self { InfixString(format!("-({})", self.0)) }
    fn sin(self) -> Self { InfixString(format!("sin({})", self.0)) }
    fn cos(self) -> Self { InfixString(format!("cos({})", self.0)) }
    fn tan(self) -> Self { InfixString(format!("tan({})", self.0)) }
    fn exp(self) -> Self { InfixString(format!("exp({})", self.0)) }
    fn log(self) -> Self { InfixString(format!("ln({})", self.0)) }
    fn sqrt(self) -> Self { InfixString(format!("sqrt({})", self.0)) }
    fn abs(self) -> Self { InfixString(format!("abs({})", self.0)) }
    
    fn add(self, other: Self) -> Self { InfixString(format!("({} + {})", self.0, other.0)) }
    fn sub(self, other: Self) -> Self { InfixString(format!("({} - {})", self.0, other.0)) }
    fn mul(self, other: Self) -> Self { InfixString(format!("({} * {})", self.0, other.0)) }
    fn div(self, other: Self) -> Self { InfixString(format!("({} / {})", self.0, other.0)) }
    fn pow(self, other: Self) -> Self { InfixString(format!("({} ^ {})", self.0, other.0)) }
}

/// Sexpr
pub struct SExprPrinter;

impl SExprPrinter {
    pub fn print<Tag>(expr: &Expr<Tag>, num_inputs: usize) -> String {
        let env = Self::build_env(num_inputs);
        match evaluate::<SExprString, Tag>(expr, &env) {
            Ok(result) => result.0,
            Err(e) => format!("<error: {}>", e)
        }
    }
    
    fn build_env(num_inputs: usize) -> Env<SExprString> {
        let mut env = Env::new();
        for i in 0..num_inputs {
            env.insert(format!("x_{}", i), SExprString(format!("x_{}", i)));
        }
        env
    }
}

/// Infix
pub struct InfixPrinter;

impl InfixPrinter {
    pub fn print<Tag>(expr: &Expr<Tag>, num_inputs: usize) -> String {
        let env = Self::build_env(num_inputs);
        match evaluate::<InfixString, Tag>(expr, &env) {
            Ok(result) => result.0,
            Err(e) => format!("<error: {}>", e)
        }
    }
    
    fn build_env(num_inputs: usize) -> Env<InfixString> {
        let mut env = Env::new();
        for i in 0..num_inputs {
            env.insert(format!("x_{}", i), InfixString(format!("x_{}", i)));
        }
        env
    }
}

/// SSA for LLVM looking stuff
pub struct SSAPrinter;

impl SSAPrinter {
    pub fn print<T>(expr: &Expr<T>) -> String {
        let mut counter = 0;
        let mut statements = Vec::new();
        let result = Self::print_helper(expr, &mut counter, &mut statements);
        
        if statements.is_empty() {
            result
        } else {
            statements.push(format!("return {}", result));
            statements.join("\n")
        }
    }

    fn print_helper<T>(expr: &Expr<T>, counter: &mut usize, statements: &mut Vec<String>) -> String {
        match expr {
            Expr::Number(_, n) => format!("{}", n),
            Expr::Boolean(_, b) => format!("{}", b),
            Expr::Id(_, name) => name.clone(),
            Expr::Let(_, bindings, body) => {
                for (var, expr) in bindings {
                    let val = Self::print_helper(expr, counter, statements);
                    statements.push(format!("{} = {}", var, val));
                }
                Self::print_helper(body, counter, statements)
            }
            Expr::UnOp(_, op, expr) => {
                let arg = Self::print_helper(expr, counter, statements);
                let var_name = format!("t{}", counter);
                *counter += 1;
                
                let stmt = match op {
                    Op1::Neg => format!("{} = -{}", var_name, arg),
                    Op1::Sin => format!("{} = sin({})", var_name, arg),
                    Op1::Cos => format!("{} = cos({})", var_name, arg),
                    Op1::Tan => format!("{} = tan({})", var_name, arg),
                    Op1::Exp => format!("{} = exp({})", var_name, arg),
                    Op1::Log => format!("{} = log({})", var_name, arg),
                    Op1::Sqrt => format!("{} = sqrt({})", var_name, arg),
                    Op1::Abs => format!("{} = abs({})", var_name, arg),
                };
                statements.push(stmt);
                var_name
            }
            Expr::BinOp(_, op, left, right) => {
                let left_val = Self::print_helper(left, counter, statements);
                let right_val = Self::print_helper(right, counter, statements);
                let var_name = format!("t{}", counter);
                *counter += 1;
                
                let op_str = match op {
                    Op2::Add => "+",
                    Op2::Sub => "-",
                    Op2::Mul => "*",
                    Op2::Div => "/",
                    Op2::Pow => "**",
                };
                statements.push(format!("{} = {} {} {}", var_name, left_val, op_str, right_val));
                var_name
            }
            Expr::If(_, cond, then_br, else_br) => {
                let cond_val = Self::print_helper(cond, counter, statements);
                let var_name = format!("t{}", counter);
                *counter += 1;
                
                statements.push(format!("{} = if {} then", var_name, cond_val));
                let then_val = Self::print_helper(then_br, counter, statements);
                statements.push(format!("  {}", then_val));
                statements.push("else".to_string());
                let else_val = Self::print_helper(else_br, counter, statements);
                statements.push(format!("  {}", else_val));
                var_name
            }
            Expr::Loop(_, body) => {
                let var_name = format!("t{}", counter);
                *counter += 1;
                statements.push(format!("{} = loop", var_name));
                let body_val = Self::print_helper(body, counter, statements);
                statements.push(format!("  {}", body_val));
                var_name
            }
            Expr::Break(_, val) => {
                let val_str = Self::print_helper(val, counter, statements);
                format!("break {}", val_str)
            }
            Expr::Set(_, var, expr) => {
                let val = Self::print_helper(expr, counter, statements);
                statements.push(format!("{} = {}", var, val));
                var.clone()
            }
            Expr::Block(_, exprs) => {
                let mut last = String::new();
                for expr in exprs {
                    last = Self::print_helper(expr, counter, statements);
                }
                last
            }
            // like C cast ig
            Expr::Cast(_, typ, expr) => {
                let val = Self::print_helper(expr, counter, statements);
                let var_name = format!("t{}", counter);
                *counter += 1;
                let type_str = match typ {
                    Type::Float => "float",
                    Type::Int => "int",
                    Type::Bool => "bool",
                };
                statements.push(format!("{} = ({}) {}", var_name, type_str, val));
                var_name
            }
        }
    }
}
