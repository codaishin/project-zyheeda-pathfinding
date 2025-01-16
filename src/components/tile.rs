use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Transform, Visibility)]
pub struct Tile;
