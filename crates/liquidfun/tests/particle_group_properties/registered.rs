//! Exact reviewed inputs, including a byte-identical packaged-crate copy.

use std::{collections::BTreeMap, path::Path};

const INPUT: &str = include_str!("phase14-particle-groups.txt");
type Records = BTreeMap<String, (u64, Vec<u8>)>;

pub(super) fn load(name: &str) -> (u64, Vec<u8>) {
    let bytes = read_input(Path::new(env!("CARGO_MANIFEST_DIR")))
        .expect("registered input must be readable and match the packaged mirror");
    parse(&bytes)
        .expect("reviewed inputs are strict version 1 records")
        .remove(name)
        .expect("named reviewed record exists")
}

fn read_input(crate_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let repository = crate_root
        .ancestors()
        .nth(2)
        .expect("crate path has ancestors");
    if crate_root.ends_with("crates/liquidfun")
        && repository.join("tools/xtask/Cargo.toml").is_file()
    {
        let registered = std::fs::read_to_string(
            repository.join("scenarios/regressions/phase14-particle-groups.txt"),
        )?;
        if registered != INPUT {
            return Err("packaged mirror differs from registered bytes".into());
        }
        Ok(registered)
    } else {
        Ok(INPUT.to_owned())
    }
}

fn parse(input: &str) -> Result<Records, &'static str> {
    let mut lines = input.lines();
    if lines.next() != Some("version=1") {
        return Err("unsupported version");
    }
    let mut records = BTreeMap::new();
    for line in lines {
        let fields: Vec<_> = line.split(' ').collect();
        let [name, seed, controls] = fields.as_slice() else {
            return Err("invalid fields");
        };
        if !matches!(*name, "audited_windows" | "current_windows") {
            return Err("unknown record");
        }
        let decimal =
            |value: &str| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit());
        if !decimal(seed) {
            return Err("invalid seed");
        }
        let seed = seed.parse::<u64>().map_err(|_| "seed overflow")?;
        let controls: Vec<u8> = controls
            .split(',')
            .map(|value| {
                if !decimal(value) {
                    return Err("invalid control");
                }
                value.parse().map_err(|_| "control overflow")
            })
            .collect::<Result<_, _>>()?;
        if !(super::REQUIRED_OPERATION_KINDS.len()..=super::MAX_OPERATIONS)
            .contains(&controls.len())
        {
            return Err("generator bounds exceeded");
        }
        if records
            .insert((*name).to_owned(), (seed, controls))
            .is_some()
        {
            return Err("duplicate record");
        }
    }
    if records.len() != 2 {
        return Err("missing record");
    }
    Ok(records)
}

#[test]
fn repository_inputs_fail_closed_while_consumers_use_embedded_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let root =
        std::env::temp_dir().join(format!("liquidfun-registered-input-{}", std::process::id()));
    std::fs::create_dir(&root)?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let crate_root = root.join("crates/liquidfun");
        std::fs::create_dir_all(&crate_root)?;
        // Act / Assert: unrelated consumer workspace needs no repository inputs.
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n")?;
        assert_eq!(read_input(&crate_root)?, INPUT);
        std::fs::create_dir_all(root.join("tools/xtask"))?;
        std::fs::write(
            root.join("tools/xtask/Cargo.toml"),
            "[package]\nname=\"xtask\"\n",
        )?;
        assert!(read_input(&crate_root).is_err());
        std::fs::create_dir_all(root.join("scenarios/regressions"))?;
        let authority = root.join("scenarios/regressions/phase14-particle-groups.txt");
        std::fs::write(&authority, INPUT.replace("59", "58"))?;
        assert!(read_input(&crate_root).is_err());
        std::fs::write(&authority, INPUT)?;
        assert_eq!(read_input(&crate_root)?, INPUT);
        Ok(())
    })();
    std::fs::remove_dir_all(root)?;
    result
}

#[test]
fn rejects_malformed_duplicate_missing_and_unbounded_records() {
    // Arrange
    let first = INPUT.lines().nth(1).expect("fixture has first record");
    let invalid = [
        INPUT.replace("version=1", "version=2"),
        format!("{INPUT}{first}\n"),
        format!("version=1\n{first}\n"),
        INPUT.replace("59", "256"),
        INPUT.replace("0,0,0,0,0,0,0,0,0,0,0,0,0,59", "0"),
        INPUT.replace("4149329052036581951", "18446744073709551616"),
        INPUT.replace("audited_windows", "unknown"),
    ];
    // Act / Assert
    assert!(parse(INPUT).is_ok());
    for input in invalid {
        assert!(parse(&input).is_err());
    }
}
