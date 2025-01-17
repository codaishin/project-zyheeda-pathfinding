use crate::components::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default)]
pub struct Obstacle;

impl Obstacle {
	const ASSET_PATH: &str = "obstacle.json";

	fn asset() -> UseAsset<ColorMaterial> {
		UseAsset::new(Path::new(Self::ASSET_PATH))
	}

	pub fn update_color(
		mut commands: Commands,
		obstacles: Query<(Entity, Option<&UseAsset<ColorMaterial>>), Added<Obstacle>>,
		mut removed_obstacles: RemovedComponents<Obstacle>,
		original_colors: Query<&OriginalColor>,
	) {
		for (entity, color) in &obstacles {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert(Obstacle::asset());

			let Some(color) = color else {
				continue;
			};

			entity.try_insert(OriginalColor(color.clone()));
		}

		for entity in removed_obstacles.read() {
			let Ok(OriginalColor(color)) = original_colors.get(entity) else {
				continue;
			};
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.try_insert(color.clone());
			entity.remove::<OriginalColor>();
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct OriginalColor(UseAsset<ColorMaterial>);

#[cfg(test)]
mod test {
	use super::*;
	use crate::test_tools::SingleThreaded;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Obstacle::update_color);

		app
	}

	#[test]
	fn insert_color_asset() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Obstacle,
				UseAsset::<ColorMaterial>::new(Path::new("some/other")),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Obstacle::asset()),
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}

	#[test]
	fn do_not_insert_obstacle_asset_on_other_entities() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(UseAsset::<ColorMaterial>::new(Path::new("some/other")))
			.id();

		app.update();

		assert_eq!(
			Some(&UseAsset::<ColorMaterial>::new(Path::new("some/other"))),
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}

	#[test]
	fn insert_color_asset_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Obstacle,
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
	fn insert_original_color_asset_when_removed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Obstacle,
				UseAsset::<ColorMaterial>::new(Path::new("some/other")),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Obstacle>();
		app.update();

		assert_eq!(
			Some(&UseAsset::<ColorMaterial>::new(Path::new("some/other"))),
			app.world().entity(entity).get::<UseAsset<ColorMaterial>>(),
		);
	}

	#[test]
	fn cleanup_original_color_helper_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Obstacle,
				UseAsset::<ColorMaterial>::new(Path::new("some/other")),
			))
			.id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<Obstacle>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<OriginalColor>());
	}
}
