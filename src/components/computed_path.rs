use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub struct ComputedPath(pub Vec<Vec3>);
