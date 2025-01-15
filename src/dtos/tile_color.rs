use crate::traits::load_from::LoadFrom;
use bevy::{asset::LoadContext, prelude::*};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct TileColor {
	color: Color,
}

impl LoadFrom<TileColor> for ColorMaterial {
	fn load_from(TileColor { color }: TileColor, _: &mut LoadContext) -> Self {
		ColorMaterial::from_color(color)
	}
}
