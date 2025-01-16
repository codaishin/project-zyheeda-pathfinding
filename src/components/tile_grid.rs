use super::use_asset::UseAsset;
use crate::assets::grid::Grid;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility, UseAsset::<Grid>(TileGrid::grid))]
pub struct TileGrid;

impl TileGrid {
	const ASSET_PATH: &str = "grid.json";

	pub fn spawn(mut commands: Commands) {
		commands.spawn(TileGrid);
	}

	fn grid() -> UseAsset<Grid> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}
}
