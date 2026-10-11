//! Per-step FNV-1a 64 frame hash shared with the Bun spike and replay scripts
//! (Phase 37 D-01): particle positions, then rigid segments, then rigid
//! circles, each value as little-endian f32 bits.

use super::fingerprint::Fnv1a;
use crate::frame::ProofFrame;

/// Scenes stepped by the native-vs-wasm determinism spike.
const SPIKE_SCENES: [&str; 3] = ["dam-break", "water-wheel", "theo-jansen"];
/// Fixed steps per spike scene.
const SPIKE_STEPS: u32 = 300;

/// Hashes every value of every lane in order.
pub(crate) fn frame_hash_lanes(lanes: &[&[f32]]) -> u64 {
    let mut hash = Fnv1a::new();
    for lane in lanes {
        for value in *lane {
            hash.mix_f32(*value);
        }
    }
    hash.finish()
}

/// Hashes a captured frame's particle positions, rigid segments and rigid circles.
pub(crate) fn frame_hash(frame: &ProofFrame) -> u64 {
    let positions = frame.particle_positions();
    let segments = frame.rigid_segments();
    let circles = frame.rigid_circles();
    frame_hash_lanes(&[&positions, &segments, &circles])
}

#[cfg(test)]
mod tests {
    use super::{SPIKE_SCENES, SPIKE_STEPS, frame_hash, frame_hash_lanes};
    use crate::frame::ProofFrame;
    use crate::scene::parse_scene_id;
    use crate::session::SessionCore;

    #[test]
    fn frame_hash_of_no_lanes_is_the_offset_basis() {
        // Arrange
        let lanes: &[&[f32]] = &[];

        // Act
        let hash = frame_hash_lanes(lanes);

        // Assert
        assert_eq!(hash, 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn frame_hash_matches_the_shared_lane_vector() {
        // Arrange
        let lanes: &[&[f32]] = &[&[1.0, -0.0], &[], &[0.5]];

        // Act
        let hash = frame_hash_lanes(lanes);

        // Assert
        assert_eq!(hash, 0x99e0_1cab_84d7_30f5);
    }

    #[test]
    #[ignore = "phase 37 spike; run with --ignored --nocapture"]
    fn spike_prints_per_step_frame_hashes() {
        for scene in SPIKE_SCENES {
            // Arrange
            let id = parse_scene_id(scene).expect("spike scene should be in the catalog");
            let mut core = SessionCore::create(id).expect("spike scene should build");

            // Act
            for step in 1..=SPIKE_STEPS {
                core.advance(1).expect("spike scene should advance");
                let frame = ProofFrame::from(core.capture_frame().expect("frame should capture"));
                println!("SPIKE {scene} {step} {:016x}", frame_hash(&frame));
            }

            // Assert
            assert_eq!(core.step_index(), SPIKE_STEPS);
        }
    }
}
