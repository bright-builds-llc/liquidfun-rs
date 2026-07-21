//! Bounded tokenization and semantic parsing for allowlisted upstream sources.

use std::collections::BTreeSet;

use super::super::InventoryError;

const MAX_TOKENS: usize = 600_000;
const MAX_TOKEN_BYTES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Ident(String),
    String(String),
    Punct(char),
}

pub(super) fn google_test_symbols(contents: &str) -> Result<Vec<String>, InventoryError> {
    let tokens = tokenize(contents)?;
    let parameter_sets = parameter_sets(&tokens)?;
    let mut symbols = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let Token::Ident(macro_name) = &tokens[index] else {
            index += 1;
            continue;
        };
        if !matches!(macro_name.as_str(), "TEST" | "TEST_F" | "TEST_P") {
            index += 1;
            continue;
        }
        let (suite, case_name, next) = parse_two_identifier_macro(&tokens, index)?;
        if macro_name == "TEST_P" {
            let matching: Vec<_> = parameter_sets
                .iter()
                .filter(|set| set.suite == suite)
                .collect();
            if matching.len() != 1 {
                return Err(source_error(
                    "parameterized test has ambiguous registration",
                ));
            }
            for parameter in &matching[0].parameters {
                symbols.push(format!(
                    "{}/{suite}.{case_name}/{parameter}",
                    matching[0].prefix
                ));
            }
        } else {
            symbols.push(format!("{suite}.{case_name}"));
        }
        index = next;
    }
    Ok(symbols)
}

struct ParameterSet {
    prefix: String,
    suite: String,
    parameters: Vec<String>,
}

fn parameter_sets(tokens: &[Token]) -> Result<Vec<ParameterSet>, InventoryError> {
    let mut sets = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let Token::Ident(name) = token else {
            continue;
        };
        if !matches!(
            name.as_str(),
            "INSTANTIATE_TEST_CASE_P" | "INSTANTIATE_TEST_SUITE_P"
        ) {
            continue;
        }
        let prefix = identifier_at(tokens, index + 2)?;
        require_punct(tokens, index + 3, ',')?;
        let suite = identifier_at(tokens, index + 4)?;
        require_punct(tokens, index + 5, ',')?;
        if identifier_at(tokens, index + 6)? != "testing"
            || !matches!(tokens.get(index + 7), Some(Token::Punct(':')))
            || !matches!(tokens.get(index + 8), Some(Token::Punct(':')))
            || identifier_at(tokens, index + 9)? != "ValuesIn"
        {
            return Err(source_error("unsupported parameterized test registration"));
        }
        require_punct(tokens, index + 10, '(')?;
        let array = identifier_at(tokens, index + 11)?;
        require_punct(tokens, index + 12, ')')?;
        let parameters = string_array_values(tokens, &array)?;
        if parameters.is_empty() {
            return Err(source_error("parameterized test registration is empty"));
        }
        sets.push(ParameterSet {
            prefix,
            suite,
            parameters,
        });
    }
    Ok(sets)
}

fn string_array_values(tokens: &[Token], array: &str) -> Result<Vec<String>, InventoryError> {
    for index in 0..tokens.len() {
        if !matches!(&tokens[index], Token::Ident(name) if name == array) {
            continue;
        }
        let Some(open_offset) = tokens[index..]
            .iter()
            .position(|token| matches!(token, Token::Punct('{')))
        else {
            continue;
        };
        let open = index + open_offset;
        let mut depth = 0_usize;
        let mut values = Vec::new();
        for token in &tokens[open..] {
            match token {
                Token::Punct('{') => depth += 1,
                Token::Punct('}') => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Ok(values);
                    }
                }
                Token::String(value) if depth == 2 => values.push(value.clone()),
                _ => {}
            }
        }
    }
    Err(source_error("parameter array is missing or malformed"))
}

pub(super) fn testbed_registrations(
    contents: &str,
) -> Result<Vec<(String, String)>, InventoryError> {
    let tokens = tokenize(contents)?;
    let mut registrations = Vec::new();
    let mut index = 0;
    while index + 7 < tokens.len() {
        let Some(Token::Punct('{')) = tokens.get(index) else {
            index += 1;
            continue;
        };
        let Some(Token::String(title)) = tokens.get(index + 1) else {
            index += 1;
            continue;
        };
        require_punct(&tokens, index + 2, ',')?;
        let factory = identifier_at(&tokens, index + 3)?;
        require_punct(&tokens, index + 4, ':')?;
        require_punct(&tokens, index + 5, ':')?;
        if identifier_at(&tokens, index + 6)? != "Create" {
            return Err(source_error("testbed registration factory is malformed"));
        }
        require_punct(&tokens, index + 7, '}')?;
        if title.is_empty() || title.len() > MAX_TOKEN_BYTES {
            return Err(source_error("testbed registration title is invalid"));
        }
        registrations.push((title.clone(), factory));
        index += 8;
    }
    if registrations.is_empty() {
        return Err(source_error("testbed registration table is empty"));
    }
    let mut titles = BTreeSet::new();
    let mut factories = BTreeSet::new();
    for (title, factory) in &registrations {
        if !titles.insert(title) || !factories.insert(factory) {
            return Err(source_error("duplicate testbed registration"));
        }
    }
    Ok(registrations)
}

pub(super) fn testbed_scenario_class(contents: &str) -> Result<String, InventoryError> {
    let tokens = tokenize(contents)?;
    let mut classes = Vec::new();
    for index in 0..tokens.len().saturating_sub(5) {
        if !matches!(&tokens[index], Token::Ident(name) if name == "class") {
            continue;
        }
        let class = identifier_at(&tokens, index + 1)?;
        if !matches!(tokens.get(index + 2), Some(Token::Punct(':')))
            || identifier_at(&tokens, index + 3)? != "public"
        {
            continue;
        }
        let _base = identifier_at(&tokens, index + 4)?;
        let Some(open_offset) = tokens[index + 5..]
            .iter()
            .position(|token| matches!(token, Token::Punct('{')))
        else {
            return Err(source_error("testbed scenario class body is malformed"));
        };
        let open = index + 5 + open_offset;
        let close = matching_delimiter(&tokens, open, '{', '}')?;
        let has_create = tokens[open + 1..close].windows(2).any(|window| {
            matches!(&window[0], Token::Ident(name) if name == "Create")
                && matches!(window[1], Token::Punct('('))
        });
        if has_create {
            classes.push(class);
        }
    }
    if classes.len() != 1 {
        return Err(source_error("testbed scenario source is ambiguous"));
    }
    Ok(classes.remove(0))
}

fn matching_delimiter(
    tokens: &[Token],
    open: usize,
    opening: char,
    closing: char,
) -> Result<usize, InventoryError> {
    let mut depth = 0_usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Token::Punct(value) if *value == opening => depth += 1,
            Token::Punct(value) if *value == closing => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    Err(source_error("source delimiter is unbalanced"))
}

pub(super) fn contains_main(contents: &str) -> Result<bool, InventoryError> {
    let tokens = tokenize(contents)?;
    Ok(tokens.windows(2).any(|window| {
        matches!(&window[0], Token::Ident(name) if name == "main")
            && matches!(window[1], Token::Punct('('))
    }))
}

fn parse_two_identifier_macro(
    tokens: &[Token],
    index: usize,
) -> Result<(String, String, usize), InventoryError> {
    require_punct(tokens, index + 1, '(')?;
    let first = identifier_at(tokens, index + 2)?;
    require_punct(tokens, index + 3, ',')?;
    let second = identifier_at(tokens, index + 4)?;
    require_punct(tokens, index + 5, ')')?;
    Ok((first, second, index + 6))
}

fn identifier_at(tokens: &[Token], index: usize) -> Result<String, InventoryError> {
    match tokens.get(index) {
        Some(Token::Ident(value)) if value.len() <= MAX_TOKEN_BYTES => Ok(value.clone()),
        _ => Err(source_error("semantic source macro is malformed")),
    }
}

fn require_punct(tokens: &[Token], index: usize, expected: char) -> Result<(), InventoryError> {
    if matches!(tokens.get(index), Some(Token::Punct(actual)) if *actual == expected) {
        return Ok(());
    }
    Err(source_error("semantic source macro is malformed"))
}

fn tokenize(contents: &str) -> Result<Vec<Token>, InventoryError> {
    let bytes = contents.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            let Some(end) = bytes[index + 2..]
                .windows(2)
                .position(|window| window == b"*/")
            else {
                return Err(source_error("unterminated source comment"));
            };
            index += end + 4;
            continue;
        }
        if bytes[index] == b'"' {
            let (value, next) = string_token(bytes, index)?;
            tokens.push(Token::String(value));
            index = next;
        } else if bytes[index] == b'\'' {
            index = skip_quoted(bytes, index, b'\'')?;
        } else if bytes[index].is_ascii_alphabetic() || bytes[index] == b'_' {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            if index - start > MAX_TOKEN_BYTES {
                return Err(source_error("source token length limit exceeded"));
            }
            tokens.push(Token::Ident(contents[start..index].to_owned()));
        } else {
            tokens.push(Token::Punct(char::from(bytes[index])));
            index += 1;
        }
        if tokens.len() > MAX_TOKENS {
            return Err(source_error("source token limit exceeded"));
        }
    }
    Ok(tokens)
}

fn string_token(bytes: &[u8], start: usize) -> Result<(String, usize), InventoryError> {
    let mut output = String::new();
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Ok((output, index + 1)),
            b'\\' => {
                index += 1;
                let Some(escaped) = bytes.get(index) else {
                    return Err(source_error("unterminated source string"));
                };
                output.push(char::from(*escaped));
                index += 1;
            }
            byte if byte.is_ascii() => {
                output.push(char::from(byte));
                index += 1;
            }
            _ => return Err(source_error("source string is not ASCII")),
        }
        if output.len() > MAX_TOKEN_BYTES {
            return Err(source_error("source string length limit exceeded"));
        }
    }
    Err(source_error("unterminated source string"))
}

fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> Result<usize, InventoryError> {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index = index.saturating_add(2);
            continue;
        }
        if bytes[index] == quote {
            return Ok(index + 1);
        }
        index += 1;
    }
    Err(source_error("unterminated source character literal"))
}

fn source_error(message: &'static str) -> InventoryError {
    InventoryError::new("corpus-source", message)
}
