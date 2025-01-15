use crate::traits::{into_component::IntoComponent, load_asset::LoadAsset};
use bevy::{ecs::query::QueryFilter, prelude::*};
use std::path::Path;

impl<T> InsertAssetSystem for T {}

pub trait InsertAssetSystem {
	#[allow(clippy::type_complexity)]
	fn insert_asset<TAsset>(
		path: &'static Path,
	) -> impl Fn(Commands, Res<AssetServer>, Query<Entity, Self>)
	where
		TAsset: Asset,
		Handle<TAsset>: IntoComponent,
		Self: QueryFilter + Sized,
	{
		insert_asset_system::<TAsset, Self, AssetServer>(path)
	}
}

fn insert_asset_system<TAsset, TFilter, TAssetServer>(
	path: &'static Path,
) -> impl Fn(Commands, Res<TAssetServer>, Query<Entity, TFilter>)
where
	TAsset: Asset,
	Handle<TAsset>: IntoComponent,
	TFilter: QueryFilter,
	TAssetServer: Resource + LoadAsset,
{
	move |mut commands: Commands,
	      asset_server: Res<TAssetServer>,
	      entities: Query<Entity, TFilter>| {
		for entity in &entities {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let handle = asset_server.load_asset(path);
			entity.try_insert(handle.into_component());
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{new_handle, new_mock, test_tools::SingleThreaded};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::{automock, predicate::eq};
	use uuid::Uuid;

	#[derive(Component)]
	struct _Target;

	#[derive(Component, Debug, PartialEq)]
	struct _Component(Handle<_Asset>);

	#[derive(Asset, TypePath, Clone)]
	struct _Asset;

	impl IntoComponent for Handle<_Asset> {
		type TComponent = _Component;

		fn into_component(self) -> Self::TComponent {
			_Component(self)
		}
	}

	macro_rules! insert_asset_system {
		($path:expr) => {
			insert_asset_system::<_Asset, With<_Target>, _AssetServer>($path)
		};
	}

	#[derive(Resource)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	impl Default for _AssetServer {
		fn default() -> Self {
			let mut mock = Mock_AssetServer::default();
			mock.expect_load_asset()
				.return_const(Handle::<_Asset>::default());

			Self { mock }
		}
	}

	#[automock]
	impl LoadAsset for _AssetServer {
		fn load_asset<TAsset>(&self, path: &Path) -> Handle<TAsset>
		where
			TAsset: Asset,
		{
			self.mock.load_asset(path)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<_AssetServer>();

		app
	}

	#[test]
	fn insert_asset_into_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let path = Path::new("my/path");
		let entity = app.world_mut().spawn(_Target).id();
		let handle = new_handle!(_Asset);
		let mock = new_mock!(Mock_AssetServer, |mock: &mut Mock_AssetServer| {
			mock.expect_load_asset()
				.times(1)
				.with(eq(path))
				.return_const(handle.clone());
		});
		app.world_mut().resource_mut::<_AssetServer>().mock = mock;

		app.world_mut()
			.run_system_once(insert_asset_system!(path))?;

		assert_eq!(
			Some(&_Component(handle)),
			app.world().entity(entity).get::<_Component>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_run_on_filter_mismatch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let mock = new_mock!(Mock_AssetServer, |mock: &mut Mock_AssetServer| {
			mock.expect_load_asset()
				.never()
				.return_const(new_handle!(_Asset));
		});
		app.world_mut().resource_mut::<_AssetServer>().mock = mock;

		app.world_mut()
			.run_system_once(insert_asset_system!(Path::new("")))?;

		assert_eq!(None, app.world().entity(entity).get::<_Component>());
		Ok(())
	}
}
