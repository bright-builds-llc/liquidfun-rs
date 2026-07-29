type AuthorityReferences = (&'static [&'static str], &'static [&'static str]);

pub(super) fn references(slug: &str) -> Option<AuthorityReferences> {
    rigid_references(slug)
        .or_else(|| joint_references(slug))
        .or_else(|| particle_references(slug))
}

fn rigid_references(slug: &str) -> Option<AuthorityReferences> {
    let refs = match slug {
        "rigid-non-colliding-lifecycle" => (
            &["testbed.body-types-bodytypes-create"][..],
            &["subsystem.rigid-bodies-and-fixtures"][..],
        ),
        "rigid-contact-lifecycle" => (
            &["testbed.breakable-breakable-create"][..],
            &["subsystem.contacts-and-filtering"][..],
        ),
        "rigid-stack-stability" => (
            &["testbed.vertical-stack-verticalstack-create"][..],
            &["subsystem.rigid-islands-and-solver"][..],
        ),
        "rigid-sleep-and-wake" => (
            &["testbed.body-types-bodytypes-create"][..],
            &["subsystem.rigid-islands-and-solver"][..],
        ),
        "rigid-continuous-collision" => (
            &["testbed.continuous-test-continuoustest-create"][..],
            &["subsystem.world-operations-and-observation"][..],
        ),
        "rigid-collision-filtering" => (
            &["testbed.collision-filtering-collisionfiltering-create"][..],
            &["subsystem.contacts-and-filtering"][..],
        ),
        "rigid-world-queries" => (
            &["testbed.ray-cast-raycast-create"][..],
            &["subsystem.world-operations-and-observation"][..],
        ),
        "rigid-callback-timing" => (
            &["testbed.sensor-test-sensortest-create"][..],
            &["subsystem.contacts-and-filtering"][..],
        ),
        "rigid-runtime-mutation" => (
            &["testbed.shape-editing-shapeediting-create"][..],
            &["subsystem.rigid-bodies-and-fixtures"][..],
        ),
        "rigid-destruction-order" => (
            &["testbed.collision-processing-collisionprocessing-create"][..],
            &["subsystem.world-operations-and-observation"][..],
        ),
        _ => return None,
    };
    Some(refs)
}

fn joint_references(slug: &str) -> Option<AuthorityReferences> {
    let refs = match slug {
        "joint-revolute-behavior" => (
            &["testbed.revolute-revolute-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2revolutejoint-h"][..],
        ),
        "joint-prismatic-behavior" => (
            &["testbed.prismatic-prismatic-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2prismaticjoint-h"][..],
        ),
        "joint-distance-behavior" => (
            &["testbed.distance-test-distancetest-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2distancejoint-h"][..],
        ),
        "joint-pulley-behavior" => (
            &["testbed.pulleys-pulleys-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2pulleyjoint-h"][..],
        ),
        "joint-mouse-behavior" => (
            &["testbed.web-web-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2mousejoint-h"][..],
        ),
        "joint-gear-behavior" => (
            &["testbed.gears-gears-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2gearjoint-h"][..],
        ),
        "joint-wheel-behavior" => (
            &["testbed.car-car-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2wheeljoint-h"][..],
        ),
        "joint-weld-behavior" => (
            &["testbed.cantilever-cantilever-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2weldjoint-h"][..],
        ),
        "joint-friction-behavior" => (
            &["testbed.varying-friction-varyingfriction-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2frictionjoint-h"][..],
        ),
        "joint-rope-behavior" => (
            &["testbed.ropejoint-ropejoint-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2ropejoint-h"][..],
        ),
        "joint-motor-behavior" => (
            &["testbed.motor-joint-motorjoint-create"][..],
            &["public-api.liquidfun-box2d-box2d-dynamics-joints-b2motorjoint-h"][..],
        ),
        "standalone-rope-evolution" => (&["example.rope-create"][..], &["subsystem.rope"][..]),
        _ => return None,
    };
    Some(refs)
}

fn particle_references(slug: &str) -> Option<AuthorityReferences> {
    let refs = match slug {
        "particle-storage-lifecycle" => (
            &["upstream-test.functiontests-particlehandletrackcompactparticles"][..],
            &["subsystem.particle-storage-and-lifecycle"][..],
        ),
        "particle-contacts-and-coupling" => (
            &["upstream-test.bodycontacttests-particlefixturecontactlistener"][..],
            &["subsystem.particle-contacts-and-coupling"][..],
        ),
        "particle-forces-and-statistics" => (
            &["testbed.impulse-impulse-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-system-pause-action" => (
            &["testbed.multiple-systems-multipleparticlesystems-create"][..],
            &["subsystem.particle-storage-and-lifecycle"][..],
        ),
        "particle-flags-water-zombie" => (
            &["testbed.particles-particles-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-flags-wall-barrier" => (
            &["testbed.wave-machine-wavemachine-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-flags-spring-elastic-reactive" => (
            &["testbed.elastic-particles-elasticparticles-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-flags-viscous-powder" => (
            &["testbed.faucet-faucet-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-flags-tensile-color" => (
            &["testbed.surface-tension-particlessurfacetension-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-flags-pressure-repulsive" => (
            &["testbed.dambreak-dambreak-create"][..],
            &["subsystem.particle-solver-behaviors"][..],
        ),
        "particle-flags-contact-listeners" => (
            &["upstream-test.bodycontacttests-particlecontactlistener"][..],
            &["subsystem.particle-contacts-and-coupling"][..],
        ),
        "particle-flags-contact-filters" => (
            &["upstream-test.bodycontacttests-enabledisableparticlecontactswithcontactfilter"][..],
            &["subsystem.particle-contacts-and-coupling"][..],
        ),
        "particle-group-construction-append" => (
            &["testbed.particle-drawing-drawingparticles-create"][..],
            &["subsystem.particle-groups-pairs-and-triads"][..],
        ),
        "particle-group-join" => (
            &["testbed.soup-stirrer-soupstirrer-create"][..],
            &["subsystem.particle-groups-pairs-and-triads"][..],
        ),
        "particle-group-split-reactive" => (
            &["testbed.surface-tension-particlessurfacetension-create"][..],
            &["subsystem.particle-groups-pairs-and-triads"][..],
        ),
        "particle-group-solid-rigid" => (
            &["testbed.rigid-particles-rigidparticles-create"][..],
            &["subsystem.particle-groups-pairs-and-triads"][..],
        ),
        "particle-group-destruction" => (
            &["upstream-test.callbacktests-destroyparticlegroupwithcallback"][..],
            &["subsystem.particle-groups-pairs-and-triads"][..],
        ),
        "particle-aabb-query-controls" => (
            &["upstream-test.callbacktests-querycallback"][..],
            &["subsystem.world-operations-and-observation"][..],
        ),
        "particle-ray-callback-controls" => (
            &["upstream-test.callbacktests-raycastcallback"][..],
            &["subsystem.world-operations-and-observation"][..],
        ),
        "particle-lifecycle-callbacks" => (
            &["upstream-test.callbacktests-destroyparticlewithcallback"][..],
            &["subsystem.particle-storage-and-lifecycle"][..],
        ),
        "particle-mutations" => (
            &["upstream-test.functiontests-particlesystemsetparticlevelocity"][..],
            &["subsystem.particle-storage-and-lifecycle"][..],
        ),
        _ => return None,
    };
    Some(refs)
}
