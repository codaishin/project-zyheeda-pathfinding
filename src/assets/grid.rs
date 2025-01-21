use crate::{
	components::grid_context::GridContext,
	traits::{
		computable_grid::{
			ComputableGrid,
			ComputeGrid,
			ComputeGridNode,
			GetComputeGridNode,
			GetTranslation,
		},
		into_component::IntoComponent,
	},
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
	type TComponent = GridContext;

	fn into_component(self) -> Self::TComponent {
		GridContext::from_handle(self)
	}
}

impl ComputableGrid for Grid {
	type TIter<'a> = GridTranslations<'a>;

	fn translations(&self) -> Self::TIter<'_> {
		GridTranslations {
			grid: self,
			width: 0,
			height: 0,
			offset: Vec2 {
				x: -((self.width - 1) as f32 / 2.),
				y: -((self.height - 1) as f32 / 2.),
			},
		}
	}

	fn grid(&self) -> ComputeGrid {
		ComputeGrid {
			width: self.width,
			height: self.height,
		}
	}
}

impl GetComputeGridNode for Grid {
	fn compute_grid_node(&self, Vec2 { x, y }: Vec2) -> Option<ComputeGridNode> {
		Some(ComputeGridNode::new(
			(x / self.scale + self.width as f32 / 2.) as usize,
			(y / self.scale + self.height as f32 / 2.) as usize,
		))
	}
}

impl GetTranslation for Grid {
	fn translation(&self, ComputeGridNode { x, y }: ComputeGridNode) -> Option<Vec2> {
		Some(
			Vec2 {
				x: (x as f32 - (self.width - 1) as f32 / 2.),
				y: (y as f32 - (self.height - 1) as f32 / 2.),
			} * self.scale,
		)
	}
}

pub struct GridTranslations<'a> {
	width: usize,
	height: usize,
	offset: Vec2,
	grid: &'a Grid,
}

impl GridTranslations<'_> {
	fn out_of_bounds(&self) -> bool {
		self.width >= self.grid.width
	}

	fn iterate(&mut self) {
		self.height += 1;

		if self.height < self.grid.height {
			return;
		}

		self.width += 1;
		self.height = 0;
	}
}

impl Iterator for GridTranslations<'_> {
	type Item = Vec2;

	fn next(&mut self) -> Option<Self::Item> {
		if self.out_of_bounds() {
			return None;
		}

		let translation = Vec2::new(self.width as f32, self.height as f32);

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

		assert_eq!(Vec2::ZERO, translation);
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
				Vec2::new(-1., -1.),
				Vec2::new(-1., 0.),
				Vec2::new(-1., 1.),
				Vec2::new(0., -1.),
				Vec2::new(0., 0.),
				Vec2::new(0., 1.),
				Vec2::new(1., -1.),
				Vec2::new(1., 0.),
				Vec2::new(1., 1.),
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
				Vec2::new(-0.5, -0.5),
				Vec2::new(-0.5, 0.5),
				Vec2::new(0.5, -0.5),
				Vec2::new(0.5, 0.5),
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
				Vec2::new(-5., -5.),
				Vec2::new(-5., 5.),
				Vec2::new(5., -5.),
				Vec2::new(5., 5.),
			],
			translations
		);
	}

	#[test]
	fn get_compute_node_1_by_1() {
		let grid = Grid {
			width: 1,
			height: 1,
			..default()
		};

		let node = grid.compute_grid_node(Vec2::ZERO);

		assert_eq!(Some(ComputeGridNode::ZERO), node);
	}

	#[test]
	fn get_compute_node_3_by_3() {
		let grid = Grid {
			width: 3,
			height: 3,
			..default()
		};

		let node = grid.compute_grid_node(Vec2::ZERO);

		assert_eq!(Some(ComputeGridNode::new(1, 1)), node);
	}

	#[test]
	fn get_compute_node_4_by_4() {
		let grid = Grid {
			width: 4,
			height: 4,
			..default()
		};

		let node = grid.compute_grid_node(Vec2::new(1.5, -0.5));

		assert_eq!(Some(ComputeGridNode::new(3, 1)), node);
	}

	#[test]
	fn get_compute_node_4_by_3_scaled_by_10() {
		let grid = Grid {
			width: 4,
			height: 3,
			scale: 10.,
		};

		let node = grid.compute_grid_node(Vec2::new(15., 10.));

		assert_eq!(Some(ComputeGridNode::new(3, 2)), node);
	}

	#[test]
	fn get_translation_1_by_1() {
		let grid = Grid {
			width: 1,
			height: 1,
			..default()
		};

		let node = grid.translation(ComputeGridNode::ZERO);

		assert_eq!(Some(Vec2::ZERO), node);
	}

	#[test]
	fn get_translation_3_by_3() {
		let grid = Grid {
			width: 3,
			height: 3,
			..default()
		};

		let node = grid.translation(ComputeGridNode::new(2, 0));

		assert_eq!(Some(Vec2::new(1., -1.)), node);
	}

	#[test]
	fn get_translation_4_by_4() {
		let grid = Grid {
			width: 4,
			height: 4,
			..default()
		};

		let node = grid.translation(ComputeGridNode::new(3, 2));

		assert_eq!(Some(Vec2::new(1.5, 0.5)), node);
	}

	#[test]
	fn get_translation_4_by_3_scaled_by_10() {
		let grid = Grid {
			width: 4,
			height: 3,
			scale: 10.,
		};

		let node = grid.translation(ComputeGridNode::new(3, 2));

		assert_eq!(Some(Vec2::new(15., 10.),), node);
	}
}
