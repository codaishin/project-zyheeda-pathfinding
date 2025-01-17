use super::use_asset::UseAsset;
use crate::assets::grid::Grid;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Transform, Visibility, UseAsset<Grid>(TileGrid::asset))]
pub struct TileGrid;

impl TileGrid {
	const ASSET_PATH: &str = "grid.json";

	fn asset() -> UseAsset<Grid> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}
}
