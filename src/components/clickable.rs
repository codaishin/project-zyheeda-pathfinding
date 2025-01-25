use crate::{
	resources::mouse_world_position::MouseWorldPosition,
	traits::{
		asset_handle::AssetHandle,
		get_key::GetKey,
		is_point_hit::{IsPointHit, Relative},
		set_value::SetValue,
	},
};
use bevy::prelude::*;
use std::{hash::Hash, marker::PhantomData, ops::Deref};

#[derive(Component, Debug, PartialEq, Default)]
pub struct Clickable<TKeyDefinition>
where
	TKeyDefinition: GetKey,
{
	clicked: bool,
	_p: PhantomData<TKeyDefinition>,
}

impl<TKeyDefinition> Clickable<TKeyDefinition>
where
	TKeyDefinition: GetKey + Sync + Send + 'static,
	TKeyDefinition::TKey: Copy + Eq + Hash + Send + Sync + 'static,
{
	#[cfg(test)]
	pub fn new(clicked: bool) -> Self {
		Self {
			clicked,
			_p: PhantomData,
		}
	}

	pub fn is_clicked(&self) -> bool {
		self.clicked
	}

	fn set_clicked<TCollider>(
		mut clickable: Mut<Clickable<TKeyDefinition>>,
		collider: &TCollider,
		transform: &Transform,
		colliders: &Res<Assets<TCollider::TAsset>>,
		mouse_position: Vec2,
	) where
		TCollider: Component + AssetHandle,
		TCollider::TAsset: IsPointHit,
	{
		let Some(collider) = colliders.get(collider.get_handle()) else {
			return;
		};

		let relative_mouse_position = Relative::position(mouse_position).to(transform);

		if clickable.clicked == collider.is_point_hit(relative_mouse_position) {
			return;
		};

		clickable.clicked = !clickable.clicked;
	}

	fn set_not_clicked(mut clickable: Mut<Clickable<TKeyDefinition>>) {
		if !clickable.clicked {
			return;
		}

		clickable.clicked = false;
	}

	pub fn detect_click_on<TCollider>(
		mut entities: Query<(&mut Self, &TCollider, &Transform)>,
		colliders: Res<Assets<TCollider::TAsset>>,
		mouse_world_position: Res<MouseWorldPosition>,
		input: Res<ButtonInput<TKeyDefinition::TKey>>,
	) where
		TCollider: Component + AssetHandle,
		TCollider::TAsset: IsPointHit,
	{
		let pressed = input.pressed(TKeyDefinition::get_key());

		let MouseWorldPosition(Some(position)) = *mouse_world_position else {
			return;
		};

		for (clickable, collider, transform) in &mut entities {
			match pressed {
				false => Self::set_not_clicked(clickable),
				true => Self::set_clicked(clickable, collider, transform, &colliders, position),
			}
		}
	}

	pub fn toggle<TComponent>(
		toggle_on: TComponent::TValue,
	) -> impl Fn(Query<(&Self, &mut TComponent), Changed<Self>>)
	where
		TComponent:
			SetValue + Default + Deref<Target = TComponent::TValue> + Component + PartialEq + Copy,
		TComponent::TValue: PartialEq + Copy,
	{
		move |mut toggles| {
			for (Self { clicked, .. }, mut toggle) in &mut toggles {
				if !clicked {
					continue;
				}

				match **toggle == toggle_on {
					true => *toggle = TComponent::default(),
					false => toggle.set_value(toggle_on),
				}
			}
		}
	}

	fn just_clicked(clickable: &Ref<Self>) -> bool {
		clickable.is_changed() && clickable.clicked
	}

	fn only_others_clicked(clickable: &Ref<Self>, any_clicked: bool) -> bool {
		any_clicked && !clickable.clicked
	}

	pub fn switch_on_single<TComponent>(
		switch_on_state: TComponent::TValue,
	) -> impl Fn(Query<(Ref<Self>, &mut TComponent)>)
	where
		TComponent: SetValue + Default + Component + Deref<Target = TComponent::TValue>,
		TComponent::TValue: PartialEq + Copy,
	{
		let switched_on = move |switch: &TComponent| **switch == switch_on_state;

		move |mut switches| {
			let any_clicked = switches.iter().any(|(clickable, _)| clickable.clicked);

			for (clickable, mut switch) in &mut switches {
				if Self::just_clicked(&clickable) {
					switch.set_value(switch_on_state);
				}

				if Self::only_others_clicked(&clickable, any_clicked) && switched_on(&switch) {
					*switch = TComponent::default();
				}
			}
		}
	}
}

#[derive(Debug, PartialEq, Default)]
pub struct MouseLeft;

impl GetKey for MouseLeft {
	type TKey = MouseButton;

	fn get_key() -> Self::TKey {
		const { MouseButton::Left }
	}
}

#[derive(Debug, PartialEq, Default)]
pub struct MouseRight;

impl GetKey for MouseRight {
	type TKey = MouseButton;

	fn get_key() -> Self::TKey {
		const { MouseButton::Right }
	}
}

#[cfg(test)]
mod test_update {
	use super::*;
	use crate::{new_handle, new_mock, test_tools::SingleThreaded};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::{automock, predicate::eq};

	#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
	struct _DeviceKey;

	#[derive(Debug, PartialEq, Eq, Default)]
	struct _Button;

	impl GetKey for _Button {
		type TKey = _DeviceKey;

		fn get_key() -> Self::TKey {
			_DeviceKey
		}
	}

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

	enum _Device {
		Pressed(Option<Vec2>),
		Held(Option<Vec2>),
		Released,
	}

	fn setup(
		handle: &Handle<_ColliderAsset>,
		collider_asset: _ColliderAsset,
		mouse_click: _Device,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<_ColliderAsset>::default();
		let mut mouse_input = ButtonInput::<_DeviceKey>::default();
		let mouse_position = MouseWorldPosition(match mouse_click {
			_Device::Pressed(mouse_position) => {
				mouse_input.press(_DeviceKey);
				mouse_position
			}
			_Device::Held(mouse_position) => {
				mouse_input.press(_DeviceKey);
				mouse_input.clear_just_pressed(_DeviceKey);
				mouse_position
			}
			_Device::Released => Some(Vec2::default()),
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
		let mut app = setup(&handle, asset, _Device::Pressed(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Collider(handle),
			))
			.id();

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)?;

		assert_eq!(
			Some(&Clickable::<_Button> {
				clicked: false,
				..default()
			}),
			app.world().entity(entity).get::<Clickable<_Button>>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_insert_clicked_when_clicked_not_already_present() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset::default();
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _Device::Pressed(Some(Vec2::ZERO)));
		let entity = app.world_mut().spawn(_Collider(handle)).id();

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)?;

		assert_eq!(None, app.world().entity(entity).get::<Clickable<_Button>>());
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
		let mut app = setup(&handle, asset, _Device::Pressed(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
				_Collider(handle),
			))
			.id();

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)?;

		assert_eq!(
			Some(&Clickable::<_Button> {
				clicked: true,
				..default()
			}),
			app.world().entity(entity).get::<Clickable<_Button>>(),
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
		let mut app = setup(&handle, asset, _Device::Pressed(Some(Vec2::new(1., 2.))));
		app.world_mut().spawn((
			Clickable::<_Button> {
				clicked: false,
				..default()
			},
			_Collider(handle),
		));

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)
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
		let mut app = setup(&handle, asset, _Device::Pressed(Some(Vec2::new(1., 2.))));
		app.world_mut().spawn((
			Clickable::<_Button> {
				clicked: false,
				..default()
			},
			_Collider(handle),
			Transform::from_xyz(3., 3., 0.),
		));

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)
	}

	#[test]
	fn set_not_clicked_when_released() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().never().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _Device::Released);
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Collider(handle),
			))
			.id();

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)?;

		assert_eq!(
			Some(&Clickable::<_Button> {
				clicked: false,
				..default()
			}),
			app.world().entity(entity).get::<Clickable<_Button>>(),
		);
		Ok(())
	}

	#[test]
	fn also_react_to_longer_mouse_hold() -> Result<(), RunSystemError> {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _Device::Held(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
				_Collider(handle),
			))
			.id();

		app.world_mut()
			.run_system_once(Clickable::<_Button>::detect_click_on::<_Collider>)?;

		assert_eq!(
			Some(&Clickable::<_Button> {
				clicked: true,
				..default()
			}),
			app.world().entity(entity).get::<Clickable<_Button>>(),
		);
		Ok(())
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Changed(bool);

	impl _Changed {
		fn detect(mut commands: Commands, entities: Query<(Entity, Ref<Clickable<_Button>>)>) {
			for (entity, clickable) in &entities {
				let mut entity = commands.entity(entity);
				entity.insert(_Changed(clickable.is_changed()));
			}
		}
	}

	#[test]
	fn do_not_mut_deref_clickable_when_nothing_changed_on_hold() {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _Device::Held(Some(Vec2::ZERO)));
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Collider(handle),
			))
			.id();

		app.add_systems(
			Update,
			(
				Clickable::<_Button>::detect_click_on::<_Collider>,
				_Changed::detect,
			)
				.chain(),
		);
		app.update();
		app.update();

		assert_eq!(
			Some(&_Changed(false)),
			app.world().entity(entity).get::<_Changed>(),
		);
	}

	#[test]
	fn do_not_mut_deref_clickable_when_nothing_changed_on_released() {
		let asset = _ColliderAsset {
			mock: new_mock!(Mock_ColliderAsset, |mock| {
				mock.expect_is_point_hit().return_const(true);
			}),
		};
		let handle = new_handle!(_ColliderAsset);
		let mut app = setup(&handle, asset, _Device::Released);
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
				_Collider(handle),
			))
			.id();

		app.add_systems(
			Update,
			(
				Clickable::<_Button>::detect_click_on::<_Collider>,
				_Changed::detect,
			)
				.chain(),
		);
		app.update();
		app.update();

		assert_eq!(
			Some(&_Changed(false)),
			app.world().entity(entity).get::<_Changed>(),
		);
	}
}

#[cfg(test)]
mod test_toggle {
	use super::*;
	use crate::test_tools::SingleThreaded;
	use std::ops::DerefMut;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _DeviceKey;

	#[derive(Debug, PartialEq, Default)]
	struct _Button;

	impl GetKey for _Button {
		type TKey = _DeviceKey;

		fn get_key() -> Self::TKey {
			_DeviceKey
		}
	}

	#[derive(Component, Debug, PartialEq, Default, Clone, Copy)]
	enum _Component {
		#[default]
		ToggleOff,
		ToggleOn,
	}

	impl SetValue for _Component {
		type TValue = _Component;

		fn set_value(&mut self, value: Self::TValue) {
			*self = value
		}
	}

	impl Deref for _Component {
		type Target = _Component;

		fn deref(&self) -> &Self::Target {
			self
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			Clickable::<_Button>::toggle::<_Component>(_Component::ToggleOn),
		);

		app
	}

	#[test]
	fn toggle_on_when_clicked() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Component::ToggleOff,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component::ToggleOn),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn toggle_on_when_clicked_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Component::ToggleOff,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Component::ToggleOff);
		app.update();

		assert_eq!(
			Some(&_Component::ToggleOff),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn toggle_on_when_clicked_again_after_mut_deref() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Component::ToggleOff,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Component::ToggleOff);
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Clickable<_Button>>()
			.unwrap()
			.deref_mut();
		app.update();

		assert_eq!(
			Some(&_Component::ToggleOn),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn do_nothing_when_not_clicked() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
				_Component::ToggleOff,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component::ToggleOff),
			app.world().entity(entity).get::<_Component>()
		);
	}

	#[test]
	fn toggle_off_when_clicked() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
				_Component::ToggleOn,
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component::ToggleOff),
			app.world().entity(entity).get::<_Component>()
		);
	}
}

#[cfg(test)]
mod test_switch_on_single {
	use super::*;
	use crate::test_tools::SingleThreaded;
	use std::ops::DerefMut;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _DeviceKey;

	#[derive(Debug, PartialEq, Default)]
	struct _Button;

	impl GetKey for _Button {
		type TKey = _DeviceKey;

		fn get_key() -> Self::TKey {
			_DeviceKey
		}
	}

	#[derive(Component, Debug, PartialEq, Default, Clone, Copy)]
	enum _Component {
		#[default]
		SwitchedOff,
		SwitchedOn,
		OtherState,
	}

	impl SetValue for _Component {
		type TValue = _Component;

		fn set_value(&mut self, value: Self::TValue) {
			*self = value;
		}
	}

	impl Deref for _Component {
		type Target = _Component;

		fn deref(&self) -> &Self::Target {
			self
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			Clickable::<_Button>::switch_on_single::<_Component>(_Component::SwitchedOn),
		);

		app
	}

	#[test]
	fn switch_component_on() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::SwitchedOff,
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component::SwitchedOn),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn switch_component_off_if_new_switched_on() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::SwitchedOn,
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
			))
			.id();
		app.world_mut().spawn((
			_Component::SwitchedOff,
			Clickable::<_Button> {
				clicked: true,
				..default()
			},
		));

		app.update();

		assert_eq!(
			Some(&_Component::SwitchedOff),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn do_not_switch_component_off_if_not_on_and_new_switched_on() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::OtherState,
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
			))
			.id();
		app.world_mut().spawn((
			_Component::SwitchedOff,
			Clickable::<_Button> {
				clicked: true,
				..default()
			},
		));

		app.update();

		assert_eq!(
			Some(&_Component::OtherState),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn do_not_switch_component_off_if_no_other_switched_on() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::SwitchedOn,
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Component::SwitchedOn),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn switch_component_on_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::SwitchedOff,
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Component::SwitchedOff);
		app.update();

		assert_eq!(
			Some(&_Component::SwitchedOff),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn switch_component_on_again_after_mut_deref() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::SwitchedOff,
				Clickable::<_Button> {
					clicked: true,
					..default()
				},
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<Clickable<_Button>>()
			.unwrap()
			.deref_mut();
		app.world_mut()
			.entity_mut(entity)
			.insert(_Component::SwitchedOff);
		app.update();

		assert_eq!(
			Some(&_Component::SwitchedOn),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn switch_component_off_if_new_switched_on_in_later_frame() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				_Component::SwitchedOn,
				Clickable::<_Button> {
					clicked: false,
					..default()
				},
			))
			.id();

		app.update();

		app.world_mut().spawn((
			_Component::SwitchedOff,
			Clickable::<_Button> {
				clicked: true,
				..default()
			},
		));

		app.update();

		assert_eq!(
			Some(&_Component::SwitchedOff),
			app.world().entity(entity).get::<_Component>(),
		);
	}
}
