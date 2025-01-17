use super::{clickable::Clickable, use_asset::UseAsset};
use crate::assets::tile_collider_definition::TileColliderDefinition;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Tile::asset),
	UseAsset<ColorMaterial>(Tile::asset),
	UseAsset<TileColliderDefinition>(Tile::asset),
	Clickable,
)]
pub struct Tile;

impl Tile {
	const ASSET_PATH: &str = "tile.json";

	pub fn asset<TAsset>() -> UseAsset<TAsset>
	where
		TAsset: Asset,
	{
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}
}
