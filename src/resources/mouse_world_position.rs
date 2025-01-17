use std::ops::DerefMut;

use crate::traits::get_mouse_ray::{GetMouseRay, MouseRayCaster};
use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq, Default)]
pub struct MouseWorldPosition(Option<Vec3>);

impl MouseWorldPosition {
	pub fn update_using<TCamera>(
		mut mouse_position: ResMut<Self>,
		cameras: Query<(&TCamera::TMouseRayCaster, &GlobalTransform), With<TCamera>>,
		windows: Query<&Window>,
	) where
		TCamera: Component + MouseRayCaster,
	{
		let MouseWorldPosition(mouse_position) = mouse_position.deref_mut();
		*mouse_position = get_mouse_position(cameras, windows);
	}
}

fn get_mouse_position<TCamera>(
	cameras: Query<(&TCamera::TMouseRayCaster, &GlobalTransform), With<TCamera>>,
	windows: Query<&Window>,
) -> Option<Vec3>
where
	TCamera: Component + MouseRayCaster,
{
	let (camera, transform) = cameras.get_single().ok()?;
	let window = windows.get_single().ok()?;
	let ray = camera.get_mouse_ray(transform, window)?;
	let toi = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Z))?;
	Some(ray.origin + ray.direction * toi)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{new_mock, traits::get_mouse_ray::GetMouseRay};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::automock;

	const DEFAULT_RAY: Ray3d = Ray3d {
		origin: Vec3::Z,
		direction: Dir3::NEG_Z,
	};

	#[derive(Component)]
	#[require(_Camera)]
	struct _CameraLabel;

	impl MouseRayCaster for _CameraLabel {
		type TMouseRayCaster = _Camera;
	}

	#[derive(Component)]
	#[require(GlobalTransform)]
	struct _Camera {
		mock: Mock_Camera,
	}

	impl Default for _Camera {
		fn default() -> Self {
			Self {
				mock: new_mock!(Mock_Camera, |mock| {
					mock.expect_get_mouse_ray().return_const(Some(DEFAULT_RAY));
				}),
			}
		}
	}

	#[automock]
	impl GetMouseRay for _Camera {
		fn get_mouse_ray(&self, transform: &GlobalTransform, window: &Window) -> Option<Ray3d> {
			self.mock.get_mouse_ray(transform, window)
		}
	}

	fn setup(window_name: &str) -> App {
		let mut app = App::new();
		app.init_resource::<MouseWorldPosition>();
		app.world_mut().spawn(Window {
			title: window_name.to_owned(),
			..default()
		});

		app
	}

	#[test]
	fn set_mouse_position() -> Result<(), RunSystemError> {
		let mut app = setup("");
		app.world_mut().spawn(_CameraLabel);

		app.world_mut()
			.run_system_once(MouseWorldPosition::update_using::<_CameraLabel>)?;

		assert_eq!(
			&MouseWorldPosition(Some(Vec3::ZERO)),
			app.world().resource::<MouseWorldPosition>(),
		);
		Ok(())
	}

	#[test]
	fn use_cam_transform_and_main_window_to_retrieve_mouse_ray() -> Result<(), RunSystemError> {
		let mut app = setup("MY MAIN WINDOW");

		app.world_mut().spawn((
			_CameraLabel,
			_Camera {
				mock: new_mock!(Mock_Camera, |mock| {
					mock.expect_get_mouse_ray()
						.times(1)
						.withf(|transform, window| {
							// `Window` does not impl `PartialEq`, so we check the window title instead
							assert_eq!(
								(&GlobalTransform::from_xyz(1., 2., 3.), "MY MAIN WINDOW"),
								(transform, window.title.as_str())
							);
							true
						})
						.return_const(Some(DEFAULT_RAY));
				}),
			},
			GlobalTransform::from_xyz(1., 2., 3.),
		));

		app.world_mut()
			.run_system_once(MouseWorldPosition::update_using::<_CameraLabel>)?;
		Ok(())
	}

	#[test]
	fn set_mouse_position_none_when_ray_none() -> Result<(), RunSystemError> {
		let mut app = setup("");
		app.world_mut()
			.insert_resource(MouseWorldPosition(Some(Vec3::ZERO)));
		app.world_mut().spawn((
			_CameraLabel,
			_Camera {
				mock: new_mock!(Mock_Camera, |mock| {
					mock.expect_get_mouse_ray().return_const(None);
				}),
			},
		));

		app.world_mut()
			.run_system_once(MouseWorldPosition::update_using::<_CameraLabel>)?;

		assert_eq!(
			&MouseWorldPosition(None),
			app.world().resource::<MouseWorldPosition>(),
		);
		Ok(())
	}

	#[test]
	fn set_mouse_position_none_when_ray_not_intersecting_ground() -> Result<(), RunSystemError> {
		let mut app = setup("");
		app.world_mut()
			.insert_resource(MouseWorldPosition(Some(Vec3::ZERO)));
		app.world_mut().spawn((
			_CameraLabel,
			_Camera {
				mock: new_mock!(Mock_Camera, |mock| {
					mock.expect_get_mouse_ray().return_const(Some(Ray3d {
						origin: Vec3::NEG_Z,
						direction: Dir3::NEG_Z,
					}));
				}),
			},
		));

		app.world_mut()
			.run_system_once(MouseWorldPosition::update_using::<_CameraLabel>)?;

		assert_eq!(
			&MouseWorldPosition(None),
			app.world().resource::<MouseWorldPosition>(),
		);
		Ok(())
	}

	#[test]
	fn set_mouse_position_when_ray_intersecting_ground() -> Result<(), RunSystemError> {
		let mut app = setup("");
		app.world_mut()
			.insert_resource(MouseWorldPosition(Some(Vec3::ZERO)));
		app.world_mut().spawn((
			_CameraLabel,
			_Camera {
				mock: new_mock!(Mock_Camera, |mock| {
					mock.expect_get_mouse_ray().return_const(Some(Ray3d {
						origin: Vec3::new(0., 5., 3.),
						direction: Dir3::new(Vec3::new(0., -2., -3.)).unwrap(),
					}));
				}),
			},
		));

		app.world_mut()
			.run_system_once(MouseWorldPosition::update_using::<_CameraLabel>)?;

		assert_eq!(
			&MouseWorldPosition(Some(Vec3::new(0., 3., 0.))),
			app.world().resource::<MouseWorldPosition>(),
		);
		Ok(())
	}
}
