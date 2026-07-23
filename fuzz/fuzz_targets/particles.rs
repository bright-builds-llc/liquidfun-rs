#![no_main]

//! Fuzzes bounded particle-system mutation programs.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _disposition = liquidfun_fuzz::fuzz_particles(data);
});
