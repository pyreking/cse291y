# AST Refactoring Summary

## What We've Built

A **modular, extensible AST-based evaluation system** that eliminates code duplication and makes it easy to add new numeric backends (JAX, TensorFlow, NumPy, etc.).

## Key Design Decisions

### 1. ✅ Using `arbitrary` instead of `rand`
**Why it's better:**
- Structure-aware fuzzing (fuzzer learns AST structure)
- Automatic test case minimization (shrinking)
- Efficient byte consumption
- Standard for Rust fuzzing ecosystem

### 2. ✅ Modular file structure

```
src/ast_evaluator/
├── mod.rs              # Core trait and generic evaluator
├── ad_backend.rs       # AD trait implementation
├── pytorch_backend.rs  # PyTorch implementation  
└── unified.rs          # Combined evaluator
```

**Benefits:**
- Easy to add new backends (JAX, TensorFlow, etc.)
- Clear separation of concerns
- Each backend in its own file
- No code duplication

### 3. ✅ Unified trait with minimal duplication

```rust
pub trait NumericBackend {
    // Standard operations (same names for AD and PyTorch)
    fn sin(self) -> Self;      // ✓ Same in both
    fn cos(self) -> Self;      // ✓ Same in both
    fn sqrt(self) -> Self;     // ✓ Same in both
    fn add(self, other: Self) -> Self;  // ✓ Same in both
    
    // Only the differences
    fn pow_generic(self, other: Self) -> Self;  // Different per backend!
}
```

**AD backend:**
```rust
fn pow_generic(self, other: Self) -> Self {
    self.powf(other)  // Or self.powi(n) for integers
}
```

**PyTorch backend:**
```rust
fn pow_generic(self, other: Self) -> Self {
    PyTorchTensor(self.0.pow(&other.0))
}
```

## Code Duplication Eliminated

### Before (Macro approach):
```rust
// Had to maintain TWO implementations
macro_rules! compute_expression_ad {
    ($x:expr, $y:expr) => {
        ($x.sin() * $y.exp()).powi(2) + $x.sqrt()
    };
}

macro_rules! compute_expression_pytorch {
    ($x:expr, $y:expr) => {
        ($x.sin() * $y.exp()).pow_tensor_scalar(2.0) + $x.sqrt()
    };
}
```

### After (AST approach):
```rust
// Single AST definition!
let expr = E::add(
    E::pow(E::mul(E::sin(E::var("x")), E::exp(E::var("y"))), E::num(2.0)),
    E::sqrt(E::var("x"))
);

// Works for BOTH AD and PyTorch via UnifiedEvaluator
let evaluator = UnifiedEvaluator::new(expr, 2, 1);
```

## How to Add a New Backend (e.g., JAX)

1. **Create `src/ast_evaluator/jax_backend.rs`:**
```rust
pub struct JaxArray(/* JAX type */);

impl NumericBackend for JaxArray {
    fn sin(self) -> Self { /* JAX's sin */ }
    fn pow_generic(self, other: Self) -> Self { /* JAX's pow */ }
    // ... implement other methods
}
```

2. **Add to `mod.rs`:**
```rust
pub mod jax_backend;
pub use jax_backend::JaxEvaluator;
```

3. **Done!** The generic `evaluate()` function works automatically.

## Usage Example

```rust
use ad_trait_fuzzer::ast_expr::SimpleExpr as E;
use ad_trait_fuzzer::ast_evaluator::UnifiedEvaluator;

// Build expression once
let expr = E::add(
    E::pow(E::mul(E::sin(E::var("x")), E::exp(E::var("y"))), E::num(2.0)),
    E::sqrt(E::var("x"))
);

// Create unified evaluator (works for both AD and PyTorch!)
let evaluator = UnifiedEvaluator::new(expr, 2, 1);

// Use in fuzzing - automatically tests both implementations!
let harness = FuzzHarness::new(evaluator);
```

## Benefits Summary

✅ **Zero code duplication** - Define expression once, works everywhere  
✅ **Easy to extend** - Add new backends by implementing one trait  
✅ **Type-safe** - Rust's type system catches errors at compile time  
✅ **Efficient fuzzing** - `arbitrary` integration for smart input generation  
✅ **Maintainable** - Each backend in its own file, clear organization  
✅ **Future-proof** - Can add JAX, TensorFlow, NumPy, etc. trivially  

## What's Different from Before?

| Aspect | Old (Macros) | New (AST) |
|--------|-------------|-----------|
| **Duplication** | 2 macros per expression | Single AST definition |
| **Extensibility** | Hard to add backends | Add one file per backend |
| **Random generation** | Complex with macros | Built-in with `arbitrary` |
| **Maintainability** | Code spread across macros | Modular file structure |
| **Power operation** | Separate macros | Single `pow_generic` trait method |

This architecture is **production-ready** and can scale to testing dozens of AD libraries! 🎯
