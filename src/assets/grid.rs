use crate::{
	components::tile_builder::TileBuilder,
	traits::{into_component::IntoComponent, translations::Translations},
};
use bevy::prelude::*;

#[derive(Asset, TypePath, Debug, PartialEq)]
pub struct Grid {
	pub height: usize,
	pub width: usize,
	pub scale: f32,
}

impl Grid {
	const DEFAULT: Grid = Grid {
		height: 1,
		width: 1,
		scale: 1.,
	};
}

impl Default for Grid {
	fn default() -> Self {
		Self::DEFAULT
	}
}

impl IntoComponent for Handle<Grid> {
	type TComponent = TileBuilder;

	fn into_component(self) -> Self::TComponent {
		TileBuilder(self)
	}
}

impl Translations for Grid {
	type TIter<'a> = GridTranslations<'a>;

	fn translations(&self) -> Self::TIter<'_> {
		GridTranslations {
			grid: self,
			width: 1,
			height: 1,
			offset: -Vec3::new(
				(self.width + 1) as f32 / 2.,
				(self.height + 1) as f32 / 2.,
				0.,
			),
		}
	}
}

pub struct GridTranslations<'a> {
	width: usize,
	height: usize,
	offset: Vec3,
	grid: &'a Grid,
}

impl GridTranslations<'_> {
	fn out_of_bounds(&self) -> bool {
		self.width > self.grid.width
	}

	fn iterate(&mut self) {
		self.height += 1;

		if self.height <= self.grid.height {
			return;
		}

		self.width += 1;
		self.height = 1;
	}
}

impl Iterator for GridTranslations<'_> {
	type Item = Vec3;

	fn next(&mut self) -> Option<Self::Item> {
		if self.out_of_bounds() {
			return None;
		}

		let translation = Vec3::new(self.width as f32, self.height as f32, 0.);

		self.iterate();

		Some((translation + self.offset) * self.grid.scale)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assert_count;

	#[test]
	fn translations_1_by_1() {
		let grid = Grid {
			height: 1,
			width: 1,
			..default()
		};

		let [translation] = assert_count!(1, grid.translations());

		assert_eq!(Vec3::ZERO, translation);
	}

	#[test]
	fn translations_3_by_3() {
		let grid = Grid {
			width: 3,
			height: 3,
			..default()
		};

		let translations = assert_count!(9, grid.translations());

		assert_eq!(
			[
				Vec3::new(-1., -1., 0.),
				Vec3::new(-1., 0., 0.),
				Vec3::new(-1., 1., 0.),
				Vec3::new(0., -1., 0.),
				Vec3::new(0., 0., 0.),
				Vec3::new(0., 1., 0.),
				Vec3::new(1., -1., 0.),
				Vec3::new(1., 0., 0.),
				Vec3::new(1., 1., 0.),
			],
			translations
		);
	}

	#[test]
	fn translations_2_by_2() {
		let grid = Grid {
			width: 2,
			height: 2,
			..default()
		};

		let translations = assert_count!(4, grid.translations());

		assert_eq!(
			[
				Vec3::new(-0.5, -0.5, 0.),
				Vec3::new(-0.5, 0.5, 0.),
				Vec3::new(0.5, -0.5, 0.),
				Vec3::new(0.5, 0.5, 0.),
			],
			translations
		);
	}

	#[test]
	fn translations_2_by_2_with_scale_10() {
		let grid = Grid {
			width: 2,
			height: 2,
			scale: 10.,
		};

		let translations = assert_count!(4, grid.translations());

		assert_eq!(
			[
				Vec3::new(-5., -5., 0.),
				Vec3::new(-5., 5., 0.),
				Vec3::new(5., -5., 0.),
				Vec3::new(5., 5., 0.),
			],
			translations
		);
	}
}
