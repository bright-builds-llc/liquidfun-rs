//! Successful capacity reuse and failed candidate validation.
use super::*;
use crate::identity::{HandleIdentity, Identity, WorldKey};

#[test]
fn successful_boundary_buffers_are_reused_across_candidate_sizes() {
    // Arrange
    let world = WorldKey::fresh().expect("world");
    let owner = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let ids: Vec<_> = (0..16)
        .map(|slot| {
            ParticleId::from_identity(Identity::new_particle(world, slot, 0, owner.identity()))
        })
        .collect();
    let positions = [Vec2::ZERO; 16];
    let mut buffers = BoundaryBuffers::default();
    let mut previous_pointer = std::ptr::null::<Vec2>();

    // Act / Assert
    for count in [16, 3, 0, 16] {
        let candidate = BoundaryCandidate::new_with_buffers(
            owner,
            &ids[..count],
            &positions[..count],
            &positions[..count],
            &positions[..count],
            &vec![ParticleFlags::WATER; count],
            &vec![None; count],
            &[],
            false,
            count,
            &mut buffers,
        )
        .expect("candidate");
        assert_eq!(candidate.positions.len(), count);
        assert!(candidate.effects.is_empty());
        assert!(candidate.pass_trace.is_empty());
        if !previous_pointer.is_null() {
            assert_eq!(
                candidate.positions.as_ptr(),
                previous_pointer,
                "capacity should survive recycle"
            );
        }
        previous_pointer = candidate.positions.as_ptr();
        candidate.recycle(&mut buffers);
        assert!(buffers.positions.is_empty());
        assert!(buffers.particle_ids.is_empty());
    }
}
