//! Full canonical bundles and mutations at the independent validator boundary.

use super::{Fixture, TestResult, hash_file, write_json};
use crate::workspace_root;
use serde_json::{Value, json};
use std::{fmt::Write, fs, path::Path};

pub(crate) fn write_bundle(retained: &Path, candidate: &str, tree: &str) -> TestResult {
    let directory = retained.join("canonical");
    fs::create_dir_all(directory.join("logs"))?;
    let workflow = fs::read_to_string(
        workspace_root().join(".github/workflows/phase13-1-canonical-native.yml"),
    )?;
    // Exercise the actual producer's inventory against the validator's independent contract.
    let commands = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("run_logged "))
        .map(|line| {
            let (name, command) = line.split_once(' ').ok_or("command must have arguments")?;
            Ok(json!({"name":name,"command":command.replace('\'', ""),"exit_code":0}))
        })
        .collect::<TestResult<Vec<_>>>()?;
    assert_eq!(commands.len(), 21);
    write_json(&directory.join("commands.json"), &json!(commands))?;
    let mut tsv = String::new();
    let mut logs = Vec::new();
    for command in &commands {
        let name = command["name"].as_str().ok_or("name must be a string")?;
        let argv = command["command"].as_str().ok_or("argv must be a string")?;
        writeln!(tsv, "{name}\t{argv}\t0")?;
        logs.push(format!("logs/{name}.log"));
    }
    fs::write(directory.join("command-exits.tsv"), tsv)?;
    for name in [
        "install-rust",
        "install-llvm",
        "install-build-tools",
        "tool-identities",
    ] {
        logs.push(format!("logs/{name}.log"));
    }
    logs.sort();
    let mut digests = String::new();
    for relative in logs {
        let log = directory.join(&relative);
        fs::write(&log, format!("canonical fixture {relative}\n"))?;
        writeln!(digests, "{}  {relative}", hash_file(&log)?)?;
    }
    fs::write(directory.join("logs.sha256"), digests)?;
    let mut compile_digests = String::new();
    for preset in [
        "oracle-asan-ubsan",
        "oracle-debug",
        "oracle-release",
        "upstream-tests",
    ] {
        let mut records = ["collision_probe", "math_probe", "protocol_bits", "rigid_world"]
            .map(|name| {
                let prefix = if matches!(preset, "oracle-debug" | "oracle-release") { "<repo>" } else { "/fixture/repo" };
                let file = format!("{prefix}/tools/reference/src/{name}.cpp");
                json!({"file":file,"directory":"/fixture/build","command":format!("/usr/bin/clang++-22 -fno-fast-math -ffp-contract=off -c {file} -o <build>/{name}.o")})
            }).to_vec();
        if preset == "upstream-tests" {
            for name in [
                "BlockAllocator",
                "BodyContacts",
                "Callback",
                "Color",
                "Common",
                "Confinement",
                "Conservation",
                "FreeList",
                "Function",
                "HelloWorld",
                "IntrusiveList",
                "SlabAllocator",
                "TrackedBlock",
            ] {
                let file = format!(
                    "/fixture/repo/third_party/liquidfun/liquidfun/Box2D/Unittests/{name}/{name}Tests.cpp"
                );
                records.push(json!({"file":file,"directory":"/fixture/build","command":format!("/usr/bin/clang++-22 -c {file} -o {name}.o")}));
            }
            let file = "/fixture/repo/third_party/liquidfun/googletest/src/gtest-all.cc";
            records.push(json!({"file":file,"directory":"/fixture/build","command":format!("/usr/bin/clang++-22 -c {file} -o gtest.o")}));
        }
        let relative = format!("compile-commands-{preset}.json");
        let compile_file = directory.join(&relative);
        write_json(&compile_file, &json!(records))?;
        writeln!(compile_digests, "{}  {relative}", hash_file(&compile_file)?)?;
    }
    fs::write(directory.join("compile-commands.sha256"), compile_digests)?;
    write_json(
        &directory.join("identity.json"),
        &json!({
            "schema_version":1,"candidate_sha":candidate,"candidate_tree":tree,
            "workflow_run_id":"7","workflow_job_id":"canonical-native",
            "runner":{"os":"ubuntu-24.04","architecture":"x86_64"},
            "tools":{"rust":"1.97.0","clang":"22.1.8","cmake":"4.3.3","ninja":"1.13.2"},
            "presets":["oracle-debug","oracle-release","oracle-asan-ubsan","upstream-tests"],
            "command_order":commands.iter().map(|entry| entry["name"].clone()).collect::<Vec<_>>(),
            "command_exits":commands.iter().map(|entry| json!({"name":entry["name"],"exit_code":0})).collect::<Vec<_>>(),
            "compile_command_digests":"compile-commands.sha256","log_digests":"logs.sha256",
            "evidence_tier":"D1"
        }),
    )
}

fn mutate_identity(fixture: &Fixture, mutation: impl FnOnce(&mut Value)) -> TestResult {
    let identity_file = fixture.retained.join("canonical/identity.json");
    let mut identity = serde_json::from_slice(&fs::read(&identity_file)?)?;
    mutation(&mut identity);
    write_json(&identity_file, &identity)?;
    let digest = hash_file(&identity_file)?;
    fixture.mutate_evidence(|evidence| evidence["canonical_identity"]["sha256"] = json!(digest))
}

#[test]
fn rejects_incomplete_canonical_command_sequence() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    mutate_identity(&fixture, |identity| {
        identity["command_exits"]
            .as_array_mut()
            .expect("array")
            .remove(2);
        identity["command_order"]
            .as_array_mut()
            .expect("array")
            .remove(2);
    })?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_reordered_canonical_commands() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    mutate_identity(&fixture, |identity| {
        identity["command_exits"]
            .as_array_mut()
            .expect("array")
            .swap(0, 1);
        identity["command_order"]
            .as_array_mut()
            .expect("array")
            .swap(0, 1);
    })?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_missing_compile_digest_identity() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    mutate_identity(&fixture, |identity| {
        identity
            .as_object_mut()
            .expect("object")
            .remove("compile_command_digests");
    })?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_missing_canonical_presets() -> TestResult {
    let fixture = Fixture::new()?;
    mutate_identity(&fixture, |identity| {
        identity["presets"] = json!(["oracle-debug"]);
    })?;
    fixture.assert_rejected()
}

#[test]
fn rejects_canonical_argv_drift() -> TestResult {
    let fixture = Fixture::new()?;
    let tsv = fixture.retained.join("canonical/command-exits.tsv");
    fs::write(
        &tsv,
        fs::read_to_string(&tsv)?.replace("--runs 2", "--runs 1"),
    )?;
    fixture.assert_rejected()
}

#[test]
fn rejects_missing_retained_canonical_records() -> TestResult {
    for relative in [
        "logs.sha256",
        "compile-commands.sha256",
        "commands.json",
        "command-exits.tsv",
        "compile-commands-oracle-debug.json",
        "compile-commands-oracle-asan-ubsan.json",
        "compile-commands-upstream-tests.json",
        "logs/build-debug.log",
        "logs/build-asan.log",
        "logs/ctest-upstream-tests.log",
    ] {
        // Arrange
        let fixture = Fixture::new()?;
        fs::remove_file(fixture.retained.join("canonical").join(relative))?;
        // Act / Assert
        fixture.assert_rejected()?;
    }
    Ok(())
}

#[test]
fn rejects_empty_canonical_digest_inventories() -> TestResult {
    for relative in ["logs.sha256", "compile-commands.sha256"] {
        let fixture = Fixture::new()?;
        fs::write(fixture.retained.join("canonical").join(relative), "")?;
        fixture.assert_rejected()?;
    }
    Ok(())
}

#[test]
fn rejects_tampered_canonical_contents() -> TestResult {
    for relative in [
        "logs/build-debug.log",
        "compile-commands-oracle-debug.json",
        "compile-commands-oracle-asan-ubsan.json",
        "compile-commands-upstream-tests.json",
        "commands.json",
    ] {
        let fixture = Fixture::new()?;
        fs::write(
            fixture.retained.join("canonical").join(relative),
            "tampered\n",
        )?;
        fixture.assert_rejected()?;
    }
    Ok(())
}

#[test]
fn rejects_incomplete_compile_records_even_with_refreshed_digest() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    let directory = fixture.retained.join("canonical");
    let compile_file = directory.join("compile-commands-oracle-debug.json");
    let old_digest = hash_file(&compile_file)?;
    fs::write(&compile_file, "[]\n")?;
    let digest_file = directory.join("compile-commands.sha256");
    fs::write(
        &digest_file,
        fs::read_to_string(&digest_file)?.replace(&old_digest, &hash_file(&compile_file)?),
    )?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_missing_upstream_test_source_even_with_refreshed_digest() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    let directory = fixture.retained.join("canonical");
    let compile_file = directory.join("compile-commands-upstream-tests.json");
    let old_digest = hash_file(&compile_file)?;
    let mut records: Vec<Value> = serde_json::from_slice(&fs::read(&compile_file)?)?;
    records.retain(|record| {
        !record["file"]
            .as_str()
            .expect("fixture path")
            .ends_with("/TrackedBlockTests.cpp")
    });
    write_json(&compile_file, &json!(records))?;
    let digest_file = directory.join("compile-commands.sha256");
    fs::write(
        &digest_file,
        fs::read_to_string(&digest_file)?.replace(&old_digest, &hash_file(&compile_file)?),
    )?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_malformed_raw_compile_records_even_with_refreshed_digest() -> TestResult {
    for preset in ["oracle-asan-ubsan", "upstream-tests"] {
        // Arrange
        let fixture = Fixture::new()?;
        let directory = fixture.retained.join("canonical");
        let compile_file = directory.join(format!("compile-commands-{preset}.json"));
        let old_digest = hash_file(&compile_file)?;
        let mut records: Vec<Value> = serde_json::from_slice(&fs::read(&compile_file)?)?;
        records[0]["directory"] = json!(null);
        write_json(&compile_file, &json!(records))?;
        let digest_file = directory.join("compile-commands.sha256");
        fs::write(
            &digest_file,
            fs::read_to_string(&digest_file)?.replace(&old_digest, &hash_file(&compile_file)?),
        )?;
        // Act / Assert
        fixture.assert_rejected()?;
    }
    Ok(())
}

#[test]
fn rejects_raw_witness_path_substitution_even_with_refreshed_digest() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    let directory = fixture.retained.join("canonical");
    let compile_file = directory.join("compile-commands-oracle-asan-ubsan.json");
    let old_digest = hash_file(&compile_file)?;
    let modified =
        fs::read_to_string(&compile_file)?.replace("/tools/reference/src/", "/unrelated/");
    fs::write(&compile_file, modified)?;
    let digest_file = directory.join("compile-commands.sha256");
    fs::write(
        &digest_file,
        fs::read_to_string(&digest_file)?.replace(&old_digest, &hash_file(&compile_file)?),
    )?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_duplicate_canonical_digest_records() -> TestResult {
    let fixture = Fixture::new()?;
    let digest_file = fixture.retained.join("canonical/logs.sha256");
    let records = fs::read_to_string(&digest_file)?;
    fs::write(
        &digest_file,
        format!("{records}{}\n", records.lines().next().ok_or("first line")?),
    )?;
    fixture.assert_rejected()
}

#[test]
fn rejects_unterminated_malformed_digest_suffix() -> TestResult {
    // Arrange
    let fixture = Fixture::new()?;
    let digest_file = fixture.retained.join("canonical/logs.sha256");
    fs::write(
        &digest_file,
        format!("{}garbage", fs::read_to_string(&digest_file)?),
    )?;
    // Act / Assert
    fixture.assert_rejected()
}

#[test]
fn rejects_omitted_canonical_log_digest() -> TestResult {
    let fixture = Fixture::new()?;
    let digest_file = fixture.retained.join("canonical/logs.sha256");
    let records = fs::read_to_string(&digest_file)?;
    fs::write(
        &digest_file,
        records.split_once('\n').ok_or("first digest line")?.1,
    )?;
    fixture.assert_rejected()
}

#[test]
fn rejects_canonical_candidate_binding_drift() -> TestResult {
    let fixture = Fixture::new()?;
    mutate_identity(&fixture, |identity| {
        identity["candidate_sha"] = json!("0000000000000000000000000000000000000000");
    })?;
    fixture.assert_rejected()
}
