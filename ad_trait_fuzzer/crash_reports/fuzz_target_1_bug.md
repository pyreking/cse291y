thread '<unnamed>' (3354) panicked at fuzz_target_1.rs:94:13:
Differential Fuzzing Failure: Gradients differ greatly! Reverse: 26585916608555260000000000000, Forward: 26585916608555253000000000000
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
==3354== ERROR: libFuzzer: deadly signal
    #0 0x632e6b6c6ee1 in __sanitizer_print_stack_trace /rustc/llvm/src/llvm-project/compiler-rt/lib/asan/asan_stack.cpp:87:3
    #1 0x632e6b71babe in fuzzer::PrintStackTrace() /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerUtil.cpp:210:38
    #2 0x632e6b70ebe9 in fuzzer::Fuzzer::CrashCallback() /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerLoop.cpp:231:18
    #3 0x632e6b70ebe9 in fuzzer::Fuzzer::CrashCallback() /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerLoop.cpp:226:6
    #4 0x758f7444251f  (/lib/x86_64-linux-gnu/libc.so.6+0x4251f) (BuildId: 4f7b0c955c3d81d7cac1501a2498b69d1d82bfe7)
    #5 0x758f744969fb in pthread_kill (/lib/x86_64-linux-gnu/libc.so.6+0x969fb) (BuildId: 4f7b0c955c3d81d7cac1501a2498b69d1d82bfe7)
    #6 0x758f74442475 in gsignal (/lib/x86_64-linux-gnu/libc.so.6+0x42475) (BuildId: 4f7b0c955c3d81d7cac1501a2498b69d1d82bfe7)
    #7 0x758f744287f2 in abort (/lib/x86_64-linux-gnu/libc.so.6+0x287f2) (BuildId: 4f7b0c955c3d81d7cac1501a2498b69d1d82bfe7)
    #8 0x632e6b763b19 in std::sys::pal::unix::abort_internal::hbf9815254ee075fc /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/sys/pal/unix/mod.rs:365:14
    #9 0x632e6b7661b8 in std::process::abort::h1395eb7cbc885c86 /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/process.rs:2511:5
    #10 0x632e6b706f64 in libfuzzer_sys::initialize::_$u7b$$u7b$closure$u7d$$u7d$::h00aedc06d71bac57 /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/src/lib.rs:94:9
    #11 0x632e6b766ace in _$LT$alloc..boxed..Box$LT$F$C$A$GT$$u20$as$u20$core..ops..function..Fn$LT$Args$GT$$GT$::call::h97607c3467d3b8bc /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/alloc/src/boxed.rs:2019:9
    #12 0x632e6b766ace in std::panicking::panic_with_hook::hd8308519547a2d36 /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/panicking.rs:842:13
    #13 0x632e6b766929 in std::panicking::panic_handler::_$u7b$$u7b$closure$u7d$$u7d$::h18aececbc13641b9 /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/panicking.rs:707:13
    #14 0x632e6b764cf8 in std::sys::backtrace::__rust_end_short_backtrace::h9089c54d7ac9e38f /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/sys/backtrace.rs:174:18
    #15 0x632e6b75685c in __rustc::rust_begin_unwind /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/panicking.rs:698:5
    #16 0x632e6b79699f in core::panicking::panic_fmt::h9ee86f2a0ff782c6 /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/core/src/panicking.rs:80:14
    #17 0x632e6b6f3f2d in fuzz_target_1::_::__libfuzzer_sys_run::h95abe90f206295f6 /mnt/c/Users/the-grand-line/Desktop/CSE291Y/cse291y/ad-trait-fuzzer/fuzz/fuzz_target_1.rs
    #18 0x632e6b6fd678 in rust_fuzzer_test_input /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/src/lib.rs:276:60
    #19 0x632e6b707915 in {closure#0} /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/src/lib.rs:62:9
    #20 0x632e6b707915 in std::panicking::catch_unwind::do_call::h2b8f50dbbae07f07 /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/panicking.rs:590:40
    #21 0x632e6b708bc8 in __rust_try libfuzzer_sys.c35f3578af87e0fd-cgu.0
    #22 0x632e6b70698d in catch_unwind<i32, libfuzzer_sys::test_input_wrap::{closure_env#0}> /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/panicking.rs:553:19
    #23 0x632e6b70698d in catch_unwind<libfuzzer_sys::test_input_wrap::{closure_env#0}, i32> /rustc/2aaa62b89d22b570e560731b03e3d2d6f5c3bbce/library/std/src/panic.rs:359:14
    #24 0x632e6b70698d in LLVMFuzzerTestOneInput /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/src/lib.rs:60:22
    #25 0x632e6b70f150 in fuzzer::Fuzzer::ExecuteCallback(unsigned char const*, unsigned long) /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerLoop.cpp:619:15
    #26 0x632e6b714863 in fuzzer::Fuzzer::RunOne(unsigned char const*, unsigned long, bool, fuzzer::InputInfo*, bool, bool*) /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerLoop.cpp:516:22
    #27 0x632e6b715908 in fuzzer::Fuzzer::MutateAndTestOne() /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerLoop.cpp:765:25
    #28 0x632e6b717d37 in fuzzer::Fuzzer::Loop(std::vector<fuzzer::SizedFile, std::allocator<fuzzer::SizedFile> >&) /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerLoop.cpp:910:21
    #29 0x632e6b72ac04 in fuzzer::FuzzerDriver(int*, char***, int (*)(unsigned char const*, unsigned long)) /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerDriver.cpp:915:10
    #30 0x632e6b72cf46 in main /home/austin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libfuzzer-sys-0.4.10/libfuzzer/FuzzerMain.cpp:20:30
    #31 0x758f74429d8f  (/lib/x86_64-linux-gnu/libc.so.6+0x29d8f) (BuildId: 4f7b0c955c3d81d7cac1501a2498b69d1d82bfe7)
    #32 0x758f74429e3f in __libc_start_main (/lib/x86_64-linux-gnu/libc.so.6+0x29e3f) (BuildId: 4f7b0c955c3d81d7cac1501a2498b69d1d82bfe7)
    #33 0x632e6b632104 in _start (/mnt/c/Users/the-grand-line/Desktop/CSE291Y/cse291y/ad-trait-fuzzer/fuzz/target/x86_64-unknown-linux-gnu/release/fuzz_target_1+0x71104) (BuildId: 9f8cb5b55df511669ed442ae0214eb8c6643128e)

NOTE: libFuzzer has rudimentary signal handlers.
      Combine libFuzzer with AddressSanitizer or similar for better crash reports.
SUMMARY: libFuzzer: deadly signal
MS: 3 ChangeByte-ChangeBit-InsertRepeatedBytes-; base unit: e129f27c5103bc5cc44bcdf0a15e160d445066ff
0x0,0x0,0x0,0x0,0x0,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x40,0x0,0x0,0x0,0x0,0x0,0x5d,0x0,0x0,0x0,0x0,0x8,
\000\000\000\000\000@@@@@@@@@@@@@@@@@@@@@\000\000\000\000\000]\000\000\000\000\010
artifact_prefix='/mnt/c/Users/the-grand-line/Desktop/CSE291Y/cse291y/ad-trait-fuzzer/fuzz/artifacts/fuzz_target_1/'; Test unit written to /mnt/c/Users/the-grand-line/Desktop/CSE291Y/cse291y/ad-trait-fuzzer/fuzz/artifacts/fuzz_target_1/crash-d538d6551b3cbcf05b4886cdcbba29199963ff34
Base64: AAAAAABAQEBAQEBAQEBAQEBAQEBAQEBAQEAAAAAAAF0AAAAACA==

────────────────────────────────────────────────────────────────────────────────

Failing input:

        fuzz/artifacts/fuzz_target_1/crash-d538d6551b3cbcf05b4886cdcbba29199963ff34

Output of `std::fmt::Debug`:

        [0, 0, 0, 0, 0, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 0, 0, 0, 0, 0, 93, 0, 0, 0, 0, 8]

Reproduce with:

        cargo fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-d538d6551b3cbcf05b4886cdcbba29199963ff34

Minimize test case with:

        cargo fuzz tmin fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-d538d6551b3cbcf05b4886cdcbba29199963ff34

────────────────────────────────────────────────────────────────────────────────