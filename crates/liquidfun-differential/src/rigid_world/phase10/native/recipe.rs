//! Strict protocol-to-public-recipe conversion.

use liquidfun::collision::Shape;
use liquidfun::collision::shape::{ChainShape, CircleShape, EdgeShape, PolygonShape};
use liquidfun::math::Transform;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{ParticleColor, ParticleFlags};
use liquidfun_test_protocol::{Phase10GroupDefinition, Phase10GroupSource, Phase10Shape};

use crate::rigid_world::model::vec2;

pub(super) fn recipe(
    definition: &Phase10GroupDefinition,
    destination: ParticleGroupDestination,
) -> Result<ParticleGroupRecipe<()>, String> {
    let source = match &definition.source {
        Phase10GroupSource::Filled { shapes } => ParticleGroupSource::filled_shapes(
            shapes.iter().map(shape).collect::<Result<Vec<_>, _>>()?,
        ),
        Phase10GroupSource::Stroke { shape: source } => {
            ParticleGroupSource::stroke_shape(shape(source)?)
        }
        Phase10GroupSource::Explicit { positions } => {
            ParticleGroupSource::positions(positions.iter().copied().map(vec2).collect())
        }
    }
    .map_err(|error| error.to_string())?;
    let mut recipe = ParticleGroupRecipe::new(source, destination)
        .with_particle_flags(ParticleFlags::from_bits_retain(
            definition.particle_flags_bits,
        ))
        .with_group_flags(ParticleGroupFlags::from_bits_retain(
            definition.group_flags_bits,
        ))
        .with_transform(Transform::from_position_angle(
            vec2(definition.transform.position),
            definition.transform.angle_bits.to_f32(),
        ))
        .and_then(|recipe| recipe.with_linear_velocity(vec2(definition.linear_velocity)))
        .and_then(|recipe| recipe.with_angular_velocity(definition.angular_velocity_bits.to_f32()))
        .and_then(|recipe| recipe.with_strength(definition.strength_bits.to_f32()))
        .and_then(|recipe| recipe.with_lifetime(definition.lifetime_bits.to_f32()))
        .map_err(|error| error.to_string())?
        .with_color(ParticleColor::new(
            definition.color[0],
            definition.color[1],
            definition.color[2],
            definition.color[3],
        ));
    if let Some(stride) = definition.maybe_stride_bits {
        recipe = recipe
            .with_stride(stride.to_f32())
            .map_err(|error| error.to_string())?;
    }
    Ok(recipe)
}

fn shape(source: &Phase10Shape) -> Result<Shape, String> {
    match source {
        Phase10Shape::Circle {
            center,
            radius_bits,
        } => CircleShape::new(vec2(*center), radius_bits.to_f32()).map(Shape::from),
        Phase10Shape::Polygon { vertices } => {
            let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
            PolygonShape::new(&vertices).map(Shape::from)
        }
        Phase10Shape::Edge { vertex_a, vertex_b } => {
            EdgeShape::new(vec2(*vertex_a), vec2(*vertex_b)).map(Shape::from)
        }
        Phase10Shape::Chain { vertices, looped } => {
            let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
            if *looped {
                ChainShape::closed(&vertices)
            } else {
                ChainShape::open(&vertices, None, None)
            }
            .map(Shape::from)
        }
    }
    .map_err(|error| error.to_string())
}
