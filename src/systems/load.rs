use crate::{resources::loaded_asset::LoadedAsset, traits::load_asset::LoadAsset};
use bevy::prelude::*;
use std::path::Path;

impl<T> Load for T {}

pub trait Load {
	fn load_from(path: &'static Path) -> impl Fn(Commands, Res<AssetServer>)
	where
		Self: Asset + Sized,
	{
		load_from::<Self, AssetServer>(path)
	}
}

fn load_from<TAsset, TAssetServer>(path: &'static Path) -> impl Fn(Commands, Res<TAssetServer>)
where
	TAsset: Asset,
	TAssetServer: LoadAsset + Resource,
{
	|mut commands, asset_server| {
		let handle = asset_server.load_asset::<TAsset>(path);
		commands.insert_resource(LoadedAsset(handle));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{new_handle, new_mock, resources::loaded_asset::LoadedAsset};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use mockall::{automock, predicate::eq};

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset;

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
		let mut app = App::new();
		app.init_resource::<_AssetServer>();

		app
	}

	#[test]
	fn load_from_path() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().resource_mut::<_AssetServer>().mock =
			new_mock!(Mock_AssetServer, |mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<_Asset>()
					.times(1)
					.with(eq(Path::new("my/path")))
					.return_const(Handle::default());
			});

		app.world_mut()
			.run_system_once(load_from::<_Asset, _AssetServer>(Path::new("my/path")))?;

		Ok(())
	}

	#[test]
	fn store_handle() -> Result<(), RunSystemError> {
		let handle = new_handle!(_Asset);
		let mut app = setup();
		app.world_mut().resource_mut::<_AssetServer>().mock =
			new_mock!(Mock_AssetServer, |mock: &mut Mock_AssetServer| {
				mock.expect_load_asset::<_Asset>()
					.return_const(handle.clone());
			});

		app.world_mut()
			.run_system_once(load_from::<_Asset, _AssetServer>(Path::new("my/path")))?;

		assert_eq!(
			Some(&LoadedAsset(handle)),
			app.world().get_resource::<LoadedAsset<_Asset>>(),
		);
		Ok(())
	}
}
