fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should be present")
        .to_path_buf()
}

fn oracle_name() -> &'static str {
    if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    }
}

fn run_git(root: &Path, arguments: &[&str]) -> io::Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn git_head(root: &Path) -> io::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    Err(io::Error::other(format!(
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn write_adapter_inputs(root: &Path) -> io::Result<()> {
    let source = root.join("tools/reference/src");
    fs::create_dir_all(&source)?;
    fs::write(
        root.join("tools/reference/adapter-inputs.txt"),
        "tools/reference/src/fixture_adapter.cpp\ntools/reference/src/fixture_adapter.hpp\n",
    )?;
    fs::write(
        source.join("fixture_adapter.cpp"),
        b"fixture adapter implementation\n",
    )?;
    fs::write(
        source.join("fixture_adapter.hpp"),
        b"fixture adapter interface\n",
    )
}

fn write_compile_database(root: &Path, common_flag: &str) -> io::Result<()> {
    let build = root.join("target/reference/oracle-debug");
    fs::create_dir_all(&build)?;
    let units = [
        "collision_probe.cpp",
        "math_probe.cpp",
        "protocol_bits.cpp",
        "rigid_world.cpp",
    ];
    let entries = units
        .map(|unit| {
            let source = root.join("tools/reference/src").join(unit);
            serde_json::json!({
                "directory": build,
                "file": source,
                "command": format!(
                    "clang++ -I{}/tools/reference/src {common_flag} -o {}/{unit}.o -c {}",
                    root.display(),
                    build.display(),
                    source.display()
                ),
            })
        })
        .to_vec();
    fs::write(
        build.join("compile_commands.json"),
        serde_json::to_vec_pretty(&entries)?,
    )
}

fn snapshot_tree(root: &Path) -> io::Result<Vec<(PathBuf, Vec<u8>)>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    collect_snapshot(root, Path::new(""), &mut files)?;
    Ok(files)
}

fn collect_snapshot(
    root: &Path,
    relative: &Path,
    files: &mut Vec<(PathBuf, Vec<u8>)>,
) -> io::Result<()> {
    let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let child_relative = relative.join(entry.file_name());
        if path.is_dir() {
            collect_snapshot(&path, &child_relative, files)?;
        } else {
            files.push((child_relative, fs::read(path)?));
        }
    }
    Ok(())
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
