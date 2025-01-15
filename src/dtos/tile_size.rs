use crate::traits::load_from::LoadFrom;
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
