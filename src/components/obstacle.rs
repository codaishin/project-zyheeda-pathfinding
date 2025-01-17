use super::tile::Tile;
use crate::components::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Tile::mesh),
	UseAsset<ColorMaterial>(Obstacle::color)
)]
pub struct Obstacle;

impl Obstacle {
	const ASSET_PATH: &str = "obstacle.json";

	fn color() -> UseAsset<ColorMaterial> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}
}
