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

pub struct Grid {
	pub height: usize,
	pub width: usize,
	pub scale: f32,
}

impl Grid {
	fn translations(&self) -> GridTranslations {
		GridTranslations {
			grid: self,
			width: 1.,
			height: 1.,
			offset: -Vec3::new(
				(self.width + 1) as f32 / 2.,
				(self.height + 1) as f32 / 2.,
				0.,
			),
		}
	}
}

impl Default for Grid {
	fn default() -> Self {
		Self {
			height: 1,
			width: 1,
			scale: 1.,
		}
	}
}

struct GridTranslations<'a> {
	width: f32,
	height: f32,
	offset: Vec3,
	grid: &'a Grid,
}

impl GridTranslations<'_> {
	fn out_of_bounds(&self) -> bool {
		self.width > self.grid.width as f32
	}

	fn iterate(&mut self) {
		self.height += 1.;

		if self.height <= self.grid.height as f32 {
			return;
		}

		self.width += 1.;
		self.height = 1.;
	}
}

impl Iterator for GridTranslations<'_> {
	type Item = Vec3;

	fn next(&mut self) -> Option<Self::Item> {
		if self.out_of_bounds() {
			return None;
		}

		let translation = Vec3::new(self.width, self.height, 0.);

		self.iterate();

		Some((translation + self.offset) * self.grid.scale)
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
