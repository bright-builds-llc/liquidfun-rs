//! Circular drum wall and the ribs that turn with it.

use std::f32::consts::TAU;

use liquidfun::math::Vec2;

/// Drum center. The body, joint, and camera share this origin.
pub(super) const DRUM_CENTER: Vec2 = Vec2::ZERO;
/// Inside face of the drum wall, in meters.
pub(super) const INNER_RADIUS: f32 = 1.28;
/// Wall thickness. Thicker than one step of motion at the fastest spin.
pub(super) const WALL_THICKNESS: f32 = 0.20;
pub(super) const OUTER_RADIUS: f32 = INNER_RADIUS + WALL_THICKNESS;
/// Facets in the circular wall. Shared vertices meet, so the drum has no gaps.
pub(super) const SEGMENT_COUNT: usize = 24;
pub(super) const RIB_COUNT: usize = 4;
/// How far a rib reaches in from the inner wall, in meters.
const RIB_DEPTH: f32 = 0.42;
/// Half-width of a rib across its spin direction, in meters.
const RIB_HALF_THICKNESS: f32 = 0.07;
/// How far a rib sits into the wall so particles cannot slip behind it.
const RIB_WALL_OVERLAP: f32 = 0.04;
/// First rib sits on a diagonal, clear of the resting water.
const RIB_PHASE: f32 = TAU / 8.0;

pub(super) type Quad = [Vec2; 4];

pub(super) fn wall_quads() -> [Quad; SEGMENT_COUNT] {
    ring_quads(true)
}

/// Closed inner and outer chords. Fixtures overlap; this ring is only for drawing.
pub(super) fn outline_wall_quads() -> [Quad; SEGMENT_COUNT] {
    ring_quads(false)
}

fn ring_quads(overlap_seams: bool) -> [Quad; SEGMENT_COUNT] {
    let mut quads = [[Vec2::ZERO; 4]; SEGMENT_COUNT];
    for (index, quad) in quads.iter_mut().enumerate() {
        *quad = wall_quad(index, overlap_seams);
    }
    quads
}

pub(super) fn rib_quads() -> [Quad; RIB_COUNT] {
    let mut quads = [[Vec2::ZERO; 4]; RIB_COUNT];
    for (index, quad) in quads.iter_mut().enumerate() {
        *quad = rib_quad(index);
    }
    quads
}

fn wall_quad(index: usize, overlap_seams: bool) -> Quad {
    let step = TAU / f32::from(u16_index(SEGMENT_COUNT));
    // Neighboring fixtures overlap so a particle cannot slip out along a seam.
    let overlap = if overlap_seams { step * 0.45 } else { 0.0 };
    let start = step * f32::from(u16_index(index)) - overlap;
    let end = start + step + overlap * 2.0;
    [
        polar(INNER_RADIUS, start),
        polar(INNER_RADIUS, end),
        polar(OUTER_RADIUS, end),
        polar(OUTER_RADIUS, start),
    ]
}

fn rib_quad(index: usize) -> Quad {
    let theta = RIB_PHASE + TAU * f32::from(u16_index(index)) / f32::from(u16_index(RIB_COUNT));
    let center_radius = INNER_RADIUS - RIB_DEPTH * 0.5 + RIB_WALL_OVERLAP;
    let center = polar(center_radius, theta);
    let half_radial = RIB_DEPTH * 0.5;
    let local = [
        Vec2::new(-half_radial, -RIB_HALF_THICKNESS),
        Vec2::new(half_radial, -RIB_HALF_THICKNESS),
        Vec2::new(half_radial, RIB_HALF_THICKNESS),
        Vec2::new(-half_radial, RIB_HALF_THICKNESS),
    ];
    [
        center + rotate(local[0], theta),
        center + rotate(local[1], theta),
        center + rotate(local[2], theta),
        center + rotate(local[3], theta),
    ]
}

fn polar(radius: f32, angle: f32) -> Vec2 {
    Vec2::new(radius * angle.cos(), radius * angle.sin())
}

fn rotate(point: Vec2, angle: f32) -> Vec2 {
    let (sin, cos) = angle.sin_cos();
    Vec2::new(point.x * cos - point.y * sin, point.x * sin + point.y * cos)
}

fn u16_index(index: usize) -> u16 {
    u16::try_from(index).unwrap_or(u16::MAX)
}
