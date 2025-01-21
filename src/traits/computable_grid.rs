use bevy::prelude::*;

pub trait ComputableGrid {
	type TIter<'a>: Iterator<Item = Vec2>
	where
		Self: 'a;

	fn grid(&self) -> ComputeGrid;
	fn translations(&self) -> Self::TIter<'_>;
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct ComputeGrid {
	pub width: usize,
	pub height: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct ComputeGridNode {
	pub x: usize,
	pub y: usize,
}

impl ComputeGridNode {
	pub const ZERO: Self = Self::from_translation(Vec2::ZERO);

	pub const fn new(x: usize, y: usize) -> Self {
		Self { x, y }
	}

	pub const fn from_translation(Vec2 { x, y }: Vec2) -> Self {
		Self {
			x: x as usize,
			y: y as usize,
		}
	}
}
