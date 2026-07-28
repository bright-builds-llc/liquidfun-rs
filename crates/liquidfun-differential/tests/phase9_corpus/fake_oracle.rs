struct FakePhase9OracleRoot {
    root: PathBuf,
}

impl FakePhase9OracleRoot {
    fn new(behavior: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "liquidfun-phase9-retained-oracle-{}-{nonce}",
            std::process::id()
        ));
        let preset = root.join("target/reference/oracle-debug");
        fs::create_dir_all(&preset).expect("fake preset directory should be created");
        let destination = preset.join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
        fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), &destination)
            .expect("fake oracle should copy into the reviewed path");
        copy_adapter_inputs(&root);
        write_fake_compile_database(&root);
        fs::write(preset.join("behavior.txt"), behavior)
            .expect("fake oracle behavior should be written");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

fn write_fake_compile_database(root: &Path) {
    let build = root.join("target/reference/oracle-debug");
    let entries = FAKE_PHASE9_RESULT_UNITS
        .map(|unit| {
            let source = root.join("tools/reference/src").join(unit);
            json!({
                "directory": build,
                "file": source,
                "command": format!(
                    "clang++ -I{}/tools/reference/src -O0 -g -o {}/{unit}.o -c {}",
                    root.display(),
                    build.display(),
                    source.display()
                ),
            })
        })
        .to_vec();
    fs::write(
        build.join("compile_commands.json"),
        serde_json::to_vec_pretty(&entries)
            .expect("fake compile database should encode deterministically"),
    )
    .expect("fake compile database should be written");
}

fn copy_adapter_inputs(destination_root: &Path) {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest_path = Path::new("tools/reference/adapter-inputs.txt");
    let manifest = fs::read_to_string(source_root.join(manifest_path))
        .expect("reviewed adapter input manifest should be readable");
    let destination_manifest = destination_root.join(manifest_path);
    fs::create_dir_all(
        destination_manifest
            .parent()
            .expect("adapter manifest should have a parent"),
    )
    .expect("adapter manifest directory should be created");
    fs::write(&destination_manifest, &manifest).expect("adapter input manifest should be copied");
    for relative in manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let destination = destination_root.join(relative);
        fs::create_dir_all(
            destination
                .parent()
                .expect("adapter input should have a parent"),
        )
        .expect("adapter input directory should be created");
        fs::copy(source_root.join(relative), destination)
            .expect("reviewed adapter input should be copied");
    }
}

impl Drop for FakePhase9OracleRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("fake oracle root should be removable");
    }
}
