use super::{
	tile::Tile,
	tile_type::{TileType, TileTypeValue},
};
use crate::{
	assets::grid::Grid,
	traits::computable_grid::{ComputableGrid, ComputeGrid, ComputeGridNode, GetComputeGridNode},
};
use bevy::prelude::*;
use std::{collections::HashSet, fmt::Debug};

#[derive(Component, Debug, PartialEq)]
pub struct GridContext<TGrid = Grid>
where
	TGrid: Asset,
{
	handle: Handle<TGrid>,
	grid: ComputeGrid,
	obstacles: HashSet<ComputeGridNode>,
}

impl<TGrid> GridContext<TGrid>
where
	TGrid: Asset,
{
	pub fn from_handle(handle: Handle<TGrid>) -> Self {
		Self {
			handle,
			grid: ComputeGrid::default(),
			obstacles: HashSet::default(),
		}
	}

	pub fn spawn_tiles(
		mut commands: Commands,
		mut contexts: Query<(Entity, &mut Self, Option<&Children>)>,
		asset_events: EventReader<AssetEvent<TGrid>>,
		grids: Res<Assets<TGrid>>,
		tiles: Query<(), With<Tile>>,
	) where
		TGrid: ComputableGrid,
	{
		if asset_events.is_empty() {
			return;
		}

		let changed_assets = changed_assets(asset_events);

		for (entity, mut context, children) in &mut contexts {
			if !changed_assets.contains(&context.handle.id()) {
				continue;
			}

			despawn_old_tiles(&mut commands, children, &tiles);
			spawn_new_tiles(&mut commands, &mut context, entity, &grids);
		}
	}

	pub fn track_obstacles(
		mut contexts: Query<&mut Self>,
		grids: Res<Assets<TGrid>>,
		tiles: Query<(&Transform, &TileType, &Parent), Changed<TileType>>,
	) where
		TGrid: GetComputeGridNode,
	{
		for (transform, tile_type, parent) in &tiles {
			if **tile_type != TileTypeValue::Obstacle {
				continue;
			}
			let Ok(mut context) = contexts.get_mut(parent.get()) else {
				continue;
			};
			let Some(grid) = grids.get(&context.handle) else {
				continue;
			};
			let Some(node) = grid.compute_grid_node(transform.translation.xy()) else {
				continue;
			};

			context.obstacles.insert(node);
		}
	}
}

#[cfg(test)]
impl<TGrid> Default for GridContext<TGrid>
where
	TGrid: Asset,
{
	fn default() -> Self {
		Self {
			handle: Default::default(),
			grid: Default::default(),
			obstacles: Default::default(),
		}
	}
}

fn changed_assets<T>(mut asset_events: EventReader<'_, '_, AssetEvent<T>>) -> Vec<AssetId<T>>
where
	T: Asset,
{
	let changed_assets = asset_events
		.read()
		.filter_map(is_changed)
		.collect::<Vec<_>>();
	changed_assets
}

fn despawn_old_tiles(
	commands: &mut Commands,
	children: Option<&Children>,
	tiles: &Query<(), With<Tile>>,
) {
	let Some(children) = children else {
		return;
	};

	for child in children.iter().filter(is_contained_in(tiles)) {
		let Some(child) = commands.get_entity(*child) else {
			continue;
		};
		child.despawn_recursive();
	}
}

fn spawn_new_tiles<TGrid>(
	commands: &mut Commands,
	context: &mut Mut<GridContext<TGrid>>,
	entity: Entity,
	grids: &Res<Assets<TGrid>>,
) where
	TGrid: Asset + ComputableGrid,
{
	let Some(grid) = grids.get(&context.handle) else {
		return;
	};
	let Some(mut entity) = commands.get_entity(entity) else {
		return;
	};

	context.grid = grid.grid();
	context.obstacles.clear();
	for Vec2 { x, y } in grid.translations() {
		entity.with_child((Tile, Transform::from_xyz(x, y, 0.)));
	}
}

fn is_changed<T>(event: &AssetEvent<T>) -> Option<AssetId<T>>
where
	T: Asset,
{
	match event {
		AssetEvent::Added { id } => Some(*id),
		AssetEvent::Modified { id } => Some(*id),
		_ => None,
	}
}

fn is_contained_in<'a>(tiles: &'a Query<(), With<Tile>>) -> impl FnMut(&&Entity) -> bool + 'a {
	|entity| tiles.contains(**entity)
}

#[cfg(test)]
mod test_spawning_tiles {
	use super::*;
	use crate::{assert_count, components::tile::Tile, new_handle, test_tools::SingleThreaded};
	use std::vec::IntoIter;

	#[derive(Asset, TypePath, Default)]
	struct _Grid {
		grid: ComputeGrid,
		translations: Vec<Vec2>,
	}

	impl ComputableGrid for _Grid {
		type TIter<'a>
			= IntoIter<Vec2>
		where
			Self: 'a;

		fn translations(&self) -> Self::TIter<'_> {
			self.translations.clone().into_iter()
		}

		fn grid(&self) -> ComputeGrid {
			self.grid
		}
	}

	fn setup(handle: &Handle<_Grid>, grid: _Grid) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<_Grid>::default();

		assets.insert(handle.id(), grid);
		app.add_event::<AssetEvent<_Grid>>();
		app.insert_resource(assets);
		app.add_systems(Update, GridContext::<_Grid>::spawn_tiles);

		app
	}

	fn is_child_of(entity: Entity) -> impl Fn(&EntityRef) -> bool {
		move |child| {
			let Some(parent) = child.get::<Parent>() else {
				return false;
			};
			parent.get() == entity
		}
	}

	#[test]
	fn spawn_tiles_as_children_when_grid_asset_added() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				translations: vec![Vec2::splat(1.), Vec2::splat(2.)],
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext::from_handle(handle.clone()))
			.id();

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		let children = assert_count!(2, app.world().iter_entities().filter(is_child_of(entity)));
		assert_eq!(
			[
				(Some(&Tile), Some(Vec3::new(1., 1., 0.))),
				(Some(&Tile), Some(Vec3::new(2., 2., 0.)))
			],
			children.map(|c| (c.get::<Tile>(), c.get::<Transform>().map(|t| t.translation)))
		);
	}

	#[test]
	fn store_compute_grid_when_grid_asset_added() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				grid: ComputeGrid {
					width: 11,
					height: 4,
				},
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext::from_handle(handle.clone()))
			.id();

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		assert_eq!(
			Some(ComputeGrid {
				width: 11,
				height: 4
			}),
			app.world()
				.entity(entity)
				.get::<GridContext<_Grid>>()
				.map(|g| g.grid)
		);
	}

	#[test]
	fn clear_obstacles_when_grid_asset_added() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				grid: ComputeGrid {
					width: 11,
					height: 4,
				},
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext {
				handle: handle.clone(),
				obstacles: HashSet::from([ComputeGridNode::new(1, 2)]),
				..default()
			})
			.id();

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		assert_eq!(
			Some(&HashSet::from([])),
			app.world()
				.entity(entity)
				.get::<GridContext<_Grid>>()
				.map(|g| &g.obstacles)
		);
	}

	#[test]
	fn do_not_spawn_tiles_when_other_identical_gird_asset_added() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				translations: vec![Vec2::splat(1.), Vec2::splat(2.)],
				..default()
			},
		);
		let entity = app.world_mut().spawn(GridContext::from_handle(handle)).id();

		app.world_mut().send_event(AssetEvent::Added {
			id: new_handle!(_Grid).id(),
		});
		app.update();

		assert_count!(0, app.world().iter_entities().filter(is_child_of(entity)));
	}

	#[test]
	fn spawn_tiles_as_children_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				translations: vec![Vec2::splat(1.), Vec2::splat(2.)],
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext::from_handle(handle.clone()))
			.id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		let children = assert_count!(2, app.world().iter_entities().filter(is_child_of(entity)));
		assert_eq!(
			[
				(Some(&Tile), Some(Vec3::new(1., 1., 0.))),
				(Some(&Tile), Some(Vec3::new(2., 2., 0.)))
			],
			children.map(|c| (c.get::<Tile>(), c.get::<Transform>().map(|t| t.translation)))
		);
	}

	#[test]
	fn despawn_old_tiles_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				translations: vec![Vec2::splat(1.), Vec2::splat(2.)],
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext::from_handle(handle.clone()))
			.id();
		let child = app.world_mut().spawn(Tile).set_parent(entity).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		assert!(app.world().get_entity(child).is_err());
	}

	#[test]
	fn do_not_despawn_non_tiles_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				translations: vec![Vec2::splat(1.), Vec2::splat(2.)],
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext::from_handle(handle.clone()))
			.id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		assert!(app.world().get_entity(child).is_ok());
	}

	#[test]
	fn despawn_old_tiles_recursively_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(
			&handle,
			_Grid {
				translations: vec![Vec2::splat(1.), Vec2::splat(2.)],
				..default()
			},
		);
		let entity = app
			.world_mut()
			.spawn(GridContext::from_handle(handle.clone()))
			.id();
		let child = app.world_mut().spawn(Tile).set_parent(entity).id();
		let child_child = app.world_mut().spawn_empty().set_parent(child).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		assert!(app.world().get_entity(child_child).is_err());
	}
}

#[cfg(test)]
mod test_tracking_of_tiles {
	use super::*;
	use crate::{
		components::tile_type::{TileType, TileTypeValue},
		new_handle,
		test_tools::SingleThreaded,
	};

	#[derive(Asset, TypePath)]
	struct _Grid;

	impl GetComputeGridNode for _Grid {
		fn compute_grid_node(&self, Vec2 { x, y }: Vec2) -> Option<ComputeGridNode> {
			Some(ComputeGridNode {
				x: x as usize,
				y: y as usize,
			})
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Changed(bool);

	fn detect_change(mut commands: Commands, contexts: Query<(Entity, Ref<GridContext<_Grid>>)>) {
		for (entity, context) in &contexts {
			commands
				.entity(entity)
				.insert(_Changed(context.is_changed()));
		}
	}

	fn setup(handle: &Handle<_Grid>, grid: _Grid) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut grids = Assets::default();
		grids.insert(handle, grid);

		app.insert_resource(grids);
		app.add_systems(
			Update,
			(GridContext::<_Grid>::track_obstacles, detect_change).chain(),
		);

		app
	}

	#[test]
	fn add_obstacle() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid);
		let entity = app
			.world_mut()
			.spawn(GridContext {
				handle,
				..default()
			})
			.with_child((
				TileType::from_value(TileTypeValue::Obstacle),
				Transform::from_xyz(1., 2., 3.),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&HashSet::from([ComputeGridNode::new(1, 2)])),
			app.world()
				.entity(entity)
				.get::<GridContext<_Grid>>()
				.map(|g| &g.obstacles)
		);
	}

	#[test]
	fn do_not_add_obstacle_when_not_child() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid);
		let entity = app
			.world_mut()
			.spawn(GridContext {
				handle,
				..default()
			})
			.id();
		app.world_mut().spawn((
			TileType::from_value(TileTypeValue::Obstacle),
			Transform::from_xyz(1., 2., 3.),
		));

		app.update();

		assert_eq!(
			Some(&HashSet::from([])),
			app.world()
				.entity(entity)
				.get::<GridContext<_Grid>>()
				.map(|g| &g.obstacles)
		);
	}

	#[test]
	fn do_not_add_obstacle_when_not_tile_type_is_not_obstacle() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid);
		let entity = app
			.world_mut()
			.spawn(GridContext {
				handle,
				..default()
			})
			.with_child((
				TileType::from_value(TileTypeValue::Walkable),
				Transform::from_xyz(1., 2., 3.),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&HashSet::from([])),
			app.world()
				.entity(entity)
				.get::<GridContext<_Grid>>()
				.map(|g| &g.obstacles)
		);
	}

	#[test]
	fn mut_deref_context_obstacle_only_once() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid);
		let entity = app
			.world_mut()
			.spawn(GridContext {
				handle,
				..default()
			})
			.with_child((
				TileType::from_value(TileTypeValue::Obstacle),
				Transform::from_xyz(1., 2., 3.),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&_Changed(false)),
			app.world().entity(entity).get::<_Changed>()
		);
	}

	#[test]
	fn mut_deref_context_if_tile_type_mutable_deref_occurred() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid);
		let entity = app
			.world_mut()
			.spawn(GridContext {
				handle,
				..default()
			})
			.id();
		let child = app
			.world_mut()
			.spawn((
				TileType::from_value(TileTypeValue::Obstacle),
				Transform::from_xyz(1., 2., 3.),
			))
			.set_parent(entity)
			.id();

		app.update();
		app.world_mut()
			.entity_mut(child)
			.get_mut::<TileType>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			Some(&_Changed(true)),
			app.world().entity(entity).get::<_Changed>()
		);
	}
}
