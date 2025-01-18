use crate::{components::tile_type::TileType, traits::get_key::GetKey};
use bevy::prelude::*;
use std::hash::Hash;

#[derive(States, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum PathPlacement {
	#[default]
	Start,
	End,
}

impl PathPlacement {
	pub fn toggle_with<TKeyDefinition>(
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
			PathPlacement::Start if last_placed.is(TileType::Start) => next.set(PathPlacement::End),
			PathPlacement::End if last_placed.is(TileType::End) => next.set(PathPlacement::Start),
			_ => {}
		};
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct LastPlaced(Option<TileType>);

impl LastPlaced {
	fn is(&self, tile_type: TileType) -> bool {
		self.0.map(|t| t == tile_type).unwrap_or(false)
	}

	fn update(&mut self, changed_tiles: Query<&TileType, Changed<TileType>>) {
		let last_placed = changed_tiles
			.iter()
			.find(|t| t == &&TileType::Start || t == &&TileType::End);

		let Some(last_placed_tile) = last_placed else {
			return;
		};
		*self = LastPlaced(Some(*last_placed_tile));
	}
}

#[cfg(test)]
mod tests {
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
		fn set_tile(&mut self, entity: Entity, tile: TileType) -> &mut Self;
	}

	impl SetTile for App {
		fn set_tile(&mut self, entity: Entity, tile: TileType) -> &mut Self {
			let mut entity = self.world_mut().entity_mut(entity);
			let mut target = entity.get_mut::<TileType>().unwrap();

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
		app.add_systems(Update, PathPlacement::toggle_with::<_Definition>);

		app
	}

	#[test]
	fn toggle_to_place_end_on_released() {
		let mut app = setup(PathPlacement::Start);

		app.world_mut().spawn(TileType::Start);
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
	fn toggle_to_place_start_on_released_when_one_end_tile_present() {
		let mut app = setup(PathPlacement::End);

		app.world_mut().spawn(TileType::End);
		app.release(_Key);
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

		app.world_mut().spawn(TileType::End);
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

		app.set_tile(start, TileType::Start);
		app.update();
		app.set_tile(end, TileType::End);
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

		app.set_tile(start, TileType::Start);
		app.update();
		app.set_tile(obstacle, TileType::Obstacle);
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

		app.world_mut().spawn(TileType::Start);
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
		app.set_tile(end, TileType::End);
		app.update();
		app.set_tile(start, TileType::Start);
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::End,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}

	#[test]
	fn toggle_to_place_start_on_released_when_last_placed_obstacle() {
		let mut app = setup(PathPlacement::End);
		// spawning in reverse order, so we cannot rely on query iteration order
		let [end, obstacle] = [
			app.world_mut().spawn(TileType::default()).id(),
			app.world_mut().spawn(TileType::default()).id(),
		];

		app.set_tile(end, TileType::End);
		app.update();
		app.set_tile(obstacle, TileType::Obstacle);
		app.release(_Key);
		app.update();
		app.update();

		assert_eq!(
			&PathPlacement::Start,
			app.world().resource::<State<PathPlacement>>().get(),
		);
	}
}
