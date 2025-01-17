use crate::{
	assets::tile_collider_definition::TileColliderDefinition,
	traits::asset_handle::AssetHandle,
};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct TileCollider(pub Handle<TileColliderDefinition>);

impl AssetHandle for TileCollider {
	type TAsset = TileColliderDefinition;

	fn get_handle(&self) -> &Handle<Self::TAsset> {
		let Self(handle) = self;
		handle
	}
}
