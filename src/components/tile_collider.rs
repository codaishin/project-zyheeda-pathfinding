use crate::{assets::collider_definition::ColliderDefinition, traits::asset_handle::AssetHandle};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct TileCollider(pub Handle<ColliderDefinition>);

impl AssetHandle for TileCollider {
	type TAsset = ColliderDefinition;

	fn get_handle(&self) -> &Handle<Self::TAsset> {
		let Self(handle) = self;
		handle
	}
}
