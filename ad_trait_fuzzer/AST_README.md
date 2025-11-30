# AST Expr Eval

AST based approach for defining and evaluating mathematical expressions that can be compiled to both:
1. **Rust AD types** (forward and reverse automatic differentiation)
2. **PyTorch tensors** (for ground truth gradients via autograd)

## Arch

### Core

1. **`ast_expr.rs`** - AST def
   - `Expr<Tag>` - Expression tree with generic metadata tag
   - `Op1`, `Op2` - Unary/binary operators (sin, cos, add, mul, pow, etc.)

2. **`ast_evaluator/`** - Multi-backend evaluation
   - `MainBackend` trait - interface for numeric operations
   - `ad_backend.rs` - Implements MainBackend for any `T: AD`
   - `pytorch_backend.rs` - Implements MainBackend for PyTorch tensors
   - `unified.rs` - `AllEvaluators` bundles both backends for the same expression
   - `evaluate()` - Generic traversal function working with any MainBackend

3. **`ast_generator.rs`** - Random AST generation from fuzzer bytes
   - Uses `arbitrary` crate to convert raw bytes into AST
   - config depth, operations, and complexity 

## Quick Start

### Running the AST Fuzzer

```bash
cd ad_trait_fuzzer
cargo +nightly fuzz run fuzz_target_ast
```

### Configuration via Environment Variables

```bash
# Set maximum AST depth (default: 4)
AST_MAX_DEPTH=5 cargo +nightly fuzz run fuzz_target_ast

# Disable risky operations
AST_ALLOW_LOG=false cargo +nightly fuzz run fuzz_target_ast

# Select specific oracle checks
FUZZ_ORACLE=rev_fwd cargo +nightly fuzz run fuzz_target_ast
# Options: all, rev_fwd, rev_gt, fwd_gt
```

## Usage Example

### Random Generation from Fuzzer Input

```rust
use fuzz_core::ast_generator::{generate_from_bytes, AstGenConfig};
use fuzz_core::ast_evaluator::UnifiedEvaluator;

// Configure generator
let config = AstGenConfig {
    max_depth: 4,
    max_variables: 2,
    allow_division: true,
    allow_power: true,
    allow_log: false,  // Disabled by default - can cause numerical issues
};

// Generate AST from fuzzer bytes
let expr = generate_from_bytes(data, config)?;

// Create unified evaluator (supports both AD and PyTorch)
let evaluator = UnifiedEvaluator::new(expr, 2, 1);

// Use in fuzzing harness
run_ad_tests(inputs, evaluator, &oracles, &gt_calculators, mode)?;
```

## How It Works

The key insight: **one AST, multiple backends via trait abstraction**.

```
                         Expr<Tag>
                             |
                    evaluate(&expr, env)
                             |
                    MainBackend trait
                       /           \
                      /             \
          impl<T: AD> for T    PyTorchTensor
                 |                   |
         AD operations          Tensor ops
       (forward/reverse)       (via libtorch)
```

### The MainBackend Trait

Both AD types and PyTorch tensors implement the same interface:

```rust
trait MainBackend {
    fn sin(self) -> Self;
    fn pow(self, other: Self) -> Self;
    fn add(self, other: Self) -> Self;
    // ... all math operations
}
```

**AD Backend**: Forwards to AD trait methods (`self.sin()`, `self.powf()`)
**PyTorch Backend**: Wraps tensor operations (`Tensor::sin()`, `Tensor::pow()`)

### AllEvaluators

Bundles both backends for the same expression:

```rust
struct AllEvaluators<Tag> {
    ad_eval: AdEvaluator<Tag>,        // For Calculator trait
    pytorch_eval: PyTorchEvaluator<Tag>,  // For PyTorchComputable trait
}
```

This allows the fuzzer to:
1. Evaluate with AD types (forward/reverse)
2. Evaluate with PyTorch (ground truth)
3. Compare results via oracles

## Crash Detection

When a crash occurs, the fuzzer prints:
- The exact AST expression that caused the failure
- Input values (x, y)
- Error message from the oracle

Example output:
```
=== CRASH DETECTED ===
Expression that caused the crash:
BinOp(
    (),
    Pow,
    UnOp((), Sin, Id((), "x")),
    Id((), "y")
)
Inputs: x=1.5, y=2.0
Error: Oracle check failed (Rev vs PyTorch): ...
======================
```

The raw bytes are saved to `fuzz/artifacts/fuzz_target_ast/` for reproduction.

## Performance Notes (gpt generated)

The AST evaluation has a small overhead compared to direct computation, but:
- Still fast enough for fuzzing (microseconds per evaluation)
- The flexibility and correctness guarantees are worth it
- Can optimize hot paths if needed (e.g., JIT compilation)

## Testing

Run the example fuzz target:
```bash
cargo +nightly fuzz run fuzz_target_ast_example
```
 lol