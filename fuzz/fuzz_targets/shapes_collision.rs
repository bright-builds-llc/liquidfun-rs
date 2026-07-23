#![no_main]

//! Fuzzes bounded shape construction and collision queries.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _disposition = liquidfun_fuzz::fuzz_shapes_collision(data);
});
