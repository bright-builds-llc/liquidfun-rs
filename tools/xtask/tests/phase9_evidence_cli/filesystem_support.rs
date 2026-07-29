use super::*;

pub(super) fn write_evidence_directory(
    root: &Path,
    job: &str,
    manifest: &EvidenceManifest,
) -> TestResult {
    fs::create_dir_all(root)?;
    let source_payloads = root.parent().expect("fixture root").join("cases");
    copy_directory(&source_payloads, &root.join("cases"))?;
    let mut manifest_bytes = serde_json::to_vec_pretty(manifest)?;
    manifest_bytes.push(b'\n');
    fs::write(root.join("phase9-manifest.json"), manifest_bytes)?;
    fs::write(
        root.join("phase9-trace.log"),
        b"test result: ok. 25 passed; 0 failed; 1 ignored\n",
    )?;
    fs::write(root.join("provenance.log"), b"provenance verified\n")?;
    fs::write(root.join("inventory.log"), b"inventory verified\n")?;
    fs::write(root.join("read-only.log"), b"")?;
    write_identity(root, job)
}

pub(super) fn write_identity(root: &Path, job: &str) -> TestResult {
    write_identity_for(root, job, 0, "local")
}

pub(super) fn write_identity_for(
    root: &Path,
    job: &str,
    run_id: u64,
    head_sha: &str,
) -> TestResult {
    let files = regular_files(root)?
        .into_iter()
        .filter(|path| path != "identity.json")
        .map(|path| {
            Ok(json!({
                "path": path,
                "sha256": sha256(&fs::read(root.join(&path))?),
            }))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let identity = json!({
        "run_id": run_id,
        "job": job,
        "head_sha": head_sha,
        "upstream_revision": UPSTREAM_REVISION,
        "rust": "1.97.0",
        "cmake": "4.3.3",
        "ninja": "1.13.2",
        "clang": "22.1.8",
        "target": "x86_64-unknown-linux-gnu",
        "policy": "phase9-v1",
        "trace": {
            "path": "phase9-trace.log",
            "sha256": sha256(&fs::read(root.join("phase9-trace.log"))?),
        },
        "manifest": {
            "path": "phase9-manifest.json",
            "sha256": sha256(&fs::read(root.join("phase9-manifest.json"))?),
        },
        "files": files,
    });
    let mut bytes = serde_json::to_vec_pretty(&identity)?;
    bytes.push(b'\n');
    fs::write(root.join("identity.json"), bytes)?;
    Ok(())
}

pub(super) fn write_zip(source: &Path, archive: &Path) -> TestResult {
    let files = regular_files(source)?;
    let status = Command::new("zip")
        .arg("-q")
        .arg(archive)
        .args(files)
        .current_dir(source)
        .status()?;
    if !status.success() {
        return Err("zip failed while constructing exact-ref fixture".into());
    }
    Ok(())
}

pub(super) fn refresh_identity(root: &Path) -> TestResult {
    let identity: Value = serde_json::from_slice(&fs::read(root.join("identity.json"))?)?;
    let job = identity["job"].as_str().expect("identity job").to_owned();
    write_identity(root, &job)
}

pub(super) fn regular_files(root: &Path) -> TestResult<BTreeSet<String>> {
    let mut pending = vec![(root.to_path_buf(), PathBuf::new())];
    let mut files = BTreeSet::new();
    while let Some((directory, relative)) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let child_relative = relative.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                pending.push((entry.path(), child_relative));
            } else if entry.file_type()?.is_file() {
                files.insert(child_relative.to_string_lossy().into_owned());
            }
        }
    }
    Ok(files)
}

pub(super) fn copy_directory(source: &Path, destination: &Path) -> TestResult {
    for (relative, _) in regular_files(source)?
        .into_iter()
        .map(|relative| (relative.clone(), source.join(relative)))
    {
        let target = destination.join(&relative);
        fs::create_dir_all(target.parent().expect("payload parent"))?;
        fs::copy(source.join(relative), target)?;
    }
    Ok(())
}

pub(super) fn write_payload(root: &Path, relative: &str, bytes: &[u8]) -> TestResult {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("payload parent"))?;
    fs::write(path, bytes)?;
    Ok(())
}

pub(super) fn find_binding_mut<'a>(manifest: &'a mut Value, branch_id: &str) -> &'a mut Value {
    manifest["cases"]
        .as_array_mut()
        .expect("manifest cases")
        .iter_mut()
        .flat_map(|case| {
            case["witnesses"]
                .as_array_mut()
                .expect("case witnesses")
                .iter_mut()
        })
        .find(|binding| binding["branch_id"] == branch_id)
        .expect("reviewed branch binding")
}

pub(super) fn find_object_field<'a>(value: &'a Value, field: &str) -> &'a Value {
    find_object_field_maybe(value, field).expect("proof reference field")
}

pub(super) fn find_object_field_maybe<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    if let Some(found) = value.get(field) {
        return Some(found);
    }
    value
        .as_object()
        .into_iter()
        .flat_map(|object| object.values())
        .find_map(|child| find_object_field_maybe(child, field))
}

pub(super) fn find_object_field_mut<'a>(
    value: &'a mut Value,
    field: &str,
) -> Option<&'a mut Value> {
    if value.get(field).is_some() {
        return value.get_mut(field);
    }
    value
        .as_object_mut()
        .into_iter()
        .flat_map(|object| object.values_mut())
        .find_map(|child| find_object_field_mut(child, field))
}

pub(super) fn update_payload_reference_digests(value: &mut Value, path: &str, digest: &str) {
    match value {
        Value::Array(values) => {
            for value in values {
                update_payload_reference_digests(value, path, digest);
            }
        }
        Value::Object(object) => {
            if object.get("path").and_then(Value::as_str) == Some(path) {
                object.insert("sha256".to_owned(), json!(digest));
            }
            for value in object.values_mut() {
                update_payload_reference_digests(value, path, digest);
            }
        }
        _ => {}
    }
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn run_xtask(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(workspace_root())
        .output()
}

pub(super) fn assert_output_contains(output: &Output, needle: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "stderr did not contain `{needle}`:\n{stderr}"
    );
}

pub(super) fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(super) fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

pub(super) fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask belongs to the workspace")
}
