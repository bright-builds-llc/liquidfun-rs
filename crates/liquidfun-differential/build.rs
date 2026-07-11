//! Captures exact native Rust compiler, target, profile, feature, and flag provenance.

use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, io, path::Path, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let repository_root = Path::new(&manifest_dir).ancestors().nth(2).ok_or_else(|| {
        io::Error::other("differential crate is not nested under repository root")
    })?;
    let source_manifest = Path::new(&manifest_dir).join("native-math-sources.txt");
    println!("cargo:rerun-if-changed={}", source_manifest.display());
    let source_digest = native_source_manifest_sha256(repository_root, &source_manifest)?;
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
    let mut target_features = env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap_or_default()
        .split(',')
        .filter(|feature| !feature.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    target_features.sort();
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
    let explicit_target_cpu = rustflag_values(&rustflags, "target-cpu");
    let target_cpu = if explicit_target_cpu.is_empty() {
        "baseline".to_owned()
    } else {
        format!("explicit={explicit_target_cpu}")
    };
    let explicit_target_features = rustflag_values(&rustflags, "target-feature");
    let cfg_target_features = if target_features.is_empty() {
        "<none>".to_owned()
    } else {
        target_features.join(",")
    };
    let explicit_target_features = if explicit_target_features.is_empty() {
        "<none>".to_owned()
    } else {
        explicit_target_features
    };
    let target_features = format!("cfg={cfg_target_features};explicit={explicit_target_features}");
    let (c_runtime, math_runtime) = platform_runtime_identity(&target_os, &target_env);

    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_RUSTC_VV={rustc_verbose}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_RUSTC_VERSION={rustc_version}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET={target}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_HOST={host}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_PROFILE={profile}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_OPTIMIZATION={optimization}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET_CPU={target_cpu}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET_FEATURES={target_features}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_TARGET_OS={target_os}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_LIBC={c_runtime}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_LIBM={math_runtime}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_FEATURES={features}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_ENCODED_RUSTFLAGS={rendered_rustflags}");
    println!("cargo:rustc-env=LIQUIDFUN_NATIVE_SOURCE_SHA256={source_digest}");
    Ok(())
}

fn platform_runtime_identity(target_os: &str, target_env: &str) -> (String, String) {
    let runtime_version =
        command_line("uname", &["-r"]).unwrap_or_else(|| format!("target-env-{target_env}"));
    let c_runtime = match (target_os, target_env) {
        ("linux", "gnu") => command_line("getconf", &["GNU_LIBC_VERSION"])
            .unwrap_or_else(|| format!("glibc@{runtime_version}")),
        ("linux", "musl") => format!("musl@{runtime_version}"),
        ("macos", _) => format!("libSystem@Darwin-{runtime_version}"),
        ("windows", "msvc") => format!("ucrt@{runtime_version}"),
        _ => "<unavailable-d2>".to_owned(),
    };
    let math_runtime = match target_os {
        "macos" => format!("libSystem-libm@Darwin-{runtime_version}"),
        "linux" => format!("libm@{c_runtime}"),
        "windows" => format!("ucrt-libm@{runtime_version}"),
        _ => "<unavailable-d2>".to_owned(),
    };
    (c_runtime, math_runtime)
}

fn native_source_manifest_sha256(
    repository_root: &Path,
    source_manifest: &Path,
) -> Result<String, Box<dyn Error>> {
    let manifest = fs::read_to_string(source_manifest)?;
    let mut hasher = Sha256::new();
    for relative in manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let path = Path::new(relative);
        if path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(io::Error::other(format!(
                "invalid native source manifest path `{relative}`"
            ))
            .into());
        }
        let bytes = fs::read(repository_root.join(path))?;
        println!(
            "cargo:rerun-if-changed={}",
            repository_root.join(path).display()
        );
        let file_digest = Sha256::digest(bytes);
        let relative_len = u64::try_from(relative.len())?;
        hasher.update(relative_len.to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(file_digest);
    }
    Ok(hex_encode(&hasher.finalize()))
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
