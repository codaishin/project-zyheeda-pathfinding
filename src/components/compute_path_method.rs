pub mod straight_line;

use super::{
	computed_path::ComputedPath,
	grid_context::GridContext,
	tile_type::{TileType, TileTypeValue},
};
use crate::traits::{
	computable_grid::{GetComputeGridNode, GetTranslation},
	compute_path::{ComputePath, NewComputer},
};
use bevy::prelude::*;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
pub struct ComputePathMethod<TGrid, TMethod> {
	method: TMethod,
	_p: PhantomData<TGrid>,
}

impl<TGrid, TMethod> ComputePathMethod<TGrid, TMethod>
where
	TGrid: Asset + Sync + Send + 'static,
	TMethod: Sync + Send + 'static,
{
	fn new(method: TMethod) -> Self {
		Self {
			method,
			_p: PhantomData,
		}
	}

	pub fn instantiate(
		mut commands: Commands,
		contexts: Query<(Entity, &GridContext<TGrid>), Changed<GridContext<TGrid>>>,
	) where
		TMethod: NewComputer,
	{
		for (entity, context) in &contexts {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.try_insert(Self::new(TMethod::new(
				context.grid,
				context.obstacles.clone(),
			)));
		}
	}

	pub fn compute_path(
		mut commands: Commands,
		grids: Res<Assets<TGrid>>,
		computers: Query<(Entity, &Self, &GridContext<TGrid>)>,
		computed_paths: Query<(Entity, &Parent), With<ComputedPath>>,
		changed_tiles: Query<(&Transform, &TileType), Changed<TileType>>,
		mut start: Local<Option<Vec2>>,
		mut end: Local<Option<Vec2>>,
	) where
		TGrid: GetComputeGridNode + GetTranslation,
		TMethod: ComputePath,
	{
		if changed_tiles.is_empty() {
			return;
		}

		Self::update_start_and_end(&mut start, &mut end, changed_tiles);

		let (Some(start), Some(end)) = (*start, *end) else {
			return;
		};

		for (entity, computer, context) in &computers {
			let Some(path) = computer.get_path(context, &grids, start, end) else {
				continue;
			};

			Self::spawn_path(&mut commands, entity, path, &computed_paths);
		}
	}

	fn get_path(
		&self,
		context: &GridContext<TGrid>,
		grids: &Assets<TGrid>,
		start: Vec2,
		end: Vec2,
	) -> Option<Vec<Vec3>>
	where
		TGrid: GetComputeGridNode + GetTranslation,
		TMethod: ComputePath,
	{
		let grid = grids.get(&context.handle)?;
		let start = grid.compute_grid_node(start)?;
		let end = grid.compute_grid_node(end)?;
		let path = self
			.method
			.path(start, end)
			.into_iter()
			.filter_map(|node| grid.translation(node))
			.map(|translation| translation.extend(1.))
			.collect::<Vec<_>>();

		Some(path)
	}

	fn update_start_and_end(
		start: &mut Local<Option<Vec2>>,
		end: &mut Local<Option<Vec2>>,
		changed_tiles: Query<(&Transform, &TileType), Changed<TileType>>,
	) {
		for (transform, tile_type) in &changed_tiles {
			if **tile_type == TileTypeValue::Start {
				**start = Some(transform.translation.xy());
			}
			if **tile_type == TileTypeValue::End {
				**end = Some(transform.translation.xy());
			}
		}
	}

	fn spawn_path(
		commands: &mut Commands,
		entity: Entity,
		path: Vec<Vec3>,
		computed_paths: &Query<(Entity, &Parent), With<ComputedPath>>,
	) {
		for (child, parent) in computed_paths {
			if parent.get() != entity {
				continue;
			}
			let Some(child) = commands.get_entity(child) else {
				continue;
			};
			child.despawn_recursive();
		}

		let Some(mut entity) = commands.get_entity(entity) else {
			return;
		};
		entity.with_child(ComputedPath(path));
	}
}

#[cfg(test)]
mod test_instantiation {
	use super::*;
	use crate::{
		components::grid_context::GridContext,
		test_tools::SingleThreaded,
		traits::computable_grid::{ComputeGrid, ComputeGridNode},
	};
	use std::collections::HashSet;

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Grid;

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
		app.add_systems(Update, ComputePathMethod::<_Grid, _Method>::instantiate);

		app
	}

	#[test]
	fn instantiate_from_grid_context() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GridContext::<_Grid> {
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
			Some(&ComputePathMethod::<_Grid, _Method>::new(_Method {
				grid: ComputeGrid {
					width: 10,
					height: 11
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
			})),
			app.world()
				.entity(entity)
				.get::<ComputePathMethod<_Grid, _Method>>()
		);
	}

	#[test]
	fn instantiate_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GridContext::<_Grid> {
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
			.remove::<ComputePathMethod<_Grid, _Method>>();
		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<ComputePathMethod<_Grid, _Method>>()
		);
	}

	#[test]
	fn instantiate_again_when_grid_context_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(GridContext::<_Grid> {
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
			.remove::<ComputePathMethod<_Grid, _Method>>();
		app.world_mut()
			.entity_mut(entity)
			.get_mut::<GridContext<_Grid>>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&ComputePathMethod::<_Grid, _Method>::new(_Method {
				grid: ComputeGrid {
					width: 10,
					height: 11
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
			})),
			app.world()
				.entity(entity)
				.get::<ComputePathMethod<_Grid, _Method>>()
		);
	}
}

#[cfg(test)]
mod test_compute_path {
	use super::*;
	use crate::{
		assert_count,
		components::{
			computed_path::ComputedPath,
			tile_type::{TileType, TileTypeValue},
		},
		new_handle,
		new_mock,
		test_tools::SingleThreaded,
		traits::computable_grid::{ComputeGridNode, GetComputeGridNode, GetTranslation},
	};
	use mockall::{mock, predicate::eq};

	#[derive(Asset, TypePath)]
	struct _Grid;

	impl GetComputeGridNode for _Grid {
		fn compute_grid_node(&self, Vec2 { x, y }: Vec2) -> Option<ComputeGridNode> {
			Some(ComputeGridNode::new(x as usize, y as usize))
		}
	}

	impl GetTranslation for _Grid {
		fn translation(&self, ComputeGridNode { x, y }: ComputeGridNode) -> Option<Vec2> {
			Some(Vec2::new(x as f32, y as f32))
		}
	}

	mock! {
		_Method {}
		impl ComputePath for _Method {
			fn path(&self, start: ComputeGridNode, end: ComputeGridNode) -> Vec<ComputeGridNode>;
		}
	}

	fn setup(handle: &Handle<_Grid>) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut grids = Assets::default();

		grids.insert(handle, _Grid);
		app.insert_resource(grids);
		app.add_systems(
			Update,
			ComputePathMethod::<_Grid, Mock_Method>::compute_path,
		);

		app
	}

	fn child_of(entity: Entity) -> impl Fn(&EntityRef) -> bool {
		move |child| {
			child
				.get::<Parent>()
				.map(|p| p.get() == entity)
				.unwrap_or(false)
		}
	}

	fn is<TComponent>(entity: &EntityRef) -> bool
	where
		TComponent: Component,
	{
		entity.contains::<TComponent>()
	}

	#[test]
	fn spawn_path() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.return_const(vec![ComputeGridNode::new(1, 2), ComputeGridNode::new(4, 5)]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::default(),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::default(),
			));

		app.update();

		let [path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		assert_eq!(
			Some(&ComputedPath(vec![
				Vec3::new(1., 2., 1.),
				Vec3::new(4., 5., 1.)
			])),
			path.get::<ComputedPath>()
		);
	}

	#[test]
	fn spawn_path_nodes_as_children() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		let entity = app
			.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.return_const(vec![ComputeGridNode::new(1, 2), ComputeGridNode::new(4, 5)]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::default(),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::default(),
			))
			.id();

		app.update();

		let paths = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		assert_count!(1, paths.into_iter().filter(child_of(entity)));
	}

	#[test]
	fn spawn_path_nothing_if_start_and_end_missing() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.return_const(vec![ComputeGridNode::new(1, 2), ComputeGridNode::new(4, 5)]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Obstacle),
				Transform::default(),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Obstacle),
				Transform::default(),
			));

		app.update();

		assert_count!(0, app.world().iter_entities().filter(is::<ComputedPath>));
	}

	#[test]
	fn call_path_with_correct_start_and_end() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);

		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.times(1)
						.with(
							eq(ComputeGridNode::new(1, 2)),
							eq(ComputeGridNode::new(4, 5)),
						)
						.return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			));

		app.update();
	}

	#[test]
	fn call_path_with_correct_start_and_end_reversed() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);

		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.times(1)
						.with(
							eq(ComputeGridNode::new(1, 2)),
							eq(ComputeGridNode::new(4, 5)),
						)
						.return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			));

		app.update();
	}

	#[test]
	fn act_only_once() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);

		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path().times(1).return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			));

		app.update();
		app.update();
	}

	#[test]
	fn act_again_if_new_start_added() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);

		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.times(1)
						.with(
							eq(ComputeGridNode::new(7, 8)),
							eq(ComputeGridNode::new(4, 5)),
						)
						.return_const(vec![]);
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			));

		app.update();
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Transform::from_xyz(7., 8., 9.),
		));
		app.update();
	}

	#[test]
	fn act_again_if_start_changed() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);

		let entity = app
			.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path()
						.times(1)
						.with(
							eq(ComputeGridNode::new(7, 8)),
							eq(ComputeGridNode::new(4, 5)),
						)
						.return_const(vec![]);
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			))
			.id();

		let child = app
			.world_mut()
			.spawn((
				TileType::from_value(TileTypeValue::Walkable),
				Transform::from_xyz(7., 8., 9.),
			))
			.set_parent(entity)
			.id();

		app.update();
		let mut child = app.world_mut().entity_mut(child);
		let mut tile_type = child.get_mut::<TileType>().unwrap();
		*tile_type = TileType::from_value(TileTypeValue::Start);
		app.update();
	}

	#[test]
	fn override_old_computed_path_after_change() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			));

		app.update();
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Transform::from_xyz(7., 8., 9.),
		));
		app.update();

		assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
	}

	#[test]
	fn override_old_computed_path_after_change_recursively() {
		#[derive(Component, Debug, PartialEq)]
		struct _Child;

		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			));

		app.update();
		let [path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		let path = path.id();
		let child = app.world_mut().spawn(_Child).set_parent(path).id();
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Transform::from_xyz(7., 8., 9.),
		));
		app.update();

		assert!(app.world().get_entity(child).is_err());
	}

	#[test]
	fn do_not_remove_unrelated_computed_path() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		let other = app.world_mut().spawn(ComputedPath(vec![])).id();
		app.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			));

		app.update();

		assert!(app.world().get_entity(other).is_ok());
	}
}
