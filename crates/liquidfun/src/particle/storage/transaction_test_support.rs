//! Private fixture support for complete world transaction assertions.

use super::*;

impl ParticleStorage {
    pub(crate) fn next_particle_for_test(&self) -> ParticleId {
        let (slot, generation, _) = self.identity_slot_candidate().expect("next identity fits");
        ParticleId::from_identity(Identity::new_particle(
            self.world,
            self.identity_slot_base + slot,
            generation,
            self.system.identity(),
        ))
    }

    pub(crate) fn populate_transaction_test_state(&mut self) {
        // Public operations supply topology, contacts, lifetimes and vacant identities.
        // Only inaccessible optional metadata and impractical generation exhaustion
        // are seeded here; no candidate failure is injected.
        let count = self.len();
        self.maybe_user_associations = Some(vec![Some(UserAssociationKey::new(17)); count]);
        self.update_stuck_candidates(7, 1);
        self.ensure_static_pressures()
            .expect("fixture has static pressure");
        self.ensure_tensile_accumulations()
            .expect("fixture has tensile flags");
        self.ensure_depths().expect("fixture has a solid group");
        self.replace_static_pressures(vec![0.25; count])
            .expect("aligned finite pressure");
        self.replace_tensile_accumulations(vec![Vec2::new(0.5, 0.25); count])
            .expect("aligned finite accumulation");
        self.replace_depths(vec![0.75; count])
            .expect("aligned finite depths");
        self.replace_force_range(0..count, &vec![Vec2::new(0.1, 0.2); count]);
        assert!(self.identities.len() < self.identity_capacity);
        self.identities.push(IdentityEntry {
            generation: u64::MAX,
            diagnostic_id: None,
            state: IdentityState::Retired,
        });
        self.retired_identity_slots += 1;
        self.assert_transaction_test_state();
    }

    pub(crate) fn assert_transaction_test_state(&self) {
        assert_eq!(self.check_invariants(), Ok(()));
        assert!(!self.free_identity_slots.is_empty());
        assert!(self.retired_identity_slots > 0);
        assert!(self.pending_count() > 0);
        assert!(
            self.flags
                .iter()
                .any(|flags| flags.contains(ParticleFlags::ZOMBIE))
        );
        assert!(self.maybe_colors.is_some());
        assert!(self.maybe_user_associations.is_some());
        assert!(self.maybe_stuck.is_some());
        assert!(self.maybe_expiration_times.is_some());
        assert!(self.maybe_expiration_order.is_some());
        assert!(!self.proxies.is_empty());
        assert!(!self.particle_contacts.is_empty());
        assert!(!self.body_contacts.is_empty());
        assert!(!self.pairs.is_empty());
        assert!(!self.triads.is_empty());
        assert!(self.weights.iter().any(|weight| *weight > 0.0));
        assert!(self.solver_state.has_pending_system_force());
        assert!(self.solver_state.maybe_static_pressures().is_some());
        assert!(self.solver_state.maybe_tensile_accumulations().is_some());
        assert!(self.solver_state.maybe_depths().is_some());
    }
}
