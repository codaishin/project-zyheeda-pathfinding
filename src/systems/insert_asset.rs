use bevy::{ecs::query::QueryFilter, prelude::*};
use uuid::Uuid;

use crate::traits::into_component::IntoComponent;

impl<T> InsertAssetSystem for T {}

pub trait InsertAssetSystem {
	#[allow(clippy::type_complexity)]
	fn insert_asset<T>(asset: T) -> impl Fn(Commands, ResMut<Assets<T>>, Query<Entity, Self>)
	where
		T: Asset + Clone,
		Handle<T>: IntoComponent,
		Self: QueryFilter + Sized,
	{
		let id = AssetId::Uuid {
			uuid: Uuid::new_v4(),
		};

		move |mut commands, mut assets, entities| {
			for entity in &entities {
				let Some(mut entity) = commands.get_entity(entity) else {
					continue;
				};
				let handle = get_or_insert_material(&mut assets, id, &asset);
				entity.try_insert(handle.into_component());
			}
		}
	}
}

fn get_or_insert_material<T>(assets: &mut ResMut<Assets<T>>, id: AssetId<T>, asset: &T) -> Handle<T>
where
	T: Asset + Clone,
{
	assets.get_or_insert_with(id, || asset.clone());

	let Some(handle) = assets.get_strong_handle(id) else {
		unreachable!();
	};

	handle
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assert_count;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	#[derive(Component)]
	struct _Target;

	#[derive(Component, Debug, PartialEq)]
	struct _Component;

	#[derive(Asset, TypePath, Clone)]
	struct _Asset;

	impl IntoComponent for Handle<_Asset> {
		type TComponent = _Component;

		fn into_component(self) -> Self::TComponent {
			_Component
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.init_resource::<Assets<_Asset>>();

		app
	}

	#[test]
	fn insert_asset_into_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.run_system_once(With::<_Target>::insert_asset(_Asset))?;

		assert_eq!(
			Some(&_Component),
			app.world().entity(entity).get::<_Component>(),
		);
		Ok(())
	}

	#[test]
	fn insert_asset_into_assets() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Target);

		app.world_mut()
			.run_system_once(With::<_Target>::insert_asset(_Asset))?;

		let assets = app.world().resource::<Assets<_Asset>>();
		assert_count!(1, assets.iter());
		Ok(())
	}

	#[test]
	fn insert_asset_into_assets_only_once() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Target);
		app.world_mut().spawn(_Target);

		app.world_mut()
			.run_system_once(With::<_Target>::insert_asset(_Asset))?;

		let assets = app.world().resource::<Assets<_Asset>>();
		assert_count!(1, assets.iter());
		Ok(())
	}

	#[test]
	fn insert_asset_into_assets_only_once_over_multiple_frames() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Target);

		app.add_systems(Update, With::<_Target>::insert_asset(_Asset));
		app.update();
		app.update();

		let assets = app.world().resource::<Assets<_Asset>>();
		assert_count!(1, assets.iter());
		Ok(())
	}

	#[test]
	fn do_not_run_on_filter_mismatch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(With::<_Target>::insert_asset(_Asset))?;

		assert_eq!(
			(None, 0),
			(
				app.world().entity(entity).get::<_Component>(),
				app.world().resource::<Assets<_Asset>>().iter().count()
			)
		);
		Ok(())
	}
}
