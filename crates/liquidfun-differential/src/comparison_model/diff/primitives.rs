//! Stable primitive traversal and exact renderer-neutral value comparison.

use liquidfun_test_protocol::{
    DebugPrimitiveKey, DebugPrimitiveRecord, MathProbePolicyPath, Phase4PolicyProfile, Vec2Bits,
    WireDebugPrimitive,
};

use super::super::{ComparisonError, ComparisonKind, ComparisonState};
use super::builder::EntryBuilder;

pub(super) fn compare_primitives(
    builder: &mut EntryBuilder<'_>,
    rust: &[DebugPrimitiveRecord],
    oracle: &[DebugPrimitiveRecord],
    policies: &Phase4PolicyProfile,
) -> Result<(), ComparisonError> {
    for index in 0..rust.len().max(oracle.len()) {
        let base = format!("debug_primitives.{index}");
        match (rust.get(index), oracle.get(index)) {
            (Some(rust_value), Some(oracle_value)) => {
                compare_primitive(builder, &base, rust_value, oracle_value, policies)?;
            }
            (Some(value), None) => builder.only_with_key(
                &format!("{base}.presence"),
                ComparisonKind::Presence,
                ComparisonState::RustOnly,
                Some(value),
                None::<&DebugPrimitiveRecord>,
                Some(value.key().clone()),
                "semantic primitive missing from oracle",
            )?,
            (None, Some(value)) => builder.only_with_key(
                &format!("{base}.presence"),
                ComparisonKind::Presence,
                ComparisonState::OracleOnly,
                None::<&DebugPrimitiveRecord>,
                Some(value),
                Some(value.key().clone()),
                "semantic primitive missing from Rust",
            )?,
            (None, None) => {}
        }
    }
    Ok(())
}

fn compare_primitive(
    builder: &mut EntryBuilder<'_>,
    base: &str,
    rust: &DebugPrimitiveRecord,
    oracle: &DebugPrimitiveRecord,
    policies: &Phase4PolicyProfile,
) -> Result<(), ComparisonError> {
    let maybe_key = (rust.key() == oracle.key()).then(|| rust.key().clone());
    builder.exact_with_key(
        &format!("{base}.ordering"),
        ComparisonKind::Order,
        &rust.ordering(),
        &oracle.ordering(),
        maybe_key.clone(),
        "source-significant or explicitly canonicalized primitive order",
    )?;
    compare_key(builder, base, rust.key(), oracle.key(), maybe_key.clone())?;
    let rust_primitive = rust.primitive();
    let oracle_primitive = oracle.primitive();
    builder.exact_with_key(
        &format!("{base}.kind"),
        ComparisonKind::Kind,
        &primitive_kind(rust_primitive),
        &primitive_kind(oracle_primitive),
        maybe_key.clone(),
        "closed renderer-neutral primitive kind",
    )?;
    compare_style(
        builder,
        base,
        rust_primitive,
        oracle_primitive,
        policies,
        maybe_key.clone(),
    )?;
    compare_geometry(
        builder,
        base,
        rust_primitive,
        oracle_primitive,
        policies,
        maybe_key,
    )
}

fn compare_key(
    builder: &mut EntryBuilder<'_>,
    base: &str,
    rust: &DebugPrimitiveKey,
    oracle: &DebugPrimitiveKey,
    maybe_key: Option<DebugPrimitiveKey>,
) -> Result<(), ComparisonError> {
    builder.exact_with_key(
        &format!("{base}.key.owner"),
        ComparisonKind::Identity,
        rust.owner(),
        oracle.owner(),
        maybe_key.clone(),
        "stable semantic primitive owner",
    )?;
    builder.exact_with_key(
        &format!("{base}.key.layer"),
        ComparisonKind::Kind,
        &rust.layer(),
        &oracle.layer(),
        maybe_key.clone(),
        "closed semantic debug layer",
    )?;
    builder.exact_with_key(
        &format!("{base}.key.kind"),
        ComparisonKind::Kind,
        &rust.kind(),
        &oracle.kind(),
        maybe_key.clone(),
        "closed stable-key primitive kind",
    )?;
    builder.exact_with_key(
        &format!("{base}.key.child"),
        ComparisonKind::Count,
        &rust.child(),
        &oracle.child(),
        maybe_key.clone(),
        "stable semantic child ordinal",
    )?;
    builder.exact_with_key(
        &format!("{base}.key.ordinal"),
        ComparisonKind::Order,
        &rust.ordinal(),
        &oracle.ordinal(),
        maybe_key,
        "stable semantic primitive ordinal",
    )
}

fn compare_style(
    builder: &mut EntryBuilder<'_>,
    base: &str,
    rust: &WireDebugPrimitive,
    oracle: &WireDebugPrimitive,
    policies: &Phase4PolicyProfile,
    maybe_key: Option<DebugPrimitiveKey>,
) -> Result<(), ComparisonError> {
    let rust_metadata = rust.metadata();
    let oracle_metadata = oracle.metadata();
    builder.exact_with_key(
        &format!("{base}.stroke.color"),
        ComparisonKind::Flags,
        &rust_metadata.stroke().color(),
        &oracle_metadata.stroke().color(),
        maybe_key.clone(),
        "exact renderer-neutral RGBA bits",
    )?;
    builder.numeric(
        &format!("{base}.stroke.width"),
        rust_metadata.stroke().width_bits(),
        oracle_metadata.stroke().width_bits(),
        MathProbePolicyPath::MathVectorLength,
        policies,
        maybe_key.clone(),
        "world-space stroke width",
    )?;
    builder.exact_with_key(
        &format!("{base}.fill.presence"),
        ComparisonKind::Presence,
        &rust_metadata.maybe_fill().is_some(),
        &oracle_metadata.maybe_fill().is_some(),
        maybe_key.clone(),
        "optional exact fill presence",
    )?;
    if let (Some(rust_fill), Some(oracle_fill)) =
        (rust_metadata.maybe_fill(), oracle_metadata.maybe_fill())
    {
        builder.exact_with_key(
            &format!("{base}.fill.color"),
            ComparisonKind::Flags,
            &rust_fill.color(),
            &oracle_fill.color(),
            maybe_key,
            "exact renderer-neutral fill RGBA bits",
        )?;
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive closed primitive match keeps every geometry field visibly bound"
)]
fn compare_geometry(
    builder: &mut EntryBuilder<'_>,
    base: &str,
    rust: &WireDebugPrimitive,
    oracle: &WireDebugPrimitive,
    policies: &Phase4PolicyProfile,
    maybe_key: Option<DebugPrimitiveKey>,
) -> Result<(), ComparisonError> {
    match (rust, oracle) {
        (WireDebugPrimitive::Point(rust), WireDebugPrimitive::Point(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.position"),
                rust.position(),
                oracle.position(),
                policies,
                maybe_key.clone(),
            )?;
            builder.numeric(
                &format!("{base}.radius"),
                rust.radius_bits(),
                oracle.radius_bits(),
                MathProbePolicyPath::MathVectorLength,
                policies,
                maybe_key,
                "point radius",
            )?;
        }
        (WireDebugPrimitive::Segment(rust), WireDebugPrimitive::Segment(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.start"),
                rust.start(),
                oracle.start(),
                policies,
                maybe_key.clone(),
            )?;
            compare_vec(
                builder,
                &format!("{base}.end"),
                rust.end(),
                oracle.end(),
                policies,
                maybe_key,
            )?;
        }
        (WireDebugPrimitive::Polyline(rust), WireDebugPrimitive::Polyline(oracle)) => {
            builder.exact_with_key(
                &format!("{base}.closed"),
                ComparisonKind::Presence,
                &rust.closed(),
                &oracle.closed(),
                maybe_key.clone(),
                "polyline closure state",
            )?;
            builder.exact_with_key(
                &format!("{base}.vertex_count"),
                ComparisonKind::Count,
                &rust.vertices().len(),
                &oracle.vertices().len(),
                maybe_key.clone(),
                "source-significant polyline vertex count",
            )?;
            for index in 0..rust.vertices().len().min(oracle.vertices().len()) {
                compare_vec(
                    builder,
                    &format!("{base}.vertices.{index}"),
                    rust.vertices()[index],
                    oracle.vertices()[index],
                    policies,
                    maybe_key.clone(),
                )?;
            }
            for index in rust.vertices().len().min(oracle.vertices().len())
                ..rust.vertices().len().max(oracle.vertices().len())
            {
                builder.only_with_key(
                    &format!("{base}.vertices.{index}.presence"),
                    ComparisonKind::Presence,
                    if rust.vertices().get(index).is_some() {
                        ComparisonState::RustOnly
                    } else {
                        ComparisonState::OracleOnly
                    },
                    rust.vertices().get(index),
                    oracle.vertices().get(index),
                    maybe_key.clone(),
                    "source-significant polyline vertex exists on only one backend",
                )?;
            }
        }
        (WireDebugPrimitive::Circle(rust), WireDebugPrimitive::Circle(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.center"),
                rust.center(),
                oracle.center(),
                policies,
                maybe_key.clone(),
            )?;
            builder.numeric(
                &format!("{base}.radius"),
                rust.radius_bits(),
                oracle.radius_bits(),
                MathProbePolicyPath::MathVectorLength,
                policies,
                maybe_key,
                "circle radius",
            )?;
        }
        (WireDebugPrimitive::TransformAxes(rust), WireDebugPrimitive::TransformAxes(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.position"),
                rust.transform().position,
                oracle.transform().position,
                policies,
                maybe_key.clone(),
            )?;
            builder.numeric(
                &format!("{base}.angle"),
                rust.transform().angle_bits,
                oracle.transform().angle_bits,
                MathProbePolicyPath::MathRotation,
                policies,
                maybe_key.clone(),
                "transform angle",
            )?;
            builder.numeric(
                &format!("{base}.scale"),
                rust.scale_bits(),
                oracle.scale_bits(),
                MathProbePolicyPath::MathVectorLength,
                policies,
                maybe_key,
                "transform-axis scale",
            )?;
        }
        (WireDebugPrimitive::Aabb(rust), WireDebugPrimitive::Aabb(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.lower"),
                rust.lower(),
                oracle.lower(),
                policies,
                maybe_key.clone(),
            )?;
            compare_vec(
                builder,
                &format!("{base}.upper"),
                rust.upper(),
                oracle.upper(),
                policies,
                maybe_key,
            )?;
        }
        (WireDebugPrimitive::Arrow(rust), WireDebugPrimitive::Arrow(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.start"),
                rust.start(),
                oracle.start(),
                policies,
                maybe_key.clone(),
            )?;
            compare_vec(
                builder,
                &format!("{base}.end"),
                rust.end(),
                oracle.end(),
                policies,
                maybe_key,
            )?;
        }
        (WireDebugPrimitive::Label(rust), WireDebugPrimitive::Label(oracle)) => {
            compare_vec(
                builder,
                &format!("{base}.position"),
                rust.position(),
                oracle.position(),
                policies,
                maybe_key.clone(),
            )?;
            builder.exact_with_key(
                &format!("{base}.text"),
                ComparisonKind::Text,
                &rust.text(),
                &oracle.text(),
                maybe_key,
                "bounded inert semantic label",
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn compare_vec(
    builder: &mut EntryBuilder<'_>,
    base: &str,
    rust: Vec2Bits,
    oracle: Vec2Bits,
    policies: &Phase4PolicyProfile,
    maybe_key: Option<DebugPrimitiveKey>,
) -> Result<(), ComparisonError> {
    builder.numeric(
        &format!("{base}.x"),
        rust.x_bits,
        oracle.x_bits,
        MathProbePolicyPath::MathVectorLength,
        policies,
        maybe_key.clone(),
        "renderer-neutral geometry x component",
    )?;
    builder.numeric(
        &format!("{base}.y"),
        rust.y_bits,
        oracle.y_bits,
        MathProbePolicyPath::MathVectorLength,
        policies,
        maybe_key,
        "renderer-neutral geometry y component",
    )
}

const fn primitive_kind(primitive: &WireDebugPrimitive) -> &'static str {
    match primitive {
        WireDebugPrimitive::Point(_) => "point",
        WireDebugPrimitive::Segment(_) => "segment",
        WireDebugPrimitive::Polyline(_) => "polyline",
        WireDebugPrimitive::Circle(_) => "circle",
        WireDebugPrimitive::TransformAxes(_) => "transform_axes",
        WireDebugPrimitive::Aabb(_) => "aabb",
        WireDebugPrimitive::Arrow(_) => "arrow",
        WireDebugPrimitive::Label(_) => "label",
    }
}
