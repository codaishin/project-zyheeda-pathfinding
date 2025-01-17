use crate::{
	assets::tile_collider_definition::TileColliderDefinition,
	traits::load_from::LoadFrom,
};
use bevy::{asset::LoadContext, prelude::*};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct TileSize {
	width: f32,
	height: f32,
}

impl LoadFrom<TileSize> for Mesh {
	fn load_from(TileSize { width, height }: TileSize, _: &mut LoadContext) -> Self {
		Mesh::from(Rectangle::new(width, height))
	}
}

impl LoadFrom<TileSize> for TileColliderDefinition {
	fn load_from(TileSize { width, height }: TileSize, _: &mut LoadContext) -> Self {
		TileColliderDefinition {
			half_height: width / 2.,
			half_width: height / 2.,
		}
	}
}
