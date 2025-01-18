use super::tile::Tile;
use crate::components::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default, Clone, Copy)]
pub enum TileType {
	#[default]
	Walkable,
	Obstacle,
	Start,
	End,
}

impl From<TileType> for UseAsset<ColorMaterial> {
	fn from(value: TileType) -> Self {
		match value {
			TileType::Walkable => Tile::asset(),
			TileType::Obstacle => UseAsset::new(Path::new("obstacle.json")),
			TileType::Start => UseAsset::new(Path::new("start.json")),
			TileType::End => UseAsset::new(Path::new("end.json")),
		}
	}
}

impl TileType {
	pub fn update_color(mut commands: Commands, obstacles: Query<(Entity, &Self), Changed<Self>>) {
		for (entity, tile_type) in &obstacles {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert(UseAsset::from(*tile_type));
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::test_tools::SingleThreaded;
	use std::ops::DerefMut;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, TileType::update_color);

		app
	}

	#[test]
	fn insert_default_color() {
		let mut app = setup();
		let entity = app.world_mut().spawn(TileType::Walkable).id();

		app.update();

		assert_eq!(
			Some(&UseAsset::from(TileType::Walkable)),
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}

	#[test]
	fn insert_obstacle_color() {
		let mut app = setup();
		let entity = app.world_mut().spawn(TileType::Obstacle).id();

		app.update();

		assert_eq!(
			Some(&UseAsset::from(TileType::Obstacle)),
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}

	#[test]
	fn insert_color_asset_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				TileType::Obstacle,
				UseAsset::<ColorMaterial>::new(Path::new("some/other")),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<UseAsset<ColorMaterial>>();
		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}

	#[test]
	fn insert_color_asset_again_after_mut_deref() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				TileType::Obstacle,
				UseAsset::<ColorMaterial>::new(Path::new("some/other")),
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<TileType>()
			.unwrap()
			.deref_mut();
		app.world_mut()
			.entity_mut(entity)
			.remove::<UseAsset<ColorMaterial>>();
		app.update();

		assert_eq!(
			Some(&UseAsset::from(TileType::Obstacle)),
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}
}
