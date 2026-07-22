//! Independent semantic debug-layer and diagnostic overlay controls.

#![allow(
    missing_docs,
    reason = "closed overlay variants are named by the UI contract"
)]

use liquidfun::DebugLayer;

pub const MODULE_NAME: &str = "overlays";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    Shapes,
    Joints,
    Contacts,
    ContactNormals,
    ParticleContacts,
    BroadPhase,
    CentersOfMass,
    Statistics,
    Profiles,
}

/// Presentation-only visibility. Toggling a layer never creates a controller command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayState {
    enabled: [bool; 9],
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            enabled: [true, true, true, true, true, false, false, true, true],
        }
    }
}

impl OverlayState {
    #[must_use]
    pub const fn enabled(self, kind: OverlayKind) -> bool {
        self.enabled[kind.index()]
    }

    pub fn toggle(&mut self, kind: OverlayKind) {
        let index = kind.index();
        self.enabled[index] = !self.enabled[index];
    }

    /// Implements the exact `1` through `4` debug-layer shortcut groups.
    pub fn toggle_shortcut_group(&mut self, group: u8) {
        match group {
            1 => self.toggle(OverlayKind::Contacts),
            2 => self.toggle(OverlayKind::ParticleContacts),
            3 => self.toggle(OverlayKind::BroadPhase),
            4 => {
                let enable =
                    !(self.enabled(OverlayKind::Statistics) && self.enabled(OverlayKind::Profiles));
                self.enabled[OverlayKind::Statistics.index()] = enable;
                self.enabled[OverlayKind::Profiles.index()] = enable;
            }
            _ => {}
        }
    }

    #[must_use]
    pub const fn layer_visible(self, layer: DebugLayer) -> bool {
        match layer {
            DebugLayer::Shapes | DebugLayer::Particles | DebugLayer::Labels => {
                self.enabled(OverlayKind::Shapes)
            }
            DebugLayer::Joints => self.enabled(OverlayKind::Joints),
            DebugLayer::Contacts => self.enabled(OverlayKind::Contacts),
            DebugLayer::ContactNormals => self.enabled(OverlayKind::ContactNormals),
            DebugLayer::ParticleContacts => self.enabled(OverlayKind::ParticleContacts),
            DebugLayer::BroadPhase => self.enabled(OverlayKind::BroadPhase),
            DebugLayer::CentersOfMass => self.enabled(OverlayKind::CentersOfMass),
        }
    }
}

impl OverlayKind {
    const fn index(self) -> usize {
        match self {
            Self::Shapes => 0,
            Self::Joints => 1,
            Self::Contacts => 2,
            Self::ContactNormals => 3,
            Self::ParticleContacts => 4,
            Self::BroadPhase => 5,
            Self::CentersOfMass => 6,
            Self::Statistics => 7,
            Self::Profiles => 8,
        }
    }
}

/// Bounded semantic counts displayed without private storage identities.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatisticsOverlay {
    pub bodies: u32,
    pub contacts: u32,
    pub joints: u32,
    pub proxies: u32,
    pub particles: u32,
    pub particle_contacts: u32,
}

/// One wall-clock profile row. Its numeric duration is explicitly diagnostic-only.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticProfile {
    name: Box<str>,
    duration_micros: f32,
}

impl DiagnosticProfile {
    #[must_use]
    pub fn new(name: &str, duration_micros: f32) -> Option<Self> {
        if name.is_empty()
            || name.len() > 64
            || !name.is_ascii()
            || name.bytes().any(|byte| byte.is_ascii_control())
            || !duration_micros.is_finite()
            || duration_micros < 0.0
        {
            return None;
        }
        Some(Self {
            name: name.into(),
            duration_micros,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn duration_micros(&self) -> f32 {
        self.duration_micros
    }

    #[must_use]
    pub const fn authority_label(&self) -> &'static str {
        "Diagnostic timing — excluded from compatibility authority"
    }
}
