# Ad-Trait Fuzzer (CSE291Y Project)

A differential fuzzing project built for UCSD's CSE 291Y class, focused on finding correctness bugs in Rust's automatic differentiation (AD) library [`ad_trait`](https://docs.rs/ad_trait/latest/ad_trait/).

The core mechanism is a **differential fuzzing oracle** that compares the gradients computed by the forward-mode and reverse-mode implementations of the target library.

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

To begin the search for bugs, simply run the target. If you have any initial example inputs (corpus), place them in fuzz/corpus/fuzz_target_1/.

```cargo +nightly fuzz run fuzz_target_1```

### 3. Reproduce a Crash
Use the full path to a crashing artifact file to reproduce the bug (e.g., for local debugging):

cargo +nightly fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-d538d6551b3cbcf05b4886cdcbba29199963ff34

## Known issue: Building on Windows

The cargo-fuzz toolchain, which relies on LLVM's libFuzzer, has linking and compatibility issues on Windows when using the MSVC toolchain (x86_64-pc-windows-msvc).

Issue: The linker (link.exe) fails to resolve necessary Sanitizer Coverage (Sancov) symbols (__start___sancov_cntrs, __start___sancov_pcs, etc.) which are required by libFuzzer. This leads to a fatal error LNK1120: 4 unresolved externals.

### Temporary solution:

* The recommended method for building the project is to use a Linux environment or WSL. The Linux toolchain handles the linking issue.