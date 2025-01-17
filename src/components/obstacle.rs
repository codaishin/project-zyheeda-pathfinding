use super::tile::Tile;
use crate::components::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Tile::asset),
	UseAsset<ColorMaterial>(Obstacle::asset)
)]
pub struct Obstacle;

impl Obstacle {
	const ASSET_PATH: &str = "obstacle.json";

	fn asset() -> UseAsset<ColorMaterial> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}
}
