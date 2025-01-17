use bevy::prelude::*;

pub struct Relative(pub Vec2);

pub trait IsHit {
	fn is_hit(&self, relative_position: Relative) -> bool;
}
