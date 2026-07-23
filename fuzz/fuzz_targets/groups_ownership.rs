#![no_main]

//! Fuzzes bounded particle-group ownership and invalidation behavior.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _disposition = liquidfun_fuzz::fuzz_groups_ownership(data);
});
