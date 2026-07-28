use super::{Deserialize, FloatBits, ScenarioId, Serialize, Vec2Bits};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidJointKind {
    Revolute,
    Prismatic,
    Distance,
    Pulley,
    Mouse,
    Gear,
    Wheel,
    Weld,
    Friction,
    Rope,
    Motor,
}

impl RigidJointKind {
    pub const ALL: [Self; 11] = [
        Self::Revolute,
        Self::Prismatic,
        Self::Distance,
        Self::Pulley,
        Self::Mouse,
        Self::Gear,
        Self::Wheel,
        Self::Weld,
        Self::Friction,
        Self::Rope,
        Self::Motor,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidJointDefinition {
    Revolute {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        reference_angle_bits: FloatBits,
        lower_angle_bits: FloatBits,
        upper_angle_bits: FloatBits,
        motor_speed_bits: FloatBits,
        max_motor_torque_bits: FloatBits,
        limit_enabled: bool,
        motor_enabled: bool,
    },
    Prismatic {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        local_axis_a: Vec2Bits,
        reference_angle_bits: FloatBits,
        lower_translation_bits: FloatBits,
        upper_translation_bits: FloatBits,
        motor_speed_bits: FloatBits,
        max_motor_force_bits: FloatBits,
        limit_enabled: bool,
        motor_enabled: bool,
    },
    Distance {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        length_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
    },
    Pulley {
        ground_anchor_a: Vec2Bits,
        ground_anchor_b: Vec2Bits,
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        length_a_bits: FloatBits,
        length_b_bits: FloatBits,
        ratio_bits: FloatBits,
    },
    Mouse {
        target: Vec2Bits,
        max_force_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
    },
    Gear {
        joint_a_id: ScenarioId,
        joint_b_id: ScenarioId,
        ratio_bits: FloatBits,
    },
    Wheel {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        local_axis_a: Vec2Bits,
        motor_speed_bits: FloatBits,
        max_motor_torque_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
        motor_enabled: bool,
    },
    Weld {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        reference_angle_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
    },
    Friction {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        max_force_bits: FloatBits,
        max_torque_bits: FloatBits,
    },
    Rope {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        max_length_bits: FloatBits,
    },
    Motor {
        linear_offset: Vec2Bits,
        angular_offset_bits: FloatBits,
        max_force_bits: FloatBits,
        max_torque_bits: FloatBits,
        correction_factor_bits: FloatBits,
    },
}

impl RigidJointDefinition {
    #[must_use]
    pub const fn joint_kind(&self) -> RigidJointKind {
        match self {
            Self::Revolute { .. } => RigidJointKind::Revolute,
            Self::Prismatic { .. } => RigidJointKind::Prismatic,
            Self::Distance { .. } => RigidJointKind::Distance,
            Self::Pulley { .. } => RigidJointKind::Pulley,
            Self::Mouse { .. } => RigidJointKind::Mouse,
            Self::Gear { .. } => RigidJointKind::Gear,
            Self::Wheel { .. } => RigidJointKind::Wheel,
            Self::Weld { .. } => RigidJointKind::Weld,
            Self::Friction { .. } => RigidJointKind::Friction,
            Self::Rope { .. } => RigidJointKind::Rope,
            Self::Motor { .. } => RigidJointKind::Motor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidJointDeclaration {
    pub joint_id: ScenarioId,
    pub body_a_id: ScenarioId,
    pub body_b_id: ScenarioId,
    pub collide_connected: bool,
    pub definition: RigidJointDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRopeDeclaration {
    pub rope_id: ScenarioId,
    pub vertices: Box<[Vec2Bits]>,
    pub masses_bits: Box<[FloatBits]>,
    pub gravity: Vec2Bits,
    pub damping_bits: FloatBits,
    pub stretch_stiffness_bits: FloatBits,
    pub bend_stiffness_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidJointMutation {
    LimitEnabled {
        enabled: bool,
    },
    Limits {
        lower_bits: FloatBits,
        upper_bits: FloatBits,
    },
    MotorEnabled {
        enabled: bool,
    },
    MotorSpeed {
        speed_bits: FloatBits,
    },
    MaxMotorForce {
        force_bits: FloatBits,
    },
    MaxMotorTorque {
        torque_bits: FloatBits,
    },
    Length {
        length_bits: FloatBits,
    },
    Frequency {
        frequency_bits: FloatBits,
    },
    DampingRatio {
        damping_ratio_bits: FloatBits,
    },
    MouseTarget {
        target: Vec2Bits,
    },
    MaxForce {
        force_bits: FloatBits,
    },
    MaxTorque {
        torque_bits: FloatBits,
    },
    GearRatio {
        ratio_bits: FloatBits,
    },
    RopeMaxLength {
        max_length_bits: FloatBits,
    },
    LinearOffset {
        offset: Vec2Bits,
    },
    AngularOffset {
        offset_bits: FloatBits,
    },
    CorrectionFactor {
        factor_bits: FloatBits,
    },
}
