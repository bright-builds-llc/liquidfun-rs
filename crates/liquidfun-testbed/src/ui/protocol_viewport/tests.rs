use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointId, CheckpointPosition, DebugPrimitiveOrder,
    DebugPrimitiveRecord, FloatBits, RequestId, Sha256Hex,
};
use serde_json::{Value, json};

use super::{
    ORACLE_COMPARISON_COLOR, ProtocolComparisonBackend, ProtocolDisplayPrimitive,
    ProtocolLayerVisibility, ProtocolScreenPoint, ProtocolScreenStyle, ProtocolViewport,
    ProtocolViewportError, RUST_COMPARISON_COLOR, comparison_style, hit_test_frame,
    project_checkpoint,
};

fn bits(value: f32) -> u32 {
    value.to_bits()
}

fn metadata(kind: &str, layer: &str, ordinal: u32) -> Value {
    json!({
        "key": {
            "owner": { "kind": "world" },
            "layer": layer,
            "kind": kind,
            "child": 0,
            "ordinal": ordinal
        },
        "stroke": {
            "color": [1, 2, 3, 4],
            "width_bits": bits(0.5)
        },
        "maybe_fill": { "color": [5, 6, 7, 8] }
    })
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the JSON fixture builder consumes each temporary value exactly once"
)]
fn record(kind: &str, _layer: &str, _ordinal: u32, value: Value) -> DebugPrimitiveRecord {
    serde_json::from_value(json!({
        "ordering": "source_significant",
        "primitive": {
            "kind": kind,
            "value": value
        }
    }))
    .expect("test primitive fixture should satisfy the wire shape")
}

fn all_primitive_records() -> Vec<DebugPrimitiveRecord> {
    vec![
        record(
            "point",
            "shapes",
            0,
            json!({
                "metadata": metadata("point", "shapes", 0),
                "position": { "x_bits": bits(1.0), "y_bits": bits(2.0) },
                "radius_bits": bits(0.25)
            }),
        ),
        record(
            "segment",
            "joints",
            1,
            json!({
                "metadata": metadata("segment", "joints", 1),
                "start": { "x_bits": bits(-1.0), "y_bits": bits(0.0) },
                "end": { "x_bits": bits(1.0), "y_bits": bits(0.0) }
            }),
        ),
        record(
            "polyline",
            "contacts",
            2,
            json!({
                "metadata": metadata("polyline", "contacts", 2),
                "vertices": [
                    { "x_bits": bits(-1.0), "y_bits": bits(-1.0) },
                    { "x_bits": bits(1.0), "y_bits": bits(-1.0) },
                    { "x_bits": bits(0.0), "y_bits": bits(1.0) }
                ],
                "closed": true
            }),
        ),
        record(
            "circle",
            "contact_normals",
            3,
            json!({
                "metadata": metadata("circle", "contact_normals", 3),
                "center": { "x_bits": bits(0.0), "y_bits": bits(0.0) },
                "radius_bits": bits(2.0)
            }),
        ),
        record(
            "transform_axes",
            "particles",
            4,
            json!({
                "metadata": metadata("transform_axes", "particles", 4),
                "transform": {
                    "position": { "x_bits": bits(0.0), "y_bits": bits(0.0) },
                    "angle_bits": bits(0.0)
                },
                "scale_bits": bits(1.0)
            }),
        ),
        record(
            "aabb",
            "particle_contacts",
            5,
            json!({
                "metadata": metadata("aabb", "particle_contacts", 5),
                "lower": { "x_bits": bits(-2.0), "y_bits": bits(-1.0) },
                "upper": { "x_bits": bits(2.0), "y_bits": bits(1.0) }
            }),
        ),
        record(
            "arrow",
            "broad_phase",
            6,
            json!({
                "metadata": metadata("arrow", "broad_phase", 6),
                "start": { "x_bits": bits(0.0), "y_bits": bits(0.0) },
                "end": { "x_bits": bits(2.0), "y_bits": bits(1.0) }
            }),
        ),
        record(
            "label",
            "labels",
            7,
            json!({
                "metadata": metadata("label", "labels", 7),
                "position": { "x_bits": bits(0.5), "y_bits": bits(-0.5) },
                "text": "fixture-a"
            }),
        ),
    ]
}

fn checkpoint(records: Vec<DebugPrimitiveRecord>) -> CanonicalCheckpoint {
    CanonicalCheckpoint::new(
        RequestId::new("request-1").expect("static request ID should be valid"),
        Sha256Hex::new("0".repeat(64)).expect("static digest should be valid"),
        CheckpointId::new("checkpoint-0001").expect("static checkpoint ID should be valid"),
        CheckpointPosition::LogicalStep { ordinal: 1 },
        FloatBits::from_f32(0.0),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        records,
        Vec::new(),
    )
    .expect("test checkpoint should satisfy the canonical contract")
}

fn viewport() -> ProtocolViewport {
    ProtocolViewport::new(100.0, 50.0, 400.0, 200.0, 0.0, 0.0, 10.0)
        .expect("static viewport should be valid")
}

#[test]
fn comparison_styles_fade_matches_and_distinguish_backends() {
    // Arrange
    let original = ProtocolScreenStyle {
        stroke: [10, 20, 30, 200],
        stroke_width: 1.0,
        maybe_fill: Some([40, 50, 60, 100]),
    };

    // Act
    let exact = comparison_style(
        original,
        liquidfun_differential::ComparisonState::ExactMatch,
        ProtocolComparisonBackend::Rust,
    );
    let rust_difference = comparison_style(
        original,
        liquidfun_differential::ComparisonState::PhysicsMismatch,
        ProtocolComparisonBackend::Rust,
    );
    let oracle_difference = comparison_style(
        original,
        liquidfun_differential::ComparisonState::PhysicsMismatch,
        ProtocolComparisonBackend::Oracle,
    );

    // Assert
    assert_eq!(exact.stroke, [10, 20, 30, 70]);
    assert_eq!(exact.maybe_fill, Some([40, 50, 60, 35]));
    assert_eq!(rust_difference.stroke, RUST_COMPARISON_COLOR);
    assert_eq!(oracle_difference.stroke, ORACLE_COMPARISON_COLOR);
    assert!(rust_difference.stroke_width >= 2.0);
    assert!(oracle_difference.maybe_fill.is_none());
}

#[test]
fn hit_testing_selects_the_clicked_semantic_key_instead_of_the_first_record() {
    // Arrange
    let records = all_primitive_records().into_iter().take(2).collect();
    let frame = project_checkpoint(
        &checkpoint(records),
        viewport(),
        ProtocolLayerVisibility::all(),
    )
    .expect("static projected frame should be valid");
    let clicked_segment = ProtocolScreenPoint { x: 300.0, y: 150.0 };

    // Act
    let selected =
        hit_test_frame(&frame, clicked_segment, 3.0).expect("the segment should be hit tested");

    // Assert
    assert_eq!(selected, frame.primitives()[1].key());
    assert_ne!(selected, frame.primitives()[0].key());
    assert!(hit_test_frame(&frame, ProtocolScreenPoint { x: 700.0, y: 700.0 }, 3.0).is_none());
}

#[test]
fn projects_all_wire_variants_in_source_order_with_exact_style() {
    // Arrange
    let checkpoint = checkpoint(all_primitive_records());

    // Act
    let frame = project_checkpoint(&checkpoint, viewport(), ProtocolLayerVisibility::all())
        .expect("bounded fixture should project");

    // Assert
    let primitives = frame.primitives();
    assert_eq!(primitives.len(), 8);
    assert!(matches!(
        primitives[0].primitive(),
        ProtocolDisplayPrimitive::Point { .. }
    ));
    assert!(matches!(
        primitives[1].primitive(),
        ProtocolDisplayPrimitive::Segment { .. }
    ));
    assert!(matches!(
        primitives[2].primitive(),
        ProtocolDisplayPrimitive::Polyline { .. }
    ));
    assert!(matches!(
        primitives[3].primitive(),
        ProtocolDisplayPrimitive::Circle { .. }
    ));
    assert!(matches!(
        primitives[4].primitive(),
        ProtocolDisplayPrimitive::TransformAxes { .. }
    ));
    assert!(matches!(
        primitives[5].primitive(),
        ProtocolDisplayPrimitive::Aabb { .. }
    ));
    assert!(matches!(
        primitives[6].primitive(),
        ProtocolDisplayPrimitive::Arrow { .. }
    ));
    assert!(matches!(
        primitives[7].primitive(),
        ProtocolDisplayPrimitive::Label { .. }
    ));
    assert_eq!(
        primitives[0].ordering(),
        DebugPrimitiveOrder::SourceSignificant
    );
    assert_eq!(primitives[0].style().stroke, [1, 2, 3, 4]);
    assert_eq!(
        primitives[0].style().stroke_width.to_bits(),
        5.0_f32.to_bits()
    );
    assert_eq!(primitives[0].style().maybe_fill, Some([5, 6, 7, 8]));
    let ProtocolDisplayPrimitive::Point { position, radius } = primitives[0].primitive() else {
        panic!("first fixture primitive should remain a point");
    };
    assert_eq!(*position, super::ProtocolScreenPoint { x: 310.0, y: 130.0 });
    assert_eq!(radius.to_bits(), 2.5_f32.to_bits());
}

#[test]
fn filters_by_protocol_semantic_layer_without_reordering() {
    // Arrange
    let checkpoint = checkpoint(all_primitive_records());
    let mut visibility = ProtocolLayerVisibility::none();
    visibility.set(liquidfun_test_protocol::DebugLayerName::Joints, true);
    visibility.set(liquidfun_test_protocol::DebugLayerName::Labels, true);

    // Act
    let frame = project_checkpoint(&checkpoint, viewport(), visibility)
        .expect("visible fixture layers should project");

    // Assert
    assert_eq!(frame.primitives().len(), 2);
    assert_eq!(frame.primitives()[0].key().ordinal(), 1);
    assert_eq!(frame.primitives()[1].key().ordinal(), 7);
}

#[test]
fn rejects_geometry_outside_reviewed_projection_bounds() {
    // Arrange
    let record = record(
        "point",
        "shapes",
        0,
        json!({
            "metadata": metadata("point", "shapes", 0),
            "position": { "x_bits": bits(1_000_001.0), "y_bits": bits(0.0) },
            "radius_bits": bits(1.0)
        }),
    );
    let checkpoint = checkpoint(vec![record]);

    // Act
    let result = project_checkpoint(&checkpoint, viewport(), ProtocolLayerVisibility::all());

    // Assert
    assert_eq!(result, Err(ProtocolViewportError::GeometryOutOfRange));
}

#[test]
fn rejects_nonfinite_or_unbounded_viewport_inputs() {
    // Arrange
    let invalid = [
        (f32::NAN, 0.0, 640.0, 480.0, 0.0, 0.0, 50.0),
        (0.0, 0.0, 0.0, 480.0, 0.0, 0.0, 50.0),
        (0.0, 0.0, 640.0, 480.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 640.0, 480.0, 0.0, 0.0, 4_097.0),
    ];

    // Act
    let viewports = invalid.map(|values| {
        ProtocolViewport::new(
            values.0, values.1, values.2, values.3, values.4, values.5, values.6,
        )
    });

    // Assert
    assert!(viewports.into_iter().all(|value| value.is_none()));
}
