use crate::traits::translations::Translations;
use bevy::prelude::*;

impl<T> SpawnComponents for T {}

pub trait SpawnComponents {
	fn spawn<TComponent>(
		mut commands: Commands,
		mut asset_events: EventReader<AssetEvent<Self>>,
		assets: Res<Assets<Self>>,
		tiles: Query<Entity, With<TComponent>>,
	) where
		Self: Asset + Translations + Sized,
		TComponent: Component + Default,
	{
		for event in asset_events.read() {
			let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = event else {
				continue;
			};
			let Some(grid) = assets.get(*id) else {
				continue;
			};

			despawn::<TComponent>(&mut commands, &tiles);
			spawn::<TComponent>(&mut commands, grid);
		}
	}
}

fn despawn<TComponent>(commands: &mut Commands, tiles: &Query<Entity, With<TComponent>>)
where
	TComponent: Component,
{
	for tile in tiles {
		let Some(entity) = commands.get_entity(tile) else {
			continue;
		};
		entity.despawn_recursive();
	}
}

fn spawn<TComponent>(commands: &mut Commands, grid: &impl Translations)
where
	TComponent: Component + Default,
{
	for translation in grid.translations() {
		commands.spawn((
			TComponent::default(),
			Transform::from_translation(translation),
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_count, new_handle, test_tools::SingleThreaded};
	use std::vec::IntoIter;

	#[derive(Asset, TypePath)]
	struct _Grid(Vec<Vec3>);

	#[derive(Component, Default, Debug, PartialEq)]
	struct _Tile;

	impl Translations for _Grid {
		type TIter<'a> = IntoIter<Vec3>;

		fn translations(&self) -> Self::TIter<'_> {
			self.0.clone().into_iter()
		}
	}

	fn setup(handle: &Handle<_Grid>, grid_asset: _Grid) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::<_Grid>::default();

		assets.insert(handle.id(), grid_asset);
		app.add_event::<AssetEvent<_Grid>>();
		app.insert_resource(assets);
		app.add_systems(Update, _Grid::spawn::<_Tile>);

		app
	}

	#[test]
	fn spawn_tiles_when_asset_added() {
		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![Vec3::new(1., 0., 0.), Vec3::new(2., 0., 0.)]);
		let mut app = setup(&handle, grid);

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		let tiles = assert_count!(2, app.world().iter_entities());
		assert_eq!(
			[
				(Some(&_Tile), Some(Vec3::new(1., 0., 0.))),
				(Some(&_Tile), Some(Vec3::new(2., 0., 0.)))
			],
			tiles.map(|t| (
				t.get::<_Tile>(),
				t.get::<Transform>().map(|t| t.translation)
			))
		);
	}

	#[test]
	fn spawn_none_when_asset_not_added() {
		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![Vec3::default()]);
		let mut app = setup(&handle, grid);

		app.update();

		assert_count!(0, app.world().iter_entities());
	}

	#[test]
	fn spawn_none_when_added_handle_does_not_match() {
		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![Vec3::default()]);
		let mut app = setup(&handle, grid);

		app.world_mut().send_event(AssetEvent::Added {
			id: new_handle!(_Grid).id(),
		});
		app.update();

		assert_count!(0, app.world().iter_entities());
	}

	#[test]
	fn spawn_tiles_when_asset_changed() {
		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![Vec3::new(1., 0., 0.), Vec3::new(2., 0., 0.)]);
		let mut app = setup(&handle, grid);

		app.world_mut()
			.send_event(AssetEvent::Modified { id: handle.id() });
		app.update();

		let tiles = assert_count!(2, app.world().iter_entities());
		assert_eq!(
			[
				(Some(&_Tile), Some(Vec3::new(1., 0., 0.))),
				(Some(&_Tile), Some(Vec3::new(2., 0., 0.)))
			],
			tiles.map(|t| (
				t.get::<_Tile>(),
				t.get::<Transform>().map(|t| t.translation)
			))
		);
	}

	#[test]
	fn despawn_old_tiles_when_asset_added() {
		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![]);
		let mut app = setup(&handle, grid);
		app.world_mut().spawn(_Tile);

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		assert_count!(0, app.world().iter_entities());
	}

	#[test]
	fn only_despawn_old_tiles_when_asset_added() {
		#[derive(Component)]
		struct _Other;

		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![]);
		let mut app = setup(&handle, grid);
		app.world_mut().spawn(_Other);

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		assert_count!(1, app.world().iter_entities());
	}

	#[test]
	fn despawn_old_tiles_recursively_when_asset_added() {
		#[derive(Component)]
		struct _Child;

		let handle = new_handle!(_Grid);
		let grid = _Grid(vec![]);
		let mut app = setup(&handle, grid);
		app.world_mut().spawn(_Tile).with_child(_Child);

		app.world_mut()
			.send_event(AssetEvent::Added { id: handle.id() });
		app.update();

		assert_count!(0, app.world().iter_entities());
	}
}
