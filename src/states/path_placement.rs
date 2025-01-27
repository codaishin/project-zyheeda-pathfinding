use crate::{
	components::{
		clickable::Clickable,
		tile_type::{TileType, TileTypeValue},
	},
	traits::get_key::GetKey,
};
use bevy::prelude::*;
use std::hash::Hash;

#[derive(States, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum PathPlacement {
	#[default]
	Start,
	End,
	Drag(Option<TileTypeValue>),
}

impl PathPlacement {
	pub fn reset_on_release<TKeyDefinition>(
		mut next: ResMut<NextState<Self>>,
		input: Res<ButtonInput<TKeyDefinition::TKey>>,
		tiles: Query<&TileType>,
	) where
		TKeyDefinition: GetKey,
		TKeyDefinition::TKey: Copy + Eq + Hash + Sync + Send + 'static,
	{
		if !input.just_released(TKeyDefinition::get_key()) {
			return;
		}

		let path_markers = tiles
			.iter()
			.filter_map(Self::start_or_end)
			.collect::<Vec<_>>();

		match path_markers.as_slice() {
			[] => next.set(PathPlacement::Start),
			[TileTypeValue::Start] => next.set(PathPlacement::End),
			[TileTypeValue::End] => next.set(PathPlacement::Start),
			[TileTypeValue::Start, TileTypeValue::End] => next.set(PathPlacement::Drag(None)),
			[TileTypeValue::End, TileTypeValue::Start] => next.set(PathPlacement::Drag(None)),
			_ => {}
		};
	}

	pub fn drag_on_hold<TKeyDefinition>(
		mut next: ResMut<NextState<PathPlacement>>,
		state: Res<State<PathPlacement>>,
		input: Res<ButtonInput<TKeyDefinition::TKey>>,
		tiles: Query<(&TileType, &Clickable<TKeyDefinition>)>,
	) where
		TKeyDefinition: GetKey + Sync + Send + 'static,
		TKeyDefinition::TKey: Copy + Eq + Hash + Sync + Send + 'static,
	{
		if !input.pressed(TKeyDefinition::get_key()) {
			return;
		}

		if state.get() != &PathPlacement::Drag(None) {
			return;
		}

		let path_markers = tiles
			.iter()
			.filter_map(Self::clicked_tile)
			.filter_map(Self::start_or_end)
			.collect::<Vec<_>>();

		let [value] = path_markers.as_slice() else {
			return;
		};

		next.set(PathPlacement::Drag(Some(*value)));
	}

	fn start_or_end(tile: &TileType) -> Option<TileTypeValue> {
		let value = tile.value();
		if value != TileTypeValue::Start && value != TileTypeValue::End {
			return None;
		}

		Some(value)
	}

	fn clicked_tile<'a, T>((tile, click): (&'a TileType, &Clickable<T>)) -> Option<&'a TileType>
	where
		T: GetKey,
	{
		if !click.is_clicked() {
			return None;
		}

		Some(tile)
	}
}

#[cfg(test)]
mod test_toggle {
	use super::*;
	use crate::test_tools::SingleThreaded;
	use bevy::state::app::StatesPlugin;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key;

	struct _Definition;

	impl GetKey for _Definition {
		type TKey = _Key;

		fn get_key() -> Self::TKey {
			_Key
		}
	}

	struct _ReleaseKey(bool);

	trait SetInput {
		fn release(&mut self, key: _Key) -> &mut Self;
	}

	impl SetInput for App {
		fn release(&mut self, key: _Key) -> &mut Self {
			let mut input = self.world_mut().resource_mut::<ButtonInput<_Key>>();
			input.press(key);
			input.release(key);

			self
		}
	}

	fn setup(path_placement: PathPlacement) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(ButtonInput::<_Key>::default());
		app.add_plugins(StatesPlugin);
		app.insert_state(path_placement);
		app.init_resource::<ButtonInput<_Key>>();
		app.add_systems(Update, PathPlacement::reset_on_release::<_Definition>);

		// Spawn a non start|end tile to force system to filter properly
		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::Walkable));

		app
	}

	#[test]
	fn toggle_to_place_start_on_released() {
		let mut app = setup(PathPlacement::End);

		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_place_end_on_released() {
		let mut app = setup(PathPlacement::Start);

		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::Start));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::End,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_toggle_to_place_end_when_not_released() {
		let mut app = setup(PathPlacement::Start);

		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_place_start_on_released_when_only_end_present() {
		let mut app = setup(PathPlacement::End);

		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::End));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_drag_none_on_released_when_start_and_end_present() {
		let mut app = setup(PathPlacement::End);

		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::Start));
		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::End));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_drag_none_on_released_when_start_and_end_present_reversed() {
		let mut app = setup(PathPlacement::End);

		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::End));
		app.world_mut()
			.spawn(TileType::from_value(TileTypeValue::Start));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}
}

#[cfg(test)]
mod test_drag {
	use super::*;
	use crate::{components::clickable::Clickable, test_tools::SingleThreaded};
	use bevy::state::app::StatesPlugin;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key;

	struct _Definition;

	impl GetKey for _Definition {
		type TKey = _Key;

		fn get_key() -> Self::TKey {
			_Key
		}
	}

	trait SetInput {
		fn press(&mut self, key: _Key) -> &mut Self;
	}

	impl SetInput for App {
		fn press(&mut self, key: _Key) -> &mut Self {
			let mut input = self.world_mut().resource_mut::<ButtonInput<_Key>>();
			input.press(key);

			self
		}
	}

	fn setup(path_placement: PathPlacement) -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<_Key>>();
		app.add_plugins(StatesPlugin);
		app.insert_state(path_placement);
		app.add_systems(Update, PathPlacement::drag_on_hold::<_Definition>);

		app
	}

	#[test]
	fn set_to_drag_start() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Clickable::<_Definition>::new(true),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(Some(TileTypeValue::Start)),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn set_to_drag_end() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::End),
			Clickable::<_Definition>::new(true),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(Some(TileTypeValue::End)),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_set_to_drag_obstacle() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Obstacle),
			Clickable::<_Definition>::new(true),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_set_to_drag_walkable() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Walkable),
			Clickable::<_Definition>::new(true),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_set_to_drag_start_when_not_clicked() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Clickable::<_Definition>::new(false),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn set_to_drag_start_when_multiple_tiles_present() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::End),
			Clickable::<_Definition>::new(false),
		));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Clickable::<_Definition>::new(true),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(Some(TileTypeValue::Start)),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_set_to_drag_start_when_not_set_to_drag() {
		let mut app = setup(PathPlacement::Start);
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Clickable::<_Definition>::new(true),
		));

		app.press(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_set_when_not_pressing() {
		let mut app = setup(PathPlacement::Drag(None));
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Clickable::<_Definition>::new(true),
		));

		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}
}
