use crate::{assets::grid::Grid, traits::translations::Translations};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub struct Tile;

impl Tile {
	pub fn spawn_in(grid: Grid) -> impl Fn(Commands) {
		move |mut commands: Commands| {
			for translation in grid.translations() {
				commands.spawn((Tile, Transform::from_translation(translation)));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_count, components::tile::Tile};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_1_tile_in_1_by_1_layout() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(Tile::spawn_in(Grid::default()))?;

		let [entity] = assert_count!(1, app.world().iter_entities());
		assert_eq!(Some(&Tile), entity.get::<Tile>());
		Ok(())
	}

	#[test]
	fn spawn_3_tiles_in_1_by_3_layout() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(Tile::spawn_in(Grid {
			width: 3,
			..default()
		}))?;

		let [a, b, c] = assert_count!(3, app.world().iter_entities());
		assert_eq!(
			(Some(&Tile), Some(&Tile), Some(&Tile),),
			(a.get::<Tile>(), b.get::<Tile>(), c.get::<Tile>(),)
		);
		Ok(())
	}

	#[test]
	fn spawn_5_tiles_in_5_by_1_layout() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(Tile::spawn_in(Grid {
			height: 5,
			..default()
		}))?;

		let [a, b, c, d, e] = assert_count!(5, app.world().iter_entities());
		assert_eq!(
			(
				Some(&Tile),
				Some(&Tile),
				Some(&Tile),
				Some(&Tile),
				Some(&Tile),
			),
			(
				a.get::<Tile>(),
				b.get::<Tile>(),
				c.get::<Tile>(),
				d.get::<Tile>(),
				e.get::<Tile>(),
			)
		);
		Ok(())
	}

	#[test]
	fn spawn_tiles_with_proper_transform_in_3_by_3() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(Tile::spawn_in(Grid {
			width: 3,
			height: 3,
			..default()
		}))?;

		let entities = assert_count!(9, app.world().iter_entities());
		assert_eq!(
			[
				Some(Vec3::new(-1., -1., 0.)),
				Some(Vec3::new(-1., 0., 0.)),
				Some(Vec3::new(-1., 1., 0.)),
				Some(Vec3::new(0., -1., 0.)),
				Some(Vec3::new(0., 0., 0.)),
				Some(Vec3::new(0., 1., 0.)),
				Some(Vec3::new(1., -1., 0.)),
				Some(Vec3::new(1., 0., 0.)),
				Some(Vec3::new(1., 1., 0.)),
			],
			entities.map(|e| e.get::<Transform>().map(|t| t.translation))
		);
		Ok(())
	}

	#[test]
	fn spawn_tiles_with_proper_transform_in_2_by_2() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(Tile::spawn_in(Grid {
			width: 2,
			height: 2,
			..default()
		}))?;

		let entities = assert_count!(4, app.world().iter_entities());
		assert_eq!(
			[
				Some(Vec3::new(-0.5, -0.5, 0.)),
				Some(Vec3::new(-0.5, 0.5, 0.)),
				Some(Vec3::new(0.5, -0.5, 0.)),
				Some(Vec3::new(0.5, 0.5, 0.)),
			],
			entities.map(|e| e.get::<Transform>().map(|t| t.translation))
		);
		Ok(())
	}

	#[test]
	fn spawn_tiles_with_proper_transform_in_2_by_2_with_scale_10() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(Tile::spawn_in(Grid {
			width: 2,
			height: 2,
			scale: 10.,
		}))?;

		let entities = assert_count!(4, app.world().iter_entities());
		assert_eq!(
			[
				Some(Vec3::new(-5., -5., 0.)),
				Some(Vec3::new(-5., 5., 0.)),
				Some(Vec3::new(5., -5., 0.)),
				Some(Vec3::new(5., 5., 0.)),
			],
			entities.map(|e| e.get::<Transform>().map(|t| t.translation))
		);
		Ok(())
	}
}
