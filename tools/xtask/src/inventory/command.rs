#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn run(args: &[String]) -> Result<(), InventoryError> {
    let repository_root = repository_root()?;
    let oracle_revision = read_oracle_revision(&repository_root)?;

    match args {
        [namespace, command] if namespace == "corpus" && command == "refresh" => {
            let count = corpus_discovery::refresh(&repository_root, &oracle_revision)?;
            println!("semantic corpus refreshed: {count} items");
            Ok(())
        }
        [namespace, command] if namespace == "corpus" && command == "check-snapshot" => {
            let count = corpus_discovery::check_snapshot(&repository_root, &oracle_revision)?;
            println!("semantic corpus snapshot verified: {count} items");
            Ok(())
        }
        [namespace, command] if namespace == "corpus" && command == "check-closure" => {
            check_corpus_closure(&repository_root, &oracle_revision)
        }
        [namespace, command] if namespace == "corpus" && command == "generate-report" => {
            generate_corpus_report(&repository_root, &oracle_revision)
        }
        [command] => match command.as_str() {
            "discover" => discover(&repository_root, &oracle_revision),
            "generate" => generate(&repository_root, &oracle_revision),
            "check" => check(&repository_root, &oracle_revision),
            "check-report" => check_report(&repository_root, &oracle_revision),
            unknown => Err(InventoryError::usage(format!(
                "unknown inventory command `{unknown}`"
            ))),
        },
        _ => Err(InventoryError::usage(
            "expected a closed inventory command shape",
        )),
    }
}

fn check_corpus_closure(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    let manifest = corpus_validation::load_and_validate(repository_root, oracle_revision)?;
    let expected = corpus_report::render(&manifest);
    require_exact_file(
        &repository_root.join("UPSTREAM-CORPUS.md"),
        &expected,
        "semantic corpus report",
        "run `cargo xtask inventory corpus generate-report`",
    )
    .map_err(|error| InventoryError::new("corpus-report", error.message))?;
    println!(
        "semantic corpus closure verified: {} items, 0 unresolved",
        manifest.items().len()
    );
    Ok(())
}

fn generate_corpus_report(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    let manifest = corpus_validation::load_and_validate(repository_root, oracle_revision)?;
    let contents = corpus_report::render(&manifest);
    let path = repository_root.join("UPSTREAM-CORPUS.md");
    fs::write(&path, contents).map_err(|error| {
        InventoryError::new(
            "corpus-filesystem",
            format!("failed to write {}: {error}", path.display()),
        )
    })?;
    println!(
        "semantic corpus report generated: {} items, 0 unresolved",
        manifest.items().len()
    );
    Ok(())
}

fn discover(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let snapshot = discovery::scan(repository_root, oracle_revision)?;
    let contents = format_discovery(&snapshot)?;
    let path = repository_root.join("reference/discovery.json");
    fs::write(&path, contents).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to write {}: {error}", path.display()),
        )
    })?;
    println!(
        "inventory discovery refreshed: {} entries",
        snapshot.entries.len()
    );
    Ok(())
}

fn generate(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let (compatibility, _, readiness) = validated_ledgers(repository_root, oracle_revision)?;
    require_current_discovery(repository_root, oracle_revision)?;
    let contents = report::render(&compatibility, &readiness);
    let path = repository_root.join("COMPATIBILITY.md");
    fs::write(&path, contents).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to write {}: {error}", path.display()),
        )
    })?;
    println!(
        "compatibility report generated: {} entries",
        compatibility.entries.len()
    );
    Ok(())
}

fn check(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let (compatibility, _, readiness) = validated_ledgers(repository_root, oracle_revision)?;
    require_current_discovery(repository_root, oracle_revision)?;
    require_current_report(repository_root, &compatibility, &readiness)?;
    println!(
        "inventory verified: {} compatibility rows",
        compatibility.entries.len()
    );
    Ok(())
}

fn check_report(repository_root: &Path, oracle_revision: &str) -> Result<(), InventoryError> {
    let (compatibility, _, readiness) = validated_ledgers(repository_root, oracle_revision)?;
    require_current_report(repository_root, &compatibility, &readiness)?;
    println!(
        "compatibility report verified: {} rows",
        compatibility.entries.len()
    );
    Ok(())
}

fn require_current_report(
    repository_root: &Path,
    compatibility: &CompatibilityLedger,
    readiness: &ReleaseReadiness,
) -> Result<(), InventoryError> {
    let expected_report = report::render(compatibility, readiness);
    require_exact_file(
        &repository_root.join("COMPATIBILITY.md"),
        &expected_report,
        "report",
        "run `cargo xtask inventory generate`",
    )
}

fn validated_ledgers(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(CompatibilityLedger, DiscoveryLedger, ReleaseReadiness), InventoryError> {
    let compatibility: CompatibilityLedger = read_json(
        &repository_root.join("reference/compatibility.json"),
        "compatibility schema",
    )?;
    let discovery: DiscoveryLedger = read_json(
        &repository_root.join("reference/discovery.json"),
        "discovery schema",
    )?;
    validation::compatibility(&compatibility, oracle_revision, repository_root)?;
    validation::discovery(&discovery, oracle_revision)?;
    validation::coverage(&compatibility, &discovery)?;
    let corpus_path = repository_root.join("reference/upstream-corpus.json");
    let corpus_bytes = fs::read(&corpus_path).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to read {}: {error}", corpus_path.display()),
        )
    })?;
    let corpus = corpus::parse_manifest(&corpus_bytes, oracle_revision).map_err(|error| {
        InventoryError::new(
            error.inventory_category(),
            format!("invalid terminal corpus authority: {error}"),
        )
    })?;
    let readiness = validation::release_readiness(&compatibility, &corpus, repository_root)?;
    Ok((compatibility, discovery, readiness))
}

fn require_current_discovery(
    repository_root: &Path,
    oracle_revision: &str,
) -> Result<(), InventoryError> {
    let expected = format_discovery(&discovery::scan(repository_root, oracle_revision)?)?;
    require_exact_file(
        &repository_root.join("reference/discovery.json"),
        &expected,
        "discovery",
        "run `cargo xtask inventory discover` and review the compatibility ledger",
    )
}

fn require_exact_file(
    path: &Path,
    expected: &str,
    label: &str,
    remedy: &str,
) -> Result<(), InventoryError> {
    let actual = fs::read_to_string(path).map_err(|error| {
        InventoryError::new(
            "stale",
            format!("failed to read {}: {error}; {remedy}", path.display()),
        )
    })?;
    if actual == expected {
        return Ok(());
    }
    Err(InventoryError::new(
        "stale",
        format!("{label} does not match deterministic generated bytes; {remedy}"),
    ))
}

fn format_discovery(ledger: &DiscoveryLedger) -> Result<String, InventoryError> {
    let mut output = String::new();
    output.push_str("{\n");
    append_formatted(
        &mut output,
        format_args!("  \"schema_version\": {},\n", ledger.schema_version),
    );
    append_formatted(
        &mut output,
        format_args!(
            "  \"oracle_revision\": {},\n",
            json_string(&ledger.oracle_revision)?
        ),
    );
    append_formatted(
        &mut output,
        format_args!(
            "  \"sort_contract\": {},\n",
            json_string(&ledger.sort_contract)?
        ),
    );
    output.push_str("  \"scopes\": [\n");
    append_json_rows(&mut output, &ledger.scopes)?;
    output.push_str("  ],\n  \"entries\": [\n");
    append_json_rows(&mut output, &ledger.entries)?;
    output.push_str("  ]\n}\n");
    Ok(output)
}

fn append_formatted(output: &mut String, arguments: Arguments<'_>) {
    output
        .write_fmt(arguments)
        .expect("writing formatted discovery data to a String cannot fail");
}

fn append_json_rows<T: Serialize>(output: &mut String, rows: &[T]) -> Result<(), InventoryError> {
    for (index, row) in rows.iter().enumerate() {
        let serialized = serde_json::to_string(row).map_err(|error| {
            InventoryError::new("schema", format!("failed to serialize discovery: {error}"))
        })?;
        output.push_str("    ");
        output.push_str(&serialized);
        if index + 1 != rows.len() {
            output.push(',');
        }
        output.push('\n');
    }
    Ok(())
}

fn json_string(value: &str) -> Result<String, InventoryError> {
    serde_json::to_string(value).map_err(|error| {
        InventoryError::new(
            "schema",
            format!("failed to serialize JSON string: {error}"),
        )
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, InventoryError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        InventoryError::new(
            "schema",
            format!("invalid {label} in {}: {error}", path.display()),
        )
    })
}

fn read_oracle_revision(repository_root: &Path) -> Result<String, InventoryError> {
    let path = repository_root.join("reference/upstream-lock.toml");
    let contents = fs::read_to_string(&path).map_err(|error| {
        InventoryError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let value: toml::Value = toml::from_str(&contents).map_err(|error| {
        InventoryError::new("schema", format!("invalid {}: {error}", path.display()))
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(toml::Value::as_integer);
    if schema_version != Some(1) {
        return Err(InventoryError::new(
            "schema",
            "upstream lock schema_version must be 1",
        ));
    }
    let Some(revision) = value.get("revision").and_then(toml::Value::as_str) else {
        return Err(InventoryError::new(
            "schema",
            "upstream lock is missing revision",
        ));
    };
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(InventoryError::new(
            "revision",
            "upstream revision must be lowercase 40-hex",
        ));
    }
    Ok(revision.to_owned())
}
