use liquidfun::collision::Aabb;
use liquidfun::math::Vec2;

pub(super) use std::collections::HashSet;

pub(super) fn aabb(lower_x: f32, lower_y: f32, upper_x: f32, upper_y: f32) -> Aabb {
    Aabb::new(Vec2::new(lower_x, lower_y), Vec2::new(upper_x, upper_y))
        .expect("test bounds should be valid")
}
