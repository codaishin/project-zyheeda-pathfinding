use super::tile::Tile;
use crate::{assets::grid::Grid, traits::translations::Translations};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub struct TileBuilder<TGrid = Grid>(pub Handle<TGrid>)
where
	TGrid: Asset + Translations;

impl<TGrid> TileBuilder<TGrid>
where
	TGrid: Asset + Translations,
{
	pub fn spawn_tiles(
		mut commands: Commands,
		asset_events: EventReader<AssetEvent<TGrid>>,
		grid_assets: Res<Assets<TGrid>>,
		grids: Query<(Entity, &Self, Option<&Children>)>,
		tiles: Query<(), With<Tile>>,
	) {
		if asset_events.is_empty() {
			return;
		}

		let changed_assets = changed_assets(asset_events);

		for (entity, TileBuilder(grid), children) in &grids {
			if !changed_assets.contains(&grid.id()) {
				continue;
			}

			despawn_old_tiles(&mut commands, children, &tiles);
			spawn_new_tiles(&mut commands, entity, &grid_assets, grid);
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
	entity: Entity,
	grid_assets: &Res<Assets<TGrid>>,
	grid: &Handle<TGrid>,
) where
	TGrid: Asset + Translations,
{
	let Some(grid) = grid_assets.get(grid) else {
		return;
	};
	let Some(mut entity) = commands.get_entity(entity) else {
		return;
	};

	for translation in grid.translations() {
		entity.with_child((Tile, Transform::from_translation(translation)));
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
mod tests {
	use super::*;
	use crate::{assert_count, components::tile::Tile, new_handle, test_tools::SingleThreaded};
	use std::vec::IntoIter;

	#[derive(Asset, TypePath)]
	struct _Grid(Vec<Vec3>);

	impl Translations for _Grid {
		type TIter<'a>
			= IntoIter<Vec3>
		where
			Self: 'a;

		fn translations(&self) -> Self::TIter<'_> {
			self.0.clone().into_iter()
		}
	}

	fn setup(handle: &Handle<_Grid>, grid: _Grid) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<_Grid>::default();

		assets.insert(handle.id(), grid);
		app.add_event::<AssetEvent<_Grid>>();
		app.insert_resource(assets);
		app.add_systems(Update, TileBuilder::<_Grid>::spawn_tiles);

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
		let mut app = setup(&handle, _Grid(vec![Vec3::splat(1.), Vec3::splat(2.)]));
		let entity = app.world_mut().spawn(TileBuilder(handle.clone())).id();

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		let children = assert_count!(2, app.world().iter_entities().filter(is_child_of(entity)));
		assert_eq!(
			[
				(Some(&Tile), Some(Vec3::splat(1.))),
				(Some(&Tile), Some(Vec3::splat(2.)))
			],
			children.map(|c| (c.get::<Tile>(), c.get::<Transform>().map(|t| t.translation)))
		);
	}

	#[test]
	fn do_not_spawn_tiles_when_other_gird_asset_added() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid(vec![Vec3::splat(1.), Vec3::splat(2.)]));
		let entity = app.world_mut().spawn(TileBuilder(handle)).id();

		app.world_mut().send_event(AssetEvent::Added {
			id: new_handle!(_Grid).id(),
		});
		app.update();

		assert_count!(0, app.world().iter_entities().filter(is_child_of(entity)));
	}

	#[test]
	fn spawn_tiles_as_children_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid(vec![Vec3::splat(1.), Vec3::splat(2.)]));
		let entity = app.world_mut().spawn(TileBuilder(handle.clone())).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		let children = assert_count!(2, app.world().iter_entities().filter(is_child_of(entity)));
		assert_eq!(
			[
				(Some(&Tile), Some(Vec3::splat(1.))),
				(Some(&Tile), Some(Vec3::splat(2.)))
			],
			children.map(|c| (c.get::<Tile>(), c.get::<Transform>().map(|t| t.translation)))
		);
	}

	#[test]
	fn despawn_old_tiles_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid(vec![Vec3::splat(1.), Vec3::splat(2.)]));
		let entity = app.world_mut().spawn(TileBuilder(handle.clone())).id();
		let child = app.world_mut().spawn(Tile).set_parent(entity).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		assert!(app.world().get_entity(child).is_err());
	}

	#[test]
	fn do_not_despawn_non_tiles_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid(vec![Vec3::splat(1.), Vec3::splat(2.)]));
		let entity = app.world_mut().spawn(TileBuilder(handle.clone())).id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		assert!(app.world().get_entity(child).is_ok());
	}

	#[test]
	fn despawn_old_tiles_recursively_when_grid_asset_modified() {
		let handle = new_handle!(_Grid);
		let mut app = setup(&handle, _Grid(vec![Vec3::splat(1.), Vec3::splat(2.)]));
		let entity = app.world_mut().spawn(TileBuilder(handle.clone())).id();
		let child = app.world_mut().spawn(Tile).set_parent(entity).id();
		let child_child = app.world_mut().spawn_empty().set_parent(child).id();

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		assert!(app.world().get_entity(child_child).is_err());
	}
}
