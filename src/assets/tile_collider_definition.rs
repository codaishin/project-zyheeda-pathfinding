use crate::{components::tile_collider::TileCollider, traits::into_component::IntoComponent};
use bevy::prelude::*;

#[derive(Asset, TypePath, Debug, PartialEq)]
pub struct TileColliderDefinition {
	pub half_height: f32,
	pub half_width: f32,
}

impl IntoComponent for Handle<TileColliderDefinition> {
	type TComponent = TileCollider;

	fn into_component(self) -> Self::TComponent {
		TileCollider(self)
	}
}
