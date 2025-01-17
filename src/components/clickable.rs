use crate::{
	resources::mouse_world_position::MouseWorldPosition,
	traits::{
		asset_handle::AssetHandle,
		is_point_hit::{IsPointHit, Relative},
	},
};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Clickable {
	clicked: bool,
}

impl Clickable {
	pub fn update_using<TCollider>(
		mut hovers: Query<(&mut Self, &TCollider, &Transform)>,
		colliders: Res<Assets<TCollider::TAsset>>,
		mouse_world_position: Res<MouseWorldPosition>,
		mouse_input: Res<ButtonInput<MouseButton>>,
	) where
		TCollider: Component + AssetHandle,
		TCollider::TAsset: IsPointHit,
	{
		if !mouse_input.just_pressed(MouseButton::Right) {
			return;
		}

		let MouseWorldPosition(Some(mouse_position)) = *mouse_world_position else {
			return;
		};

		for (mut clickable, collider, transform) in &mut hovers {
			let Some(collider) = colliders.get(collider.get_handle()) else {
				continue;
			};
			let relative_mouse_position = Relative::position(mouse_position).to(transform);
			clickable.clicked = collider.is_point_hit(relative_mouse_position);
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{new_handle, new_mock, test_tools::SingleThreaded};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::{automock, predicate::eq};

	#[derive(Asset, TypePath)]
	struct _ColliderAsset {
		mock: Mock_ColliderAsset,
	}

	impl Default for _ColliderAsset {
		fn default() -> Self {
			Self {
				mock: new_mock!(Mock_ColliderAsset, |mock| {
					mock.expect_is_point_hit().return_const(false);
				}),
			}
		}
	}

	#[automock]
	impl IsPointHit for _ColliderAsset {
		fn is_point_hit(&self, point_position: Relative) -> bool {
			self.mock.is_point_hit(point_position)
		}
	}

	#[derive(Component)]
	#[require(Transform)]
	struct _Collider(Handle<_ColliderAsset>);

	impl AssetHandle for _Collider {
		type TAsset = _ColliderAsset;

		fn get_handle(&self) -> &Handle<Self::TAsset> {
			&self.0
		}
	}

	enum _MouseClick {
		RightJustNot(Option<Vec2>),
		RightHold(Option<Vec2>),
		Nothing,
	}

	fn setup(
		handle: &Handle<_ColliderAsset>,
		collider_asset: _ColliderAsset,
		mouse_click: _MouseClick,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<_ColliderAsset>::default();
		let mut mouse_input = ButtonInput::<MouseButton>::default();
		let mouse_position = MouseWorldPosition(match mouse_click {
			_MouseClick::RightJustNot(mouse_position) => {
				mouse_input.press(MouseButton::Right);
				mouse_position
			}
			_MouseClick::RightHold(mouse_position) => {
				mouse_input.press(MouseButton::Right);
				mouse_input.clear_just_pressed(MouseButton::Right);
				mouse_position
			}
			_MouseClick::Nothing => Some(Vec2::default()),
		});

		assets.insert(handle, collider_asset);
		app.insert_resource(mouse_input);
		app.insert_resource(assets);
		app.insert_resource(mouse_position);

		app
	}

	#[test]
	fn set_to_not_clicked_when_not_hit() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset::default();
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _MouseClick::RightJustNot(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((Clickable { clicked: true }, _Collider(handle)))
			.id();

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)?;

		assert_eq!(
			Some(&Clickable { clicked: false }),
			app.world().entity(entity).get::<Clickable>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_insert_clicked_when_clicked_not_already_present() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset::default();
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _MouseClick::RightJustNot(Some(Vec2::ZERO)));
		let entity = app.world_mut().spawn(_Collider(handle)).id();

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)?;

		assert_eq!(None, app.world().entity(entity).get::<Clickable>());
		Ok(())
	}

	#[test]
	fn set_to_clicked_when_hit() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _MouseClick::RightJustNot(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((Clickable { clicked: false }, _Collider(handle)))
			.id();

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)?;

		assert_eq!(
			Some(&Clickable { clicked: true }),
			app.world().entity(entity).get::<Clickable>(),
		);
		Ok(())
	}

	#[test]
	fn call_hit_check_with_mouse_position() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit()
					.times(1)
					.with(eq(Relative::new(Vec2::new(1., 2.))))
					.return_const(false);
			}),
		};

		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(
			&handle,
			asset,
			_MouseClick::RightJustNot(Some(Vec2::new(1., 2.))),
		);
		app.world_mut()
			.spawn((Clickable { clicked: false }, _Collider(handle)));

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)
	}

	#[test]
	fn call_hit_check_with_relative_with_mouse_position_to_self() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit()
					.times(1)
					.with(eq(Relative::new(Vec2::new(-2., -1.))))
					.return_const(false);
			}),
		};

		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(
			&handle,
			asset,
			_MouseClick::RightJustNot(Some(Vec2::new(1., 2.))),
		);
		app.world_mut().spawn((
			Clickable { clicked: false },
			_Collider(handle),
			Transform::from_xyz(3., 3., 0.),
		));

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)
	}

	#[test]
	fn do_nothing_if_not_mouse_right_clicked() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().never().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _MouseClick::Nothing);
		let entity = app
			.world_mut()
			.spawn((Clickable { clicked: false }, _Collider(handle)))
			.id();

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)?;

		assert_eq!(
			Some(&Clickable { clicked: false }),
			app.world().entity(entity).get::<Clickable>(),
		);
		Ok(())
	}

	#[test]
	fn do_nothing_if_not_mouse_just_right_clicked() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().never().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _MouseClick::RightHold(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((Clickable { clicked: false }, _Collider(handle)))
			.id();

		app.world_mut()
			.run_system_once(Clickable::update_using::<_Collider>)?;

		assert_eq!(
			Some(&Clickable { clicked: false }),
			app.world().entity(entity).get::<Clickable>(),
		);
		Ok(())
	}
}
