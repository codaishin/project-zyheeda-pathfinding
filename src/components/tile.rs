use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub struct Tile;

pub struct Grid {
	pub height: usize,
	pub width: usize,
}

impl Tile {
	pub fn spawn_in(grid: Grid) -> impl Fn(Commands) {
		move |mut commands: Commands| {}
	}
}
