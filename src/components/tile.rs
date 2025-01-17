use super::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Tile::mesh),
	UseAsset<ColorMaterial>(Tile::color),
)]
pub struct Tile;

impl Tile {
	const ASSET_PATH: &str = "tile.json";

	fn color() -> UseAsset<ColorMaterial> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}

	pub fn mesh() -> UseAsset<Mesh> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}
}
