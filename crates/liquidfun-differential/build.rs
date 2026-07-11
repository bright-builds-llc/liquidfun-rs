//! Captures exact native Rust compiler, target, profile, feature, and flag provenance.

use std::{env, error::Error, io, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=src/rust_adapter.rs");
    println!("cargo:rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");

    let rustc = env::var("RUSTC")?;
    let output = Command::new(&rustc).arg("-vV").output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!("`{rustc} -vV` failed")).into());
    }
    let rustc_verbose = String::from_utf8(output.stdout)?
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" | ");
    if rustc_verbose.is_empty() {
        return Err(io::Error::other("`rustc -vV` returned no identity").into());
    }
    let rustc_version = rustc_verbose
        .split(" | ")
        .find_map(|line| line.strip_prefix("release: "))
        .ok_or_else(|| io::Error::other("`rustc -vV` omitted its release"))?;

    let target = env::var("TARGET")?;
    let host = env::var("HOST")?;
    let profile = env::var("PROFILE")?;
    let optimization = format!("O{}", env::var("OPT_LEVEL")?);
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_features = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let mut features = env::vars()
        .filter_map(|(name, _value)| name.strip_prefix("CARGO_FEATURE_").map(str::to_owned))
        .collect::<Vec<_>>();
    features.sort();
    let features = if features.is_empty() {
        "<none>".to_owned()
    } else {
        features.join(",")
    };
    let encoded_rustflags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    let encoded_rustflags = if encoded_rustflags.is_empty() {
        "<none>".to_owned()
    } else {
        format!("hex:{}", hex_encode(encoded_rustflags.as_bytes()))
    };
    let target_cpu = if encoded_rustflags.contains("target-cpu=native") {
        "native"
    } else {
        "baseline"
    };
    let target_features = if target_features.is_empty() {
        "<none>".to_owned()
    } else {
        target_features
    };
    let libc = match (target_os.as_str(), target_env.as_str()) {
        ("linux", "gnu") => "glibc",
        ("linux", "musl") => "musl",
        ("macos", _) => "libSystem",
        ("windows", "msvc") => "msvcrt",
        _ => "<unavailable-d2>",
    };
    let libm = match target_os.as_str() {
        "macos" => "libSystem",
        "linux" => "libm",
        "windows" => "msvcrt",
        _ => "<unavailable-d2>",
    };

    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_RUSTC_VV={rustc_verbose}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_RUSTC_VERSION={rustc_version}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET={target}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_HOST={host}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_PROFILE={profile}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_OPTIMIZATION={optimization}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET_CPU={target_cpu}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET_FEATURES={target_features}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET_OS={target_os}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_LIBC={libc}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_LIBM={libm}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_FEATURES={features}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_ENCODED_RUSTFLAGS={encoded_rustflags}");
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}
