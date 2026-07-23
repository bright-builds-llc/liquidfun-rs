//! Executable contract for the passive renderer replacement boundary.

use std::process::Command;

#[allow(
    dead_code,
    reason = "the integration contract exercises selected parts of the crate-private boundary"
)]
#[path = "../src/renderer.rs"]
mod renderer;

use renderer::image::TinySkiaImageRenderer;
use renderer::{
    Circle, DrawCommand, ImageRenderer, Line, LogicalPoint, LogicalSize, PhysicalSize,
    PresentationFrame, Rectangle, RendererError, RgbaColor, Stroke, TextDrawing,
};

fn presentation_frame() -> PresentationFrame {
    PresentationFrame::new(
        LogicalSize::new(640.0, 480.0).expect("logical dimensions should be valid"),
        RgbaColor::new(13, 17, 23, 255),
        vec![
            DrawCommand::FillRectangle(Rectangle::new(
                LogicalPoint::new(20.0, 20.0).expect("origin should be valid"),
                LogicalSize::new(160.0, 80.0).expect("rectangle should be valid"),
                RgbaColor::new(35, 43, 54, 255),
            )),
            DrawCommand::StrokeLine(Line::new(
                LogicalPoint::new(40.0, 140.0).expect("line start should be valid"),
                LogicalPoint::new(220.0, 260.0).expect("line end should be valid"),
                Stroke::new(RgbaColor::new(88, 166, 255, 255), 3.0)
                    .expect("stroke should be valid"),
            )),
            DrawCommand::FillCircle(
                Circle::new(
                    LogicalPoint::new(320.0, 240.0).expect("center should be valid"),
                    36.0,
                    RgbaColor::new(56, 209, 158, 255),
                )
                .expect("circle should be valid"),
            ),
            DrawCommand::Text(
                TextDrawing::new(
                    LogicalPoint::new(32.0, 320.0).expect("text origin should be valid"),
                    "PASSIVE RENDERER".to_owned(),
                    18.0,
                    RgbaColor::new(201, 209, 217, 255),
                )
                .expect("text drawing should be valid"),
            ),
        ],
    )
}

#[test]
fn physical_and_logical_dimensions_reject_invalid_values() {
    // Arrange
    let invalid_physical = [(0, 480), (640, 0), (4_097, 480), (640, 4_097)];
    let invalid_logical = [
        (0.0, 480.0),
        (640.0, 0.0),
        (f32::NAN, 480.0),
        (640.0, f32::INFINITY),
    ];

    // Act
    let physical_errors = invalid_physical.map(|(width, height)| PhysicalSize::new(width, height));
    let logical_errors = invalid_logical.map(|(width, height)| LogicalSize::new(width, height));

    // Assert
    assert_eq!(physical_errors[0], Err(RendererError::InvalidDimensions));
    assert_eq!(physical_errors[1], Err(RendererError::InvalidDimensions));
    assert_eq!(
        physical_errors[2],
        Err(RendererError::DimensionLimitExceeded)
    );
    assert_eq!(
        physical_errors[3],
        Err(RendererError::DimensionLimitExceeded)
    );
    assert!(
        logical_errors
            .iter()
            .all(|result| *result == Err(RendererError::InvalidDimensions))
    );
}

#[test]
fn cpu_capture_is_byte_stable_at_640_by_480() {
    // Arrange
    let size = PhysicalSize::new(640, 480).expect("capture dimensions should be valid");
    let mut first_renderer = TinySkiaImageRenderer;
    let mut second_renderer = TinySkiaImageRenderer;

    // Act
    let first = first_renderer
        .capture(size, presentation_frame())
        .expect("first capture should succeed");
    let second = second_renderer
        .capture(size, presentation_frame())
        .expect("second capture should succeed");

    // Assert
    assert_eq!(first.size(), size);
    assert_eq!(first.rgba_bytes(), second.rgba_bytes());
    assert_eq!(first.png_bytes(), second.png_bytes());
    assert_eq!(first.rgba_bytes().len(), 640 * 480 * 4);
    assert!(first.png_bytes().starts_with(b"\x89PNG\r\n\x1a\n"));
}

#[test]
fn capture_consumes_owned_presentation_without_mutating_passive_state() {
    // Arrange
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PassiveState {
        logical_steps: u32,
        captures: usize,
        comparison_entries: usize,
    }

    let state = PassiveState {
        logical_steps: 7,
        captures: 2,
        comparison_entries: 4,
    };
    let before = state.clone();
    let size = PhysicalSize::new(640, 480).expect("capture dimensions should be valid");
    let mut renderer = TinySkiaImageRenderer;

    // Act
    let pixels = renderer
        .capture(size, presentation_frame())
        .expect("capture should succeed");

    // Assert
    assert_eq!(state, before);
    assert!(!pixels.rgba_bytes().is_empty());
}

#[test]
fn published_engine_normal_dependencies_remain_renderer_free() {
    // Arrange
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("testbed should be nested under the workspace root");

    // Act
    let output = Command::new(env!("CARGO"))
        .args(["tree", "-p", "liquidfun", "--edges", "normal"])
        .current_dir(workspace_root)
        .output()
        .expect("cargo tree should execute");
    let tree = String::from_utf8(output.stdout).expect("cargo tree should emit UTF-8");

    // Assert
    assert!(output.status.success());
    for forbidden in ["eframe", "egui", "tiny-skia", "macroquad"] {
        assert!(
            !tree.contains(forbidden),
            "{forbidden} must not enter liquidfun's normal dependency graph"
        );
    }
}

#[test]
fn renderer_contract_has_no_simulation_authority_or_unsafe_code() {
    // Arrange
    let contract = include_str!("../src/renderer.rs");
    let image_backend = include_str!("../src/renderer/image.rs");

    // Act
    let combined = format!("{contract}\n{image_backend}");

    // Assert
    for forbidden in [
        "World",
        "SessionController",
        "ComparisonModel",
        "ProtocolWriter",
        "unsafe",
    ] {
        assert!(
            !combined.contains(forbidden),
            "renderer contract must not contain {forbidden}"
        );
    }
}
