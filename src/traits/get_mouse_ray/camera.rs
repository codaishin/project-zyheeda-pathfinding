use super::GetMouseRay;
use bevy::prelude::*;

impl GetMouseRay for Camera {
	fn get_mouse_ray(&self, transform: &GlobalTransform, window: &Window) -> Option<Ray3d> {
		window
			.cursor_position()
			.and_then(|cursor| self.viewport_to_world(transform, cursor).ok())
	}
}
