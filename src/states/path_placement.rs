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
		mut last_placed: Local<LastPlaced>,
		input: Res<ButtonInput<TKeyDefinition::TKey>>,
		current: Res<State<Self>>,
		changed_tiles: Query<&TileType, Changed<TileType>>,
	) where
		TKeyDefinition: GetKey,
		TKeyDefinition::TKey: Copy + Eq + Hash + Sync + Send + 'static,
	{
		last_placed.update(changed_tiles);

		if !input.just_released(TKeyDefinition::get_key()) {
			return;
		}

		match current.get() {
			PathPlacement::Start if last_placed.is(TileTypeValue::Start) => {
				next.set(PathPlacement::End)
			}
			PathPlacement::End if last_placed.is(TileTypeValue::End) => {
				next.set(PathPlacement::Drag(None))
			}
			PathPlacement::Drag(Some(_)) => next.set(PathPlacement::Drag(None)),
			_ => {}
		};
	}

	pub fn drag_on_hold<TKeyDefinition>(
		mut next: ResMut<NextState<PathPlacement>>,
		state: Res<State<PathPlacement>>,
		tiles: Query<(&TileType, &Clickable<TKeyDefinition>)>,
	) where
		TKeyDefinition: GetKey + Sync + Send + 'static,
		TKeyDefinition::TKey: Copy + Eq + Hash + Sync + Send + 'static,
	{
		if state.get() != &PathPlacement::Drag(None) {
			return;
		}

		let Some((tile, ..)) = tiles.iter().find(|(.., c)| c.is_clicked()) else {
			return;
		};

		let (TileTypeValue::Start | TileTypeValue::End) = tile.value() else {
			return;
		};

		next.set(PathPlacement::Drag(Some(tile.value())));
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct LastPlaced(Option<TileType>);

impl LastPlaced {
	fn is(&self, value: TileTypeValue) -> bool {
		self.0.map(|t| *t == value).unwrap_or(false)
	}

	fn update(&mut self, changed_tiles: Query<&TileType, Changed<TileType>>) {
		let last_placed = changed_tiles
			.iter()
			.find(|t| ***t == TileTypeValue::Start || ***t == TileTypeValue::End);

		let Some(last_placed_tile) = last_placed else {
			return;
		};
		*self = LastPlaced(Some(*last_placed_tile));
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

	trait SetTile {
		fn set<TComponent>(&mut self, entity: Entity, tile: TComponent) -> &mut Self
		where
			TComponent: Component;
	}

	impl SetTile for App {
		fn set<TComponent>(&mut self, entity: Entity, tile: TComponent) -> &mut Self
		where
			TComponent: Component,
		{
			let mut entity = self.world_mut().entity_mut(entity);
			let mut target = entity.get_mut::<TComponent>().unwrap();

			*target = tile;

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

		app
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
	fn do_not_toggle_to_place_end_on_released_when_only_end_present() {
		let mut app = setup(PathPlacement::Start);

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
	fn do_not_toggle_to_place_end_on_released_when_last_placed_end() {
		let mut app = setup(PathPlacement::Start);
		// spawning in reverse order, so we cannot rely on query iteration order
		let [end, start] = [
			app.world_mut().spawn(TileType::default()).id(),
			app.world_mut().spawn(TileType::default()).id(),
		];

		app.set(start, TileType::from_value(TileTypeValue::Start));
		app.update();
		app.set(end, TileType::from_value(TileTypeValue::End));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_place_end_on_released_when_last_placed_obstacle() {
		let mut app = setup(PathPlacement::Start);
		// spawning in reverse order, so we cannot rely on query iteration order
		let [start, obstacle] = [
			app.world_mut().spawn(TileType::default()).id(),
			app.world_mut().spawn(TileType::default()).id(),
		];

		app.set(start, TileType::from_value(TileTypeValue::Start));
		app.update();
		app.set(obstacle, TileType::from_value(TileTypeValue::Obstacle));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::End,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn do_not_toggle_to_place_start_on_released_when_only_start_present() {
		let mut app = setup(PathPlacement::End);

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
	fn do_not_toggle_to_place_start_on_released_when_last_placed_start() {
		let mut app = setup(PathPlacement::End);
		// spawning in reverse order, so we cannot rely on query iteration order
		let [start, end] = [
			app.world_mut().spawn(TileType::default()).id(),
			app.world_mut().spawn(TileType::default()).id(),
		];

		app.update();
		app.set(end, TileType::from_value(TileTypeValue::End));
		app.update();
		app.set(start, TileType::from_value(TileTypeValue::Start));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::End,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_drag_on_released_when_last_placed_obstacle() {
		let mut app = setup(PathPlacement::End);
		// using additionally obstacle, so we need specific filters
		let [start, end, obstacle] = [
			app.world_mut().spawn(TileType::default()).id(),
			app.world_mut().spawn(TileType::default()).id(),
			app.world_mut().spawn(TileType::default()).id(),
		];

		app.set(start, TileType::from_value(TileTypeValue::Start));
		app.update();
		app.set(end, TileType::from_value(TileTypeValue::End));
		app.update();
		app.set(obstacle, TileType::from_value(TileTypeValue::Obstacle));
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_drag_none_on_released_from_dragging_start() {
		let mut app = setup(PathPlacement::Drag(Some(TileTypeValue::Start)));

		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Drag(None),
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_drag_none_on_released_from_dragging_end() {
		let mut app = setup(PathPlacement::Drag(Some(TileTypeValue::End)));

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

	struct _ReleaseKey(bool);

	fn setup(path_placement: PathPlacement) -> App {
		let mut app = App::new().single_threaded(Update);

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

		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}
}
