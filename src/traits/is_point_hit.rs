use bevy::prelude::*;
use std::ops::Deref;

pub trait IsPointHit {
	fn is_point_hit(&self, point_position: Relative) -> bool;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Relative(Vec2);

impl Relative {
	#[cfg(test)]
	pub fn new(position: Vec2) -> Self {
		Self(position)
	}

	pub fn position(position: Vec2) -> RelativeBuilder {
		RelativeBuilder(position)
	}
}

impl Deref for Relative {
	type Target = Vec2;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct RelativeBuilder(Vec2);

impl RelativeBuilder {
	pub fn to(self, transform: &Transform) -> Relative {
		let RelativeBuilder(position) = self;
		Relative(position - transform.translation.xy())
	}
}
