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
    let rustflags = encoded_rustflags
        .split('\u{1f}')
        .filter(|flag| !flag.is_empty())
        .collect::<Vec<_>>();
    let rendered_rustflags = if rustflags.is_empty() {
        "<none>".to_owned()
    } else {
        format!(
            "hexvec:{}",
            rustflags
                .iter()
                .map(|flag| hex_encode(flag.as_bytes()))
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    let target_cpu = if rustflags_have_value(&rustflags, "target-cpu", "native") {
        "native"
    } else {
        "baseline"
    };
    let explicit_target_features = rustflag_values(&rustflags, "target-feature");
    let target_features = [
        (!target_features.is_empty()).then_some(target_features.as_str()),
        (!explicit_target_features.is_empty()).then_some(explicit_target_features.as_str()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(",");
    let target_features = if target_features.is_empty() {
        "<none>".to_owned()
    } else {
        target_features
    };
    let runtime_version =
        command_line("uname", &["-r"]).unwrap_or_else(|| format!("target-env-{target_env}"));
    let libc = match (target_os.as_str(), target_env.as_str()) {
        ("linux", "gnu") => command_line("getconf", &["GNU_LIBC_VERSION"])
            .unwrap_or_else(|| format!("glibc@{runtime_version}")),
        ("linux", "musl") => format!("musl@{runtime_version}"),
        ("macos", _) => format!("libSystem@Darwin-{runtime_version}"),
        ("windows", "msvc") => format!("ucrt@{runtime_version}"),
        _ => "<unavailable-d2>".to_owned(),
    };
    let libm = match target_os.as_str() {
        "macos" => format!("libSystem-libm@Darwin-{runtime_version}"),
        "linux" => format!("libm@{libc}"),
        "windows" => format!("ucrt-libm@{runtime_version}"),
        _ => "<unavailable-d2>".to_owned(),
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
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_ENCODED_RUSTFLAGS={rendered_rustflags}");
    Ok(())
}

fn rustflag_values(flags: &[&str], key: &str) -> String {
    let mut values = Vec::new();
    let mut index = 0;
    while index < flags.len() {
        let flag = flags[index];
        let maybe_value = flag
            .strip_prefix("-C")
            .and_then(|value| value.strip_prefix(key))
            .and_then(|value| value.strip_prefix('='))
            .or_else(|| {
                (flag == "-C")
                    .then(|| flags.get(index + 1).copied())
                    .flatten()
                    .and_then(|value| value.strip_prefix(key))
                    .and_then(|value| value.strip_prefix('='))
            });
        if let Some(value) = maybe_value {
            values.push(value);
        }
        index += 1;
    }
    values.join(",")
}

fn rustflags_have_value(flags: &[&str], key: &str, expected: &str) -> bool {
    rustflag_values(flags, key)
        .split(',')
        .any(|value| value == expected)
}

fn command_line(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
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
