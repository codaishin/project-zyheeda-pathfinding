use bevy::prelude::*;

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
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
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
}
