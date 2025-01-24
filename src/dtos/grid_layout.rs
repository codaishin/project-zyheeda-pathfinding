use crate::{assets::grid::Grid, traits::load_from::LoadFrom};
use bevy::{asset::LoadContext, math::Vec2};
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
			max: Vec2::new(width as f32, height as f32),
			scale,
		}
	}
}
