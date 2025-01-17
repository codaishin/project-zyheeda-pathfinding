use crate::assets::tile_collider_definition::TileColliderDefinition;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct TileCollider(pub Handle<TileColliderDefinition>);
