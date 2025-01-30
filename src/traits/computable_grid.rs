use bevy::prelude::*;
use std::ops::{Add, AddAssign, Sub};

pub trait ComputableGrid {
	type TIter<'a>: Iterator<Item = Vec2>
	where
		Self: 'a;

	fn grid(&self) -> ComputeGrid;
	fn translations(&self) -> Self::TIter<'_>;
}

pub trait GetComputeGridNode {
	fn compute_grid_node(&self, translation: Vec2) -> Option<ComputeGridNode>;
}

pub trait GetTranslation {
	fn translation(&self, node: ComputeGridNode) -> Option<Vec2>;
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct ComputeGrid {
	pub min: ComputeGridNode,
	pub max: ComputeGridNode,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Clone, Copy)]
pub struct ComputeGridNode {
	pub x: i32,
	pub y: i32,
}

impl ComputeGridNode {
	pub const ZERO: Self = Self::from_translation(Vec2::ZERO);

	pub const fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}

	pub const fn from_translation(Vec2 { x, y }: Vec2) -> Self {
		Self {
			x: x as i32,
			y: y as i32,
		}
	}

	pub fn right_angle_len(&self) -> u32 {
		(self.x.abs() + self.y.abs()) as u32
	}

	pub fn is_straight(&self) -> bool {
		(self.x == 0 && self.y != 0) || (self.x != 0 && self.y == 0)
	}

	pub fn is_diagonal(&self) -> bool {
		self.x.abs() == self.y.abs()
	}

	pub fn eight_sided_direction_to(&self, target: &ComputeGridNode) -> Option<ComputeGridNode> {
		if self == target {
			return None;
		}

		let direction = *target - *self;

		if direction.x == 0 && direction.y != 0 {
			return Some(ComputeGridNode {
				x: 0,
				y: unit(direction.y),
			});
		};

		if direction.y == 0 && direction.x != 0 {
			return Some(ComputeGridNode {
				x: unit(direction.x),
				y: 0,
			});
		}

		if direction.x.abs() == direction.y.abs() {
			return Some(ComputeGridNode {
				x: unit(direction.x),
				y: unit(direction.y),
			});
		}

		None
	}
}

fn unit(value: i32) -> i32 {
	if value < 0 {
		-1
	} else {
		1
	}
}

impl Add for ComputeGridNode {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl AddAssign for ComputeGridNode {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl Sub for ComputeGridNode {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}
