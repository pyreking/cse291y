# Ad-Trait Fuzzer (CSE291Y Project)

A differential fuzzing project built for UCSD's CSE 291Y class, focused on finding correctness bugs in Rust's automatic differentiation (AD) library [`ad_trait`](https://docs.rs/ad_trait/latest/ad_trait/).

The core mechanism is a **differential fuzzing oracle** that compares the gradients computed by the forward-mode and reverse-mode implementations of the target library. The entire fuzzing logic has been packaged into a reusable library crate (`fuzz_core`).

---

## Project Architecture

The project is structured as a library crate (`fuzz_core`) consumed by the `libFuzzer` target. This design separates core logic (evaluation, comparison) from the I/O layer (input decoding, test generation).

| Module | Responsibility |
| :--- | :--- |
| **`fuzz_harness`** | Defines core traits (`Calculator`, `GroundTruthCalculator`) and the master entry function (`run_ad_tests`) that executes the entire fuzzing flow. |
| **`rpn_evaluator`** | Contains the generic logic to execute Reverse Polish Notation (RPN) expressions for both AD types and PyTorch Tensors. |
| **`test_generator`** | Programmatically creates new, valid, random RPN expressions (`TestDefinition` structs) on the fly for dynamic fuzzing. |
| **`oracles`** | Houses the comparison logic (`Oracle` trait). Includes checks for **Reverse vs Forward AD** consistency and **AD vs Ground Truth (PyTorch)** consistency. |
| **`gt_calculators`** | Contains concrete implementations (e.g., `PyTorchGroundTruthCalculator`) for generating reference derivatives using external libraries. |

---

## Getting Started

### Prerequisites

This project relies on the **Rust Nightly** toolchain and the **`cargo-fuzz`** utility, which uses LLVM's `libFuzzer`.

1.  **Install Rust/Cargo:**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    . "$HOME/.cargo/env"
    sudo apt install cargo
    ```

2.  **Install Rust Nightly:**
    ```bash
    rustup install nightly
    rustup default nightly
    ```

3.  **Install `cargo-fuzz`:**
    ```bash
    cargo +nightly install cargo-fuzz
    ```

4.  **Install Linux build dependencies:**
    The fuzzer needs essential build tools and the LLVM `clang` compiler components for the sanitizers to link correctly.
    ```bash
    sudo apt update
    sudo apt install clang make build-essential
    ```

---

## Build Instructions (Recommended platform: WSL/Linux)

The project is structured with the fuzzer targets located in the `fuzz/` directory.

### 1. Initial Build

Build the fuzzer target (fuzz/fuzz_target_1.rs is the only target for now).

```cargo +nightly fuzz build fuzz_target_1```

### 2. Run the Fuzzer

To begin the search for bugs, simply run the target. You can control the runtime behavior using environment variables:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `FUZZ_MODE` | `PanicOnFirstError` | Use `continuous` to log errors but keep running; otherwise, panics on first failure. |
| `FUZZ_TESTS` | `1` | Number of random RPN expressions to generate per fuzzer input. |
| `FUZZ_ORACLE` | `all` | Controls which oracle checks run: `all`, `rev_fwd`, `rev_gt`, or `fwd_gt`. |

**Example Run:**

Run with 10 random RPN tests per input, checking only Reverse vs Forward AD consistency
``FUZZ_TESTS=10 FUZZ_ORACLE=rev_fwd cargo +nightly fuzz run fuzz_target_1``

### 3. Reproduce a Crash
Use the full path to a crashing artifact file to reproduce the bug (e.g., for local debugging):

cargo +nightly fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-d538d6551b3cbcf05b4886cdcbba29199963ff34

## Known issue: Building on Windows

The cargo-fuzz toolchain, which relies on LLVM's libFuzzer, has linking and compatibility issues on Windows when using the MSVC toolchain (x86_64-pc-windows-msvc).

Issue: The linker (link.exe) fails to resolve necessary Sanitizer Coverage (Sancov) symbols (__start___sancov_cntrs, __start___sancov_pcs, etc.) which are required by libFuzzer. This leads to a fatal error LNK1120: 4 unresolved externals.

### Temporary solution:

* The recommended method for building the project is to use a Linux environment or WSL. The Linux toolchain handles the linking issue.