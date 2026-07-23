//! Compiled regression coverage for the desktop comparison diagnostics lifecycle.

#[allow(dead_code, unused_imports)]
#[path = "../src/bin/interactive.rs"]
mod interactive;

use interactive::DesktopDiagnostics;

#[test]
fn comparison_failure_then_success_retires_the_stale_identity_error() {
    // Arrange
    let mut diagnostics = DesktopDiagnostics::<u32, &str>::default();
    diagnostics.apply_comparison(
        "identity-a",
        Err("checkpoint comparison identity mismatch: resolved_sha256".to_owned()),
    );

    // Act
    diagnostics.apply_comparison("identity-b", Ok(7));

    // Assert
    assert_eq!(diagnostics.maybe_comparison(), Some(&7));
    assert_eq!(diagnostics.maybe_compared_identity(), Some(&"identity-b"));
    assert_eq!(diagnostics.maybe_comparison_error(), None);
}

#[test]
fn comparison_failure_then_reset_clears_comparison_state() {
    // Arrange
    let mut diagnostics = DesktopDiagnostics::<u32, &str>::default();
    diagnostics.apply_comparison(
        "identity-a",
        Err("checkpoint comparison identity mismatch: resolved_sha256".to_owned()),
    );

    // Act
    diagnostics.reset_comparison();

    // Assert
    assert_eq!(diagnostics.maybe_comparison(), None);
    assert_eq!(diagnostics.maybe_compared_identity(), None);
    assert_eq!(diagnostics.maybe_comparison_error(), None);
}

#[test]
fn generic_error_survives_comparison_success_and_reset() {
    // Arrange
    let mut diagnostics = DesktopDiagnostics::<u32, &str>::default();
    let generic_error = "controller capture failed: bounded generic detail";
    diagnostics.set_error(generic_error);

    // Act
    diagnostics.apply_comparison(
        "identity-a",
        Err("checkpoint comparison identity mismatch: resolved_sha256".to_owned()),
    );

    // Assert
    assert_eq!(diagnostics.maybe_error(), Some(generic_error));
    assert_eq!(diagnostics.maybe_comparison(), None);
    assert_eq!(diagnostics.maybe_compared_identity(), Some(&"identity-a"));
    assert!(diagnostics.maybe_comparison_error().is_some());

    // Act
    diagnostics.apply_comparison("identity-b", Ok(11));

    // Assert
    assert_eq!(diagnostics.maybe_error(), Some(generic_error));
    assert_eq!(diagnostics.maybe_comparison(), Some(&11));
    assert_eq!(diagnostics.maybe_compared_identity(), Some(&"identity-b"));
    assert_eq!(diagnostics.maybe_comparison_error(), None);

    // Act
    diagnostics.reset_comparison();

    // Assert
    assert_eq!(diagnostics.maybe_error(), Some(generic_error));
    assert_eq!(diagnostics.maybe_comparison(), None);
    assert_eq!(diagnostics.maybe_compared_identity(), None);
    assert_eq!(diagnostics.maybe_comparison_error(), None);
}
