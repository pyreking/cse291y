// src/ast_expr.rs

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Op2 {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Pow,      // ^
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op1 {
    Neg,      // -x
    Sin,      // sin(x)
    Cos,      // cos(x)
    Tan,      // tan(x)
    Exp,      // exp(x)
    Log,      // log(x)
    Sqrt,     // sqrt(x)
    Abs,      // abs(x)
}

/// Type annots (for future use for if conditions, type checking, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Float,
    Int,
    Bool,
}

/// Main AST Expr type
/// T is a tag/metadata type
#[derive(Debug, Clone)]
pub enum Expr<T> {
    Number(T, f64),
    
    Boolean(T, bool),
    
    Id(T, String),
    
    /// Let binding: let [(var1, expr1), (var2, expr2), ...] in body
    Let(T, Vec<(String, Expr<T>)>, Box<Expr<T>>),
    
    UnOp(T, Op1, Box<Expr<T>>),
    
    BinOp(T, Op2, Box<Expr<T>>, Box<Expr<T>>),
    
    If(T, Box<Expr<T>>, Box<Expr<T>>, Box<Expr<T>>),
    
    /// Loop?
    Loop(T, Box<Expr<T>>),
    
    /// Break
    Break(T, Box<Expr<T>>),
    
    /// Set/assign a var
    Set(T, String, Box<Expr<T>>),
    
    /// multiple lines of expressions
    Block(T, Vec<Expr<T>>),
    
    /// Type cast?
    Cast(T, Type, Box<Expr<T>>),
}

impl<T> Expr<T> {
    /// Get the tag/metadata from any expression
    pub fn tag(&self) -> &T {
        match self {
            Expr::Number(t, _) => t,
            Expr::Boolean(t, _) => t,
            Expr::Id(t, _) => t,
            Expr::Let(t, _, _) => t,
            Expr::UnOp(t, _, _) => t,
            Expr::BinOp(t, _, _, _) => t,
            Expr::If(t, _, _, _) => t,
            Expr::Loop(t, _) => t,
            Expr::Break(t, _) => t,
            Expr::Set(t, _, _) => t,
            Expr::Block(t, _) => t,
            Expr::Cast(t, _, _) => t,
        }
    }
}

/// Simple unit type for tags when we don't need metadata
pub type SimpleExpr = Expr<()>;

impl SimpleExpr {
    /// Helper constructors for common expressions without metadata
    
    pub fn num(val: f64) -> Self {
        Expr::Number((), val)
    }
    
    pub fn var(name: impl Into<String>) -> Self {
        Expr::Id((), name.into())
    }
    
    pub fn add(left: SimpleExpr, right: SimpleExpr) -> Self {
        Expr::BinOp((), Op2::Add, Box::new(left), Box::new(right))
    }
    
    pub fn sub(left: SimpleExpr, right: SimpleExpr) -> Self {
        Expr::BinOp((), Op2::Sub, Box::new(left), Box::new(right))
    }
    
    pub fn mul(left: SimpleExpr, right: SimpleExpr) -> Self {
        Expr::BinOp((), Op2::Mul, Box::new(left), Box::new(right))
    }
    
    pub fn div(left: SimpleExpr, right: SimpleExpr) -> Self {
        Expr::BinOp((), Op2::Div, Box::new(left), Box::new(right))
    }
    
    pub fn pow(base: SimpleExpr, exp: SimpleExpr) -> Self {
        Expr::BinOp((), Op2::Pow, Box::new(base), Box::new(exp))
    }
    
    pub fn sin(expr: SimpleExpr) -> Self {
        Expr::UnOp((), Op1::Sin, Box::new(expr))
    }
    
    pub fn cos(expr: SimpleExpr) -> Self {
        Expr::UnOp((), Op1::Cos, Box::new(expr))
    }
    
    pub fn exp(expr: SimpleExpr) -> Self {
        Expr::UnOp((), Op1::Exp, Box::new(expr))
    }
    
    pub fn sqrt(expr: SimpleExpr) -> Self {
        Expr::UnOp((), Op1::Sqrt, Box::new(expr))
    }
    
    pub fn neg(expr: SimpleExpr) -> Self {
        Expr::UnOp((), Op1::Neg, Box::new(expr))
    }

    pub fn log(expr: SimpleExpr) -> Self {
        Expr::UnOp((), Op1::Log, Box::new(expr))
    }
}

/// Environment for variable bindings during evaluation
pub type Env<T> = HashMap<String, T>;
