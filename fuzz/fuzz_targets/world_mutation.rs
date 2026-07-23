#![no_main]

//! Fuzzes bounded rigid-world mutation programs.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _disposition = liquidfun_fuzz::fuzz_world_mutation(data);
});
