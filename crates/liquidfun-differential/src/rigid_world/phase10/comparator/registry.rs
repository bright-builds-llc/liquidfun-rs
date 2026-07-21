//! Closed Phase 10 result-field policy registry.

const DEFAULT_MAX_ULPS: u32 = 4;
const DEFAULT_ABSOLUTE: f32 = 1.0e-6;
const DEFAULT_RELATIVE: f32 = 1.0e-5;
const DEFAULT_DIMENSIONED_ABSOLUTE: f32 = 1.0e-5;

/// Named comparison class assigned to one closed Phase 10 semantic path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase10PolicyKind {
    /// Identity, ordering, multiplicity, branch, or count equality.
    ExactDiscrete,
    /// IEEE-754 or byte field equality.
    ExactBits,
    /// Maximum ordered IEEE-754 representation distance.
    Ulps {
        /// Inclusive maximum representable-value distance.
        maximum: u32,
    },
    /// Absolute-or-relative bound for accumulated state.
    AbsoluteRelative {
        /// Inclusive absolute floor.
        absolute: f32,
        /// Inclusive scale-relative multiplier.
        relative: f32,
    },
    /// Unit-specific absolute bound.
    DimensionedAbsolute {
        /// Inclusive unit-specific absolute threshold.
        maximum: f32,
    },
}

/// One reviewed path-to-policy binding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Phase10Policy {
    /// Closed semantic path.
    pub path: &'static str,
    /// Comparison class and its fixed threshold.
    pub kind: Phase10PolicyKind,
}

const fn exact(path: &'static str) -> Phase10Policy {
    Phase10Policy {
        path,
        kind: Phase10PolicyKind::ExactDiscrete,
    }
}

const fn bits(path: &'static str) -> Phase10Policy {
    Phase10Policy {
        path,
        kind: Phase10PolicyKind::ExactBits,
    }
}

const fn ulps(path: &'static str) -> Phase10Policy {
    Phase10Policy {
        path,
        kind: Phase10PolicyKind::Ulps {
            maximum: DEFAULT_MAX_ULPS,
        },
    }
}

const fn accumulated(path: &'static str) -> Phase10Policy {
    Phase10Policy {
        path,
        kind: Phase10PolicyKind::AbsoluteRelative {
            absolute: DEFAULT_ABSOLUTE,
            relative: DEFAULT_RELATIVE,
        },
    }
}

const fn dimensioned(path: &'static str) -> Phase10Policy {
    Phase10Policy {
        path,
        kind: Phase10PolicyKind::DimensionedAbsolute {
            maximum: DEFAULT_DIMENSIONED_ABSOLUTE,
        },
    }
}

/// Complete reviewed Phase 10 result-field policy registry in source order.
pub const PHASE10_POLICY_REGISTRY: &[Phase10Policy] = &[
    exact("phase10.provenance"),
    exact("phase10.outcome"),
    exact("phase10.group.ordinal"),
    exact("phase10.group.identity"),
    exact("phase10.group.membership"),
    bits("phase10.group.flags"),
    ulps("phase10.group.transform"),
    ulps("phase10.group.center"),
    ulps("phase10.group.linear_velocity"),
    ulps("phase10.group.angular_velocity"),
    accumulated("phase10.group.mass"),
    accumulated("phase10.group.inertia"),
    ulps("phase10.group.depth"),
    exact("phase10.particle.identity"),
    ulps("phase10.particle.position"),
    ulps("phase10.particle.velocity"),
    bits("phase10.particle.flags"),
    bits("phase10.particle.color"),
    accumulated("phase10.particle.weight"),
    exact("phase10.pair.ordinal"),
    exact("phase10.pair.identity"),
    bits("phase10.pair.flags"),
    bits("phase10.pair.strength"),
    ulps("phase10.pair.distance"),
    exact("phase10.triad.ordinal"),
    exact("phase10.triad.identity"),
    bits("phase10.triad.flags"),
    bits("phase10.triad.strength"),
    ulps("phase10.triad.offset"),
    ulps("phase10.triad.coefficient"),
    exact("phase10.contact.ordinal"),
    exact("phase10.contact.identity"),
    bits("phase10.contact.flags"),
    accumulated("phase10.contact.weight"),
    ulps("phase10.contact.normal"),
    dimensioned("phase10.body_contact.mass"),
    exact("phase10.event.ordinal"),
    exact("phase10.event.kind"),
    exact("phase10.event.identity"),
    exact("phase10.witness.ordinal"),
    exact("phase10.witness.leaf"),
    exact("phase10.witness.role"),
    exact("phase10.witness.kind"),
    bits("phase10.witness.flags"),
    ulps("phase10.witness.velocity"),
    accumulated("phase10.witness.scalar"),
    exact("phase10.witness.count"),
    exact("phase10.witness.occurrence"),
    exact("phase10.witness.topology"),
    exact("phase10.d0.bytes"),
];

/// Every path required by the exhaustive comparator.
pub const PHASE10_REQUIRED_POLICY_PATHS: &[&str] = &[
    "phase10.provenance",
    "phase10.outcome",
    "phase10.group.ordinal",
    "phase10.group.identity",
    "phase10.group.membership",
    "phase10.group.flags",
    "phase10.group.transform",
    "phase10.group.center",
    "phase10.group.linear_velocity",
    "phase10.group.angular_velocity",
    "phase10.group.mass",
    "phase10.group.inertia",
    "phase10.group.depth",
    "phase10.particle.identity",
    "phase10.particle.position",
    "phase10.particle.velocity",
    "phase10.particle.flags",
    "phase10.particle.color",
    "phase10.particle.weight",
    "phase10.pair.ordinal",
    "phase10.pair.identity",
    "phase10.pair.flags",
    "phase10.pair.strength",
    "phase10.pair.distance",
    "phase10.triad.ordinal",
    "phase10.triad.identity",
    "phase10.triad.flags",
    "phase10.triad.strength",
    "phase10.triad.offset",
    "phase10.triad.coefficient",
    "phase10.contact.ordinal",
    "phase10.contact.identity",
    "phase10.contact.flags",
    "phase10.contact.weight",
    "phase10.contact.normal",
    "phase10.body_contact.mass",
    "phase10.event.ordinal",
    "phase10.event.kind",
    "phase10.event.identity",
    "phase10.witness.ordinal",
    "phase10.witness.leaf",
    "phase10.witness.role",
    "phase10.witness.kind",
    "phase10.witness.flags",
    "phase10.witness.velocity",
    "phase10.witness.scalar",
    "phase10.witness.count",
    "phase10.witness.occurrence",
    "phase10.witness.topology",
    "phase10.d0.bytes",
];
