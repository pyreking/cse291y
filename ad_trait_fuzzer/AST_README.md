# AST-Based Expression Evaluation

This module provides an Abstract Syntax Tree (AST) based approach for defining and evaluating mathematical expressions that can be compiled to both:
1. **Rust AD types** (forward and reverse automatic differentiation)
2. **PyTorch tensors** (for ground truth gradients via autograd)

## Why AST instead of RPN?

- **More expressive**: Can represent complex control flow (if/then/else, loops, let bindings)
- **Easier to generate**: Random AST generation is more straightforward than valid RPN
- **Better for program synthesis**: Can generate full programs, not just expressions
- **Type-safe**: Can add type checking and inference
- **Extensible**: Easy to add new operations and language features

## Architecture

### Core Components

1. **`ast_expr.rs`** - AST definition
   - `Expr<T>` - Main expression type with metadata tag `T`
   - `Op1`, `Op2` - Unary and binary operators
   - Helper constructors for building expressions

2. **`ast_evaluator.rs`** - Evaluation engine
   - `AstEvalType` trait - Unified interface for AD and Tensor types
   - `evaluate_ast()` - Generic evaluator that works with both backends
   - `AstEvaluator` - Implements `Calculator` and `PyTorchComputable`

3. **`ast_generator.rs`** - Random generation
   - `AstGenerator` - Configurable random AST generator
   - `generate_from_bytes()` - Generate from fuzzer input

## Usage Example

### Manual Expression Construction

```rust
use ad_trait_fuzzer::ast_expr::SimpleExpr as E;
use ad_trait_fuzzer::ast_evaluator::AstEvaluator;

// Build: z = (sin(x) * exp(y))^2 + sqrt(x)
let sin_x = E::sin(E::var("x"));
let exp_y = E::exp(E::var("y"));
let mul = E::mul(sin_x, exp_y);
let pow2 = E::pow(mul, E::num(2.0));
let sqrt_x = E::sqrt(E::var("x"));
let expr = E::add(pow2, sqrt_x);

// Create evaluator
let evaluator = AstEvaluator {
    expr,
    num_inputs: 2,
    num_outputs: 1,
};

// Use in fuzzing harness
let harness = FuzzHarness::new(evaluator);
```

### Random Generation from Fuzzer Input

```rust
use ad_trait_fuzzer::ast_generator::{generate_from_bytes, AstGenConfig};

// Configure generator
let config = AstGenConfig {
    max_depth: 5,
    max_variables: 2,
    allow_division: true,
    allow_power: true,
    allow_log: false,  // Avoid numerical issues
};

// Generate from fuzzer bytes
if let Some(expr) = generate_from_bytes(data, config) {
    let evaluator = AstEvaluator {
        expr,
        num_inputs: 2,
        num_outputs: 1,
    };
    // Use evaluator...
}
```

## Supported Operations

### Unary Operations
- `-x` (negation)
- `sin(x)`, `cos(x)`, `tan(x)`
- `exp(x)`, `log(x)`, `sqrt(x)`, `abs(x)`

### Binary Operations
- `+`, `-`, `*`, `/`
- `^` (power)

### Control Flow (Extensible)
- `let` bindings
- `if`/`then`/`else`
- Blocks
- (Future: loops, breaks, function calls)

## How It Works

The key insight is that the **same AST** can be evaluated with different numeric backends:

```rust
// For AD types (e.g., adr, adfn<1>)
impl<T: AD> AstEvalType for T {
    fn sin(self) -> Self { self.sin() }
    fn pow(self, other: Self) -> Self { self.powf(other) }
    // ...
}

// For PyTorch tensors
impl AstEvalType for AstTensor {
    fn sin(self) -> Self { AstTensor(self.0.sin()) }
    fn pow(self, other: Self) -> Self { AstTensor(self.0.pow(&other.0)) }
    // ...
}
```

The evaluator traverses the AST and calls the appropriate trait methods, ensuring both implementations compute **exactly the same mathematical operations**.

## Advantages Over Macros

The previous macro approach (`compute_expression_ad!` and `compute_expression_pytorch!`) required maintaining two separate implementations. The AST approach:

✅ **Single source of truth** - One AST, multiple backends
✅ **Random generation** - Easy to generate random programs
✅ **Composable** - Can build complex expressions programmatically  
✅ **Debuggable** - Can print, inspect, and transform ASTs
✅ **Type-checkable** - Can add static analysis

## Migration from RPN

The RPN evaluator (`rpn_evaluator.rs`) is still available for backward compatibility. To migrate:

1. **RPN** → **AST**: Parse RPN tokens into an AST
2. **Macro** → **AST**: Convert macro invocations to AST construction
3. **Direct use**: Use the AST evaluator directly in new code

## Future Extensions

- **Type inference**: Infer types and catch errors before evaluation
- **Optimization**: Constant folding, dead code elimination
- **More operations**: Matrix operations, conditionals, loops
- **Program synthesis**: Generate full programs with state
- **Property-based testing**: Generate expressions satisfying properties
- **Gradient verification**: Symbolic differentiation for comparison

## Performance Notes

The AST evaluation has a small overhead compared to direct computation, but:
- Still fast enough for fuzzing (microseconds per evaluation)
- The flexibility and correctness guarantees are worth it
- Can optimize hot paths if needed (e.g., JIT compilation)

## Testing

Run the example fuzz target:
```bash
cargo +nightly fuzz run fuzz_target_ast_example
```

This will generate random expressions and verify that AD and PyTorch compute the same gradients!
