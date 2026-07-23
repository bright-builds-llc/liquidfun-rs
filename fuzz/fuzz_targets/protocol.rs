#![no_main]

//! Fuzzes strict protocol decoders with bounded byte inputs.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _disposition = liquidfun_fuzz::fuzz_protocol(data);
});
