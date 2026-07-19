use crate::identity::{HandleIdentity, Identity, WorldKey};

use super::*;

mod properties;

fn owner() -> ParticleSystemId {
    let world = WorldKey::fresh().expect("test world key remains available");
    ParticleSystemId::from_identity(Identity::new(world, 0, 0))
}

fn limits() -> VoronoiLimits {
    VoronoiLimits::new(64, 4_096, 16_384, 2_000_000, 8_192)
}

fn contact(a: usize, b: usize, flags: ParticleFlags) -> ParticleContact {
    ParticleContact {
        indices: [ParticleIndex(a), ParticleIndex(b)],
        flags,
        weight: 0.5,
        normal: Vec2::new(1.0, 0.0),
    }
}

#[derive(Clone)]
struct TestFilter {
    necessary: Vec<bool>,
    allow_pairs: bool,
    allow_triads: bool,
}

impl TestFilter {
    fn all(count: usize) -> Self {
        Self {
            necessary: vec![true; count],
            allow_pairs: true,
            allow_triads: true,
        }
    }
}

impl ConnectionFilter for TestFilter {
    fn is_necessary(&self, index: ParticleIndex) -> bool {
        self.necessary.get(index.0).copied().unwrap_or(false)
    }

    fn should_create_pair(&self, _indices: [ParticleIndex; 2]) -> bool {
        self.allow_pairs
    }

    fn should_create_triad(&self, _indices: [ParticleIndex; 3]) -> bool {
        self.allow_triads
    }
}

struct Fixture {
    owner: ParticleSystemId,
    positions: Vec<Vec2>,
    flags: Vec<ParticleFlags>,
    groups: Vec<Option<TopologyGroup>>,
    contacts: Vec<ParticleContact>,
    particle_diameter: f32,
}

impl Fixture {
    fn new(positions: Vec<Vec2>, flags: Vec<ParticleFlags>) -> Self {
        let count = positions.len();
        Self {
            owner: owner(),
            positions,
            flags,
            groups: vec![None; count],
            contacts: Vec::new(),
            particle_diameter: 1.0,
        }
    }

    fn input(&self) -> TopologyInput<'_> {
        TopologyInput {
            owner: self.owner,
            positions: &self.positions,
            flags: &self.flags,
            groups: &self.groups,
            contacts: &self.contacts,
            range: 0..self.positions.len(),
            particle_diameter: self.particle_diameter,
            voronoi_limits: limits(),
        }
    }
}

#[test]
fn pair_generation_preserves_contact_order_rest_distance_flags_and_minimum_strength() {
    // Arrange
    let mut fixture = Fixture::new(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(3.0, 4.0),
            Vec2::new(3.0, 0.0),
        ],
        vec![
            ParticleFlags::SPRING,
            ParticleFlags::WALL,
            ParticleFlags::SPRING,
        ],
    );
    fixture.groups = vec![
        Some(TopologyGroup::new(
            fixture.owner,
            ParticleGroupFlags::empty(),
            0.75,
        )),
        Some(TopologyGroup::new(
            fixture.owner,
            ParticleGroupFlags::empty(),
            0.25,
        )),
        None,
    ];
    fixture.contacts = vec![
        contact(2, 1, ParticleFlags::SPRING | ParticleFlags::WALL),
        contact(0, 1, ParticleFlags::SPRING),
    ];

    // Act
    let contact_order_pairs =
        generate_pairs(&fixture.input(), &TestFilter::all(3)).expect("pairs should generate");
    let generated = generate_pairs_and_triads(&fixture.input(), &TestFilter::all(3))
        .expect("finite connected pairs should generate");

    // Assert
    assert_eq!(
        contact_order_pairs
            .iter()
            .map(|pair| pair.indices)
            .collect::<Vec<_>>(),
        vec![
            [ParticleIndex(2), ParticleIndex(1)],
            [ParticleIndex(0), ParticleIndex(1)]
        ]
    );
    assert_eq!(
        generated
            .pairs
            .iter()
            .map(|pair| pair.indices)
            .collect::<Vec<_>>(),
        vec![
            [ParticleIndex(0), ParticleIndex(1)],
            [ParticleIndex(2), ParticleIndex(1)]
        ]
    );
    assert_eq!(generated.pairs[0].flags, ParticleFlags::SPRING);
    assert_eq!(generated.pairs[0].distance.to_bits(), 5.0_f32.to_bits());
    assert_eq!(generated.pairs[0].strength.to_bits(), 0.25_f32.to_bits());
    assert_eq!(generated.pairs[1].distance.to_bits(), 4.0_f32.to_bits());
}

#[test]
fn pair_gate_rejects_missing_pair_flag_zombie_range_necessity_and_filter_veto() {
    // Arrange
    let cases = [
        (
            [ParticleFlags::WALL, ParticleFlags::WALL],
            0..2,
            vec![true, true],
            true,
        ),
        (
            [
                ParticleFlags::SPRING | ParticleFlags::ZOMBIE,
                ParticleFlags::WALL,
            ],
            0..2,
            vec![true, true],
            true,
        ),
        (
            [ParticleFlags::SPRING, ParticleFlags::WALL],
            0..1,
            vec![true, true],
            true,
        ),
        (
            [ParticleFlags::SPRING, ParticleFlags::WALL],
            0..2,
            vec![false, false],
            true,
        ),
        (
            [ParticleFlags::SPRING, ParticleFlags::WALL],
            0..2,
            vec![true, true],
            false,
        ),
    ];

    for (flags, range, necessary, allow_pairs) in cases {
        let mut fixture = Fixture::new(
            vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)],
            flags.to_vec(),
        );
        fixture.contacts.push(contact(0, 1, ParticleFlags::SPRING));
        let mut input = fixture.input();
        input.range = range;
        let filter = TestFilter {
            necessary,
            allow_pairs,
            allow_triads: true,
        };

        // Act
        let generated = generate_pairs_and_triads(&input, &filter)
            .expect("ineligible finite pair should be omitted");

        // Assert
        assert!(generated.pairs.is_empty());
    }
}

#[test]
fn rigid_group_supplies_connectivity_but_foreign_group_is_rejected() {
    // Arrange
    let mut fixture = Fixture::new(
        vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)],
        vec![ParticleFlags::BARRIER, ParticleFlags::WALL],
    );
    fixture.contacts.push(contact(0, 1, ParticleFlags::BARRIER));
    fixture.groups[0] = Some(TopologyGroup::new(
        fixture.owner,
        ParticleGroupFlags::RIGID,
        0.5,
    ));

    // Act
    let connected = generate_pairs_and_triads(&fixture.input(), &TestFilter::all(2))
        .expect("rigid group should supply connectivity");
    fixture.groups[0] = Some(TopologyGroup::new(owner(), ParticleGroupFlags::RIGID, 0.5));
    let foreign = generate_pairs_and_triads(&fixture.input(), &TestFilter::all(2));

    // Assert
    assert_eq!(connected.pairs.len(), 1);
    assert_eq!(connected.pairs[0].flags, ParticleFlags::BARRIER);
    assert_eq!(foreign, Err(ConstraintError::ForeignGroupOwner));
}

#[test]
fn barrier_pair_is_preserved_while_zero_length_pair_is_a_named_error() {
    // Arrange
    let mut barrier = Fixture::new(
        vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)],
        vec![
            ParticleFlags::BARRIER | ParticleFlags::WALL,
            ParticleFlags::WALL,
        ],
    );
    barrier.contacts.push(contact(0, 1, ParticleFlags::BARRIER));
    let mut zero = Fixture::new(
        vec![Vec2::ZERO, Vec2::ZERO],
        vec![ParticleFlags::SPRING, ParticleFlags::WALL],
    );
    zero.contacts.push(contact(0, 1, ParticleFlags::SPRING));

    // Act
    let barrier_result = generate_pairs_and_triads(&barrier.input(), &TestFilter::all(2));
    let zero_result = generate_pairs_and_triads(&zero.input(), &TestFilter::all(2));

    // Assert
    assert_eq!(
        barrier_result
            .expect("probe preserves finite barrier record")
            .pairs
            .len(),
        1
    );
    assert_eq!(zero_result, Err(ConstraintError::ZeroLengthPairDistance));
}

#[test]
fn triad_generation_preserves_voronoi_orientation_and_every_rest_field() {
    // Arrange
    let mut fixture = Fixture::new(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ],
        vec![ParticleFlags::ELASTIC; 4],
    );
    fixture.groups = vec![
        Some(TopologyGroup::new(
            fixture.owner,
            ParticleGroupFlags::empty(),
            0.8,
        )),
        Some(TopologyGroup::new(
            fixture.owner,
            ParticleGroupFlags::empty(),
            0.6,
        )),
        Some(TopologyGroup::new(
            fixture.owner,
            ParticleGroupFlags::empty(),
            0.4,
        )),
        None,
    ];

    // Act
    let generated = generate_pairs_and_triads(&fixture.input(), &TestFilter::all(4))
        .expect("finite elastic square should generate");

    // Assert
    assert_eq!(
        generated
            .triads
            .iter()
            .map(|triad| triad.indices)
            .collect::<Vec<_>>(),
        vec![
            [ParticleIndex(0), ParticleIndex(1), ParticleIndex(2)],
            [ParticleIndex(1), ParticleIndex(3), ParticleIndex(2)]
        ]
    );
    for triad in &generated.triads {
        assert_triad_rest_state(*triad, &fixture.positions, &fixture.groups);
    }
}

fn assert_triad_rest_state(
    triad: ParticleTriad,
    positions: &[Vec2],
    groups: &[Option<TopologyGroup>],
) {
    let [a, b, c] = triad.indices.map(|index| positions[index.0]);
    let ab = a - b;
    let bc = b - c;
    let ca = c - a;
    let midpoint = (1.0 / 3.0) * (a + b + c);
    let expected_strength = triad.indices.iter().fold(1.0_f32, |strength, index| {
        strength.min(groups[index.0].map_or(1.0, |group| group.strength))
    });

    assert_eq!(triad.flags, ParticleFlags::ELASTIC);
    assert_eq!(triad.strength.to_bits(), expected_strength.to_bits());
    assert_eq!(triad.pa, a - midpoint);
    assert_eq!(triad.pb, b - midpoint);
    assert_eq!(triad.pc, c - midpoint);
    assert_eq!(triad.ka.to_bits(), (-ca.dot(ab)).to_bits());
    assert_eq!(triad.kb.to_bits(), (-ab.dot(bc)).to_bits());
    assert_eq!(triad.kc.to_bits(), (-bc.dot(ca)).to_bits());
    assert_eq!(
        triad.s.to_bits(),
        (a.cross(b) + b.cross(c) + c.cross(a)).to_bits()
    );
}

#[test]
fn triad_gates_require_elastic_connectable_generators_distance_and_filter_approval() {
    // Arrange
    let square = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 1.0),
    ];
    let no_elastic = Fixture::new(square.clone(), vec![ParticleFlags::WALL; 4]);
    let disconnected = Fixture::new(
        square.clone(),
        vec![
            ParticleFlags::ELASTIC,
            ParticleFlags::WATER,
            ParticleFlags::WATER,
            ParticleFlags::WATER,
        ],
    );
    let mut distant = Fixture::new(square, vec![ParticleFlags::ELASTIC; 4]);
    distant.positions[3] = Vec2::new(10.0, 10.0);
    let veto = TestFilter {
        necessary: vec![true; 4],
        allow_pairs: true,
        allow_triads: false,
    };

    // Act
    let no_elastic_result = generate_pairs_and_triads(&no_elastic.input(), &TestFilter::all(4));
    let disconnected_result = generate_pairs_and_triads(&disconnected.input(), &TestFilter::all(4));
    let distant_result = generate_pairs_and_triads(&distant.input(), &TestFilter::all(4));
    let veto_result = generate_pairs_and_triads(
        &Fixture::new(
            vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 1.0),
            ],
            vec![ParticleFlags::ELASTIC; 4],
        )
        .input(),
        &veto,
    );

    // Assert
    assert!(
        no_elastic_result
            .expect("no elastic work is a no-op")
            .triads
            .is_empty()
    );
    assert!(
        disconnected_result
            .expect("fewer than three connectable generators is a no-op")
            .triads
            .is_empty()
    );
    assert!(
        distant_result
            .expect("oversized candidate should be omitted")
            .triads
            .iter()
            .all(|triad| !triad.indices.contains(&ParticleIndex(3)))
    );
    assert!(
        veto_result
            .expect("filter veto should omit candidates")
            .triads
            .is_empty()
    );
}

#[test]
fn no_necessary_generator_is_the_probe_backed_named_error() {
    // Arrange
    let fixture = Fixture::new(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
        ],
        vec![ParticleFlags::ELASTIC; 3],
    );
    let filter = TestFilter {
        necessary: vec![false; 3],
        allow_pairs: true,
        allow_triads: true,
    };

    // Act
    let result = generate_pairs_and_triads(&fixture.input(), &filter);

    // Assert
    assert_eq!(
        result,
        Err(ConstraintError::VoronoiRequiresNecessaryGenerator)
    );
}

#[test]
fn collinear_degenerate_triad_preserves_finite_source_coefficients() {
    // Arrange
    let fixture = Fixture::new(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
        ],
        vec![ParticleFlags::ELASTIC; 3],
    );
    let indices = [ParticleIndex(0), ParticleIndex(1), ParticleIndex(2)];
    let positions = [
        fixture.positions[0],
        fixture.positions[1],
        fixture.positions[2],
    ];

    // Act
    let triad = build_triad(&fixture.input(), indices, positions, ParticleFlags::ELASTIC)
        .expect("probe preserves finite degenerate triad state");

    // Assert
    assert!(triad.pa.is_valid() && triad.pb.is_valid() && triad.pc.is_valid());
    assert!(triad.ka.is_finite() && triad.kb.is_finite() && triad.kc.is_finite());
    assert_eq!(triad.s.to_bits(), 0.0_f32.to_bits());
}

fn pair(indices: [usize; 2], strength: f32, distance: f32) -> ParticlePair {
    ParticlePair {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::SPRING,
        strength,
        distance,
    }
}

fn triad(indices: [usize; 3], marker: f32) -> ParticleTriad {
    ParticleTriad {
        indices: indices.map(ParticleIndex),
        flags: ParticleFlags::ELASTIC,
        strength: marker,
        pa: Vec2::new(marker, 0.0),
        pb: Vec2::ZERO,
        pc: Vec2::ZERO,
        ka: marker,
        kb: 0.0,
        kc: 0.0,
        s: 0.0,
    }
}

#[test]
fn append_policy_stably_sorts_and_keeps_the_first_exact_duplicate() {
    // Arrange
    let mut pairs = vec![
        pair([2, 1], 0.2, 2.0),
        pair([0, 1], 0.3, 3.0),
        pair([0, 1], 0.9, 9.0),
    ];
    let mut triads = vec![
        triad([2, 1, 0], 2.0),
        triad([0, 1, 2], 3.0),
        triad([0, 1, 2], 9.0),
    ];

    // Act
    apply_pair_policy(&mut pairs, RecordPolicy::Append);
    apply_triad_policy(&mut triads, RecordPolicy::Append);

    // Assert
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0].indices, [ParticleIndex(0), ParticleIndex(1)]);
    assert_eq!(pairs[0].strength.to_bits(), 0.3_f32.to_bits());
    assert_eq!(pairs[0].distance.to_bits(), 3.0_f32.to_bits());
    assert_eq!(triads.len(), 2);
    assert_eq!(
        triads[0].indices,
        [ParticleIndex(0), ParticleIndex(1), ParticleIndex(2)]
    );
    assert_eq!(triads[0].strength.to_bits(), 3.0_f32.to_bits());
    assert_eq!(triads[0].pa.x.to_bits(), 3.0_f32.to_bits());
}

#[test]
fn preserve_policy_leaves_historical_order_and_duplicates_bit_identical() {
    // Arrange
    let mut pairs = vec![pair([2, 1], 0.2, 2.0), pair([0, 1], 0.3, 3.0)];
    let mut triads = vec![triad([2, 1, 0], 2.0), triad([0, 1, 2], 3.0)];
    let expected_pairs = pairs.clone();
    let expected_triads = triads.clone();

    // Act
    apply_pair_policy(&mut pairs, RecordPolicy::Preserve);
    apply_triad_policy(&mut triads, RecordPolicy::Preserve);

    // Assert
    assert_eq!(pairs, expected_pairs);
    assert_eq!(triads, expected_triads);
}
