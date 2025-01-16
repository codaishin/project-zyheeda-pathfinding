use crate::{assets::grid::Grid, traits::load_from::LoadFrom};
use bevy::asset::LoadContext;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct GridLayout {
	height: usize,
	width: usize,
	scale: f32,
}

impl LoadFrom<GridLayout> for Grid {
	fn load_from(
		GridLayout {
			height,
			width,
			scale,
		}: GridLayout,
		_: &mut LoadContext,
	) -> Self {
		Grid {
			height,
			width,
			scale,
		}
	}
}
