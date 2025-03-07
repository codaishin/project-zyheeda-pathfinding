use crate::{assets::collider_definition::ColliderDefinition, traits::load_from::LoadFrom};
use bevy::{asset::LoadContext, prelude::*};
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct MeshDefinition {
	shape: Shape,
}

#[derive(Debug, PartialEq, Deserialize)]
enum Shape {
	Tile { width: f32, height: f32 },
	Line { width: f32 },
}

impl LoadFrom<MeshDefinition> for Mesh {
	fn load_from(MeshDefinition { shape }: MeshDefinition, _: &mut LoadContext) -> Self {
		match shape {
			Shape::Tile { width, height } => Mesh::from(Rectangle::new(width, height)),
			Shape::Line { width } => Mesh::from(Rectangle::new(width, 1.)),
		}
	}
}

impl LoadFrom<MeshDefinition> for ColliderDefinition {
	fn load_from(MeshDefinition { shape }: MeshDefinition, _: &mut LoadContext) -> Self {
		match shape {
			Shape::Tile { width, height } => ColliderDefinition {
				half_height: width / 2.,
				half_width: height / 2.,
			},
			Shape::Line { width } => ColliderDefinition {
				half_height: width / 2.,
				half_width: 0.5,
			},
		}
	}
}
