mod camera;

use bevy::prelude::*;

pub trait GetMouseRay {
	fn get_mouse_ray(&self, transform: &GlobalTransform, window: &Window) -> Option<Ray3d>;
}

pub trait MouseRayCaster {
	type TMouseRayCaster: GetMouseRay + Component;
}
