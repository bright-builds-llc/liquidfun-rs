pub(super) fn rendered_evidence_with_contact_normals(
    emit_contact_normals: bool,
) -> RenderedEvidence {
    let fixture = FixtureSnapshot {
        sha256: "0".repeat(64),
        profile: "phase11-v1".to_owned(),
        upstream_revision: "0".repeat(40),
        case_ids: vec!["test-case".to_owned()],
        families: vec!["rigid".to_owned()],
        verified_artifacts: 1,
    };
    let mut raster = RendererRaster::default();
    let semantic_evidence = draw_capability_scene(
        &mut raster,
        &fixture,
        "SESSION READY",
        "COMPARE EXACT",
        emit_contact_normals,
    );
    let artifacts = FRAME_SIZES
        .iter()
        .map(|(width, height, name)| {
            CapabilityArtifact::new(
                (*name).to_owned(),
                "0".repeat(64),
                1_024,
                *width,
                *height,
                true,
            )
        })
        .collect();
    RenderedEvidence {
        artifacts,
        minimum_width: 640,
        minimum_height: 480,
        maximum_dpi_scale: 2,
        resize_width: 800,
        resize_height: 600,
        non_background_pixels_minimum: raster.commands.len(),
        distinct_particle_colors: semantic_evidence.distinct_particle_colors(),
        dense_text_rows: semantic_evidence.dense_text_rows,
        focus_ring_pixels: semantic_evidence.focus_ring_pixels,
        minimum_text_contrast_ratio: contrast_ratio(TEXT, BACKGROUND),
        minimum_control_target_pixels: semantic_evidence.minimum_control_target_pixels,
        keyboard_bindings: 6,
        contact_points: semantic_evidence.contact_points,
        contact_normals: semantic_evidence.contact_normals,
        particle_contacts: semantic_evidence.particle_contacts,
        broad_phase_aabbs: semantic_evidence.broad_phase_aabbs,
        profile_names: semantic_evidence.profile_names,
        overlay_pairs: semantic_evidence.overlay_pairs,
        side_by_side_panels: semantic_evidence.side_by_side_panels,
        semantic_capture_acknowledgements: semantic_evidence.semantic_capture_acknowledgements,
        diagnostic_disclaimer_lines: semantic_evidence.diagnostic_disclaimer_lines,
    }
}
