//! Pure catalog parsing, ranking and Markdown rendering for the scene survey.

use std::fmt::Write as _;

use crate::playground::PlaygroundError;

const SCENE_IDS_MARKER: &str = "export const SCENE_IDS = [";
const TABLE_HEADER: &str = "| Rank | Scene | Median ms/step | Min ms/step | Max ms/step | Start particles | End particles | Interaction |\n";
const TABLE_DIVIDER: &str = "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |\n";

/// Parses the ordered `SCENE_IDS` array out of `web/src/catalog/scenes.ts`.
///
/// Splits the array body on commas, so one-id-per-line and single-line
/// Prettier output both parse.
pub(super) fn catalog_scene_ids(source: &str) -> Result<Vec<&str>, PlaygroundError> {
    let Some(start) = source.find(SCENE_IDS_MARKER) else {
        return Err(PlaygroundError::new(
            "spot",
            "web/src/catalog/scenes.ts is missing `export const SCENE_IDS = [`",
        ));
    };
    let body = &source[start + SCENE_IDS_MARKER.len()..];
    let Some(close) = body.find(']') else {
        return Err(PlaygroundError::new(
            "spot",
            "web/src/catalog/scenes.ts SCENE_IDS array is not closed",
        ));
    };
    let ids: Vec<&str> = body[..close]
        .split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .collect();
    if ids.is_empty() {
        return Err(PlaygroundError::new(
            "spot",
            "web/src/catalog/scenes.ts SCENE_IDS array is empty",
        ));
    }
    Ok(ids)
}

/// One survey table row.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct RankedScene {
    pub(super) scene: String,
    pub(super) median_ms: f64,
    pub(super) min_ms: f64,
    pub(super) max_ms: f64,
    pub(super) start_particles: u64,
    pub(super) end_particles: u64,
    pub(super) interaction: String,
}

/// Reads survey rows and orders them slowest median first, then by scene id.
pub(super) fn rank_scene_samples(
    scenes: &[serde_json::Value],
) -> Result<Vec<RankedScene>, PlaygroundError> {
    let mut rows = scenes
        .iter()
        .map(ranked_scene)
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by(|left, right| {
        right
            .median_ms
            .total_cmp(&left.median_ms)
            .then_with(|| left.scene.cmp(&right.scene))
    });
    Ok(rows)
}

fn ranked_scene(sample: &serde_json::Value) -> Result<RankedScene, PlaygroundError> {
    Ok(RankedScene {
        scene: str_field(sample, "scene")?.to_owned(),
        median_ms: f64_field(sample, "median_ms_per_step")?,
        min_ms: f64_field(sample, "min_ms_per_step")?,
        max_ms: f64_field(sample, "max_ms_per_step")?,
        start_particles: u64_field(sample, "start_particles")?,
        end_particles: u64_field(sample, "end_particles")?,
        interaction: str_field(sample, "interaction")?.to_owned(),
    })
}

fn str_field<'a>(sample: &'a serde_json::Value, field: &str) -> Result<&'a str, PlaygroundError> {
    sample
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| missing_field(field, "a string"))
}

fn f64_field(sample: &serde_json::Value, field: &str) -> Result<f64, PlaygroundError> {
    sample
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| missing_field(field, "a number"))
}

fn u64_field(sample: &serde_json::Value, field: &str) -> Result<u64, PlaygroundError> {
    sample
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| missing_field(field, "a non-negative integer"))
}

fn missing_field(field: &str, expected: &str) -> PlaygroundError {
    PlaygroundError::new(
        "spot",
        format!("survey scene field `{field}` must be {expected}"),
    )
}

/// Renders the ranked rows as a Markdown table, rank 1 first.
pub(super) fn render_ranked_table(rows: &[RankedScene]) -> String {
    let mut table = String::from(TABLE_HEADER);
    table.push_str(TABLE_DIVIDER);
    for (index, row) in rows.iter().enumerate() {
        writeln!(
            table,
            "| {rank} | {scene} | {median:.3} | {min:.3} | {max:.3} | {start} | {end} | {interaction} |",
            rank = index + 1,
            scene = row.scene,
            median = row.median_ms,
            min = row.min_ms,
            max = row.max_ms,
            start = row.start_particles,
            end = row.end_particles,
            interaction = row.interaction,
        )
        .expect("writing a survey table to a String cannot fail");
    }
    table
}

#[cfg(test)]
mod tests {
    use super::{RankedScene, catalog_scene_ids, rank_scene_samples, render_ranked_table};

    fn sample(scene: &str, median: f64) -> serde_json::Value {
        serde_json::json!({
            "scene": scene,
            "interaction": "default",
            "median_ms_per_step": median,
            "min_ms_per_step": median,
            "max_ms_per_step": median,
            "start_particles": 1,
            "end_particles": 2,
        })
    }

    #[test]
    fn catalog_scene_ids_parses_multiline_array() {
        // Arrange
        let source = "export const SCENE_IDS = [\n  \"a\",\n  \"b\",\n] as const;";

        // Act
        let ids = catalog_scene_ids(source);

        // Assert
        assert_eq!(ids, Ok(vec!["a", "b"]));
    }

    #[test]
    fn catalog_scene_ids_parses_single_line_array() {
        // Arrange
        let source = "export const SCENE_IDS = [\"a\", \"b\"] as const;";

        // Act
        let ids = catalog_scene_ids(source);

        // Assert
        assert_eq!(ids, Ok(vec!["a", "b"]));
    }

    #[test]
    fn catalog_scene_ids_rejects_missing_marker() {
        // Arrange
        let source = "export const OTHER_IDS = [\"a\"] as const;";

        // Act
        let ids = catalog_scene_ids(source);

        // Assert
        assert!(ids.is_err());
    }

    #[test]
    fn rank_scene_samples_orders_slowest_first_with_scene_tiebreak() {
        // Arrange
        let samples = [sample("b", 1.0), sample("a", 3.0), sample("a", 1.0)];

        // Act
        let ranked = rank_scene_samples(&samples).expect("samples should rank");

        // Assert
        let order: Vec<(&str, f64)> = ranked
            .iter()
            .map(|row| (row.scene.as_str(), row.median_ms))
            .collect();
        assert_eq!(order, [("a", 3.0), ("a", 1.0), ("b", 1.0)]);
    }

    #[test]
    fn render_ranked_table_formats_header_and_three_decimals() {
        // Arrange
        let rows = [RankedScene {
            scene: "liquid-tumbler".to_owned(),
            median_ms: 24.99,
            min_ms: 24.9,
            max_ms: 25.46,
            start_particles: 3800,
            end_particles: 3800,
            interaction: "default".to_owned(),
        }];

        // Act
        let table = render_ranked_table(&rows);

        // Assert
        assert!(table.starts_with(
            "| Rank | Scene | Median ms/step | Min ms/step | Max ms/step | Start particles | End particles | Interaction |\n"
        ));
        assert!(table.contains(
            "| 1 | liquid-tumbler | 24.990 | 24.900 | 25.460 | 3800 | 3800 | default |\n"
        ));
    }
}
