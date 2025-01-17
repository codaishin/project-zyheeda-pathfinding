use crate::traits::get_mouse_ray::MouseRayCaster;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Camera2d)]
pub struct PlayerCamera;

impl MouseRayCaster for PlayerCamera {
	type TMouseRayCaster = Camera;
}
