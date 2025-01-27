pub mod a_star;
pub mod straight_line;

use super::{
	computed_path::ComputedPath,
	despawn::Despawn,
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

	#[allow(clippy::too_many_arguments)]
	/* FIXME: This system does too much. It would probably be a good idea to move
	 *        start and end tile detection to a separate system.
	 */
	pub fn compute_path(
		mut commands: Commands,
		grids: Res<Assets<TGrid>>,
		computers: Query<(Entity, &Self, &GridContext<TGrid>)>,
		computed_paths: Query<(Entity, &Parent), With<ComputedPath>>,
		tiles: Query<(Entity, &Transform, Ref<TileType>)>,
		mut removed_tiles: RemovedComponents<TileType>,
		mut start: Local<Option<(Entity, Vec2)>>,
		mut end: Local<Option<(Entity, Vec2)>>,
	) where
		TGrid: GetComputeGridNode + GetTranslation,
		TMethod: ComputePath,
	{
		let removed = removed_tiles.read().collect::<Vec<_>>();
		if !tiles.iter().any(|(.., tile_type)| tile_type.is_changed()) && removed.is_empty() {
			return;
		}

		Self::update_start_and_end(&mut start, &mut end, tiles, removed);

		match (*start, *end) {
			(Some((_, start)), Some((_, end))) => {
				for (entity, computer, context) in &computers {
					let path = computer.get_path(context, &grids, start, end);
					Self::spawn_path(&mut commands, entity, path, &computed_paths);
				}
			}
			_ => {
				for (entity, ..) in &computers {
					Self::despawn_path(&mut commands, entity, &computed_paths);
				}
			}
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
		start: &mut Local<Option<(Entity, Vec2)>>,
		end: &mut Local<Option<(Entity, Vec2)>>,
		tiles: Query<(Entity, &Transform, Ref<TileType>)>,
		removed_tiles: Vec<Entity>,
	) {
		for (entity, transform, tile_type) in &tiles {
			if !tile_type.is_changed() {
				continue;
			}

			if **tile_type == TileTypeValue::Start {
				**start = Some((entity, transform.translation.xy()));
			} else if matches!(**start, Some((start, _)) if start == entity) {
				**start = None;
			}

			if **tile_type == TileTypeValue::End {
				**end = Some((entity, transform.translation.xy()));
			} else if matches!(**end, Some((end, _)) if end == entity) {
				**end = None;
			}
		}

		if matches!(**start, Some((entity, _)) if removed_tiles.contains(&entity) ) {
			**start = None;
		}

		if matches!(**end, Some((entity, _)) if removed_tiles.contains(&entity) ) {
			**end = None;
		}
	}

	fn spawn_path(
		commands: &mut Commands,
		entity: Entity,
		path: Option<Vec<Vec3>>,
		computed_paths: &Query<(Entity, &Parent), With<ComputedPath>>,
	) {
		let Some(path) = path else {
			return;
		};

		Self::despawn_path(commands, entity, computed_paths);

		let Some(mut entity) = commands.get_entity(entity) else {
			return;
		};
		entity.with_child(ComputedPath(path));
	}

	fn despawn_path(
		commands: &mut Commands,
		entity: Entity,
		computed_paths: &Query<(Entity, &Parent), With<ComputedPath>>,
	) {
		for (child, parent) in computed_paths {
			if parent.get() != entity {
				continue;
			}
			let Some(mut child) = commands.get_entity(child) else {
				continue;
			};
			child.try_insert(Despawn::NextFrame);
		}
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
					min: ComputeGridNode::new(1, 2),
					max: ComputeGridNode::new(3, 4),
				},
				obstacles: HashSet::from([ComputeGridNode::new(3, 4)]),
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&ComputePathMethod::<_Grid, _Method>::new(_Method {
				grid: ComputeGrid {
					min: ComputeGridNode::new(1, 2),
					max: ComputeGridNode::new(3, 4),
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
					min: ComputeGridNode::new(1, 2),
					max: ComputeGridNode::new(3, 4),
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
					min: ComputeGridNode::new(1, 2),
					max: ComputeGridNode::new(3, 4),
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
					min: ComputeGridNode::new(1, 2),
					max: ComputeGridNode::new(3, 4),
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
			Some(ComputeGridNode::new(x as i32, y as i32))
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
		let [old_path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		let old_path = old_path.id();
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Start),
			Transform::from_xyz(7., 8., 9.),
		));
		app.update();

		assert_eq!(
			Some(&Despawn::NextFrame),
			app.world().entity(old_path).get::<Despawn>(),
		);
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

	#[test]
	fn remove_computed_path_if_start_changed_to_not_being_start() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		let entity = app
			.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			))
			.id();
		let start = app
			.world_mut()
			.spawn((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.set_parent(entity)
			.id();

		app.update();
		app.world_mut()
			.entity_mut(start)
			.insert(TileType::from_value(TileTypeValue::Walkable));
		app.update();

		let [path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		assert_eq!(Some(&Despawn::NextFrame), path.get::<Despawn>());
	}

	#[test]
	fn remove_computed_path_if_end_changed_to_not_being_end() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		let entity = app
			.world_mut()
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
			.id();
		let end = app
			.world_mut()
			.spawn((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			))
			.set_parent(entity)
			.id();

		app.update();
		app.world_mut()
			.entity_mut(end)
			.insert(TileType::from_value(TileTypeValue::Walkable));
		app.update();

		let [path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		assert_eq!(Some(&Despawn::NextFrame), path.get::<Despawn>());
	}

	#[test]
	fn remove_computed_path_if_start_removed() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		let entity = app
			.world_mut()
			.spawn((
				GridContext::from_handle(handle),
				ComputePathMethod::<_Grid, Mock_Method>::new(new_mock!(Mock_Method, |mock| {
					mock.expect_path().return_const(vec![]);
				})),
			))
			.with_child((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			))
			.id();
		let start = app
			.world_mut()
			.spawn((
				TileType::from_value(TileTypeValue::Start),
				Transform::from_xyz(1., 2., 3.),
			))
			.set_parent(entity)
			.id();

		app.update();
		app.world_mut().entity_mut(start).despawn();
		app.update();

		let [path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		assert_eq!(Some(&Despawn::NextFrame), path.get::<Despawn>());
	}

	#[test]
	fn remove_computed_path_if_end_removed() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle);
		let entity = app
			.world_mut()
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
			.id();
		let end = app
			.world_mut()
			.spawn((
				TileType::from_value(TileTypeValue::End),
				Transform::from_xyz(4., 5., 6.),
			))
			.set_parent(entity)
			.id();

		app.update();
		app.world_mut().entity_mut(end).despawn();
		app.update();

		let [path] = assert_count!(1, app.world().iter_entities().filter(is::<ComputedPath>));
		assert_eq!(Some(&Despawn::NextFrame), path.get::<Despawn>());
	}
}
