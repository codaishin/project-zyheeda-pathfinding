pub mod straight_line;

use super::grid_context::GridContext;
use crate::traits::compute_path::NewComputer;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct ComputePathMethod<TMethod>(TMethod);

impl<TMethod> ComputePathMethod<TMethod>
where
	TMethod: NewComputer + Sync + Send + 'static,
{
	pub fn instantiate(
		mut commands: Commands,
		contexts: Query<(Entity, &GridContext), Changed<GridContext>>,
	) {
		for (entity, context) in &contexts {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.try_insert(Self(TMethod::new(context.grid, context.obstacles.clone())));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		assets::grid::Grid,
		components::grid_context::GridContext,
		test_tools::SingleThreaded,
		traits::computable_grid::{ComputeGrid, ComputeGridNode},
	};
	use std::collections::HashSet;

	#[derive(Debug, PartialEq)]
	struct _Method {
		grid: ComputeGrid,
		obstacles: HashSet<ComputeGridNode>,
	}

	impl NewComputer for _Method {
		fn new(grid: ComputeGrid, obstacles: HashSet<ComputeGridNode>) -> Self {
			Self { grid, obstacles }
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ComputePathMethod::<_Method>::instantiate);

		app
	}

	#[test]
	fn instantiate_from_grid_context() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GridContext::<Grid> {
				grid: ComputeGrid {
					width: 10,
					height: 11,
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&ComputePathMethod(_Method {
				grid: ComputeGrid {
					width: 10,
					height: 11
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
			})),
			app.world()
				.entity(entity)
				.get::<ComputePathMethod<_Method>>()
		);
	}

	#[test]
	fn instantiate_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GridContext::<Grid> {
				grid: ComputeGrid {
					width: 10,
					height: 11,
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
				..default()
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<ComputePathMethod<_Method>>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<ComputePathMethod<_Method>>()
		);
	}

	#[test]
	fn instantiate_again_when_grid_context_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GridContext::<Grid> {
				grid: ComputeGrid {
					width: 10,
					height: 11,
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
				..default()
			})
			.id();

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.remove::<ComputePathMethod<_Method>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<GridContext>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&ComputePathMethod(_Method {
				grid: ComputeGrid {
					width: 10,
					height: 11
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
			})),
			app.world()
				.entity(entity)
				.get::<ComputePathMethod<_Method>>()
		);
	}
}
