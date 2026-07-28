#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn parse_options(
    args: &[String],
) -> Result<BTreeMap<String, String>, DifferentialError> {
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let option = &pair[0];
        if !option.starts_with("--") {
            return Err(DifferentialError::usage(format!(
                "unexpected positional argument `{option}`"
            )));
        }
        if options.insert(option.clone(), pair[1].clone()).is_some() {
            return Err(DifferentialError::usage(format!(
                "duplicate differential option `{option}`"
            )));
        }
    }
    if !args.chunks_exact(2).remainder().is_empty() {
        return Err(DifferentialError::usage(
            "every differential option requires one value",
        ));
    }
    Ok(options)
}

pub(super) fn require_exact_options(
    options: &BTreeMap<String, String>,
    expected: &[&str],
) -> Result<(), DifferentialError> {
    if options.len() == expected.len()
        && options
            .keys()
            .all(|option| expected.contains(&option.as_str()))
    {
        return Ok(());
    }
    Err(DifferentialError::usage(
        "differential command options do not match the registered command shape",
    ))
}

pub(super) fn required_option<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, DifferentialError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| DifferentialError::usage(format!("missing required option `{name}`")))
}

pub(super) fn require_allowed<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
    allowed: &[&str],
) -> Result<&'a str, DifferentialError> {
    let value = required_option(options, name)?;
    if allowed.contains(&value) {
        return Ok(value);
    }
    Err(DifferentialError::usage(format!(
        "unregistered value `{value}` for `{name}`; allowed values: {}",
        allowed.join(", ")
    )))
}

pub(super) fn option_arguments(prefix: &[&str], options: &[(&str, &str)]) -> Vec<String> {
    prefix
        .iter()
        .copied()
        .chain(options.iter().flat_map(|(option, value)| [*option, *value]))
        .map(str::to_owned)
        .collect()
}
