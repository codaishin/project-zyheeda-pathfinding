use crate::traits::{into_component::IntoComponent, load_asset::LoadAsset};
use bevy::prelude::*;
use std::{marker::PhantomData, path::Path};

#[derive(Component, Debug, Clone)]
pub struct UseAsset<TAsset, TAssetServer = AssetServer>
where
	TAsset: Asset,
	TAssetServer: LoadAsset + Resource,
{
	path: &'static Path,
	_a: PhantomData<(TAsset, TAssetServer)>,
}

impl<TAsset, TAssetServer> UseAsset<TAsset, TAssetServer>
where
	TAsset: Asset,
	TAssetServer: LoadAsset + Resource,
{
	pub fn new(path: &'static Path) -> Self {
		Self {
			path,
			_a: PhantomData,
		}
	}

	pub fn insert_system(
		mut commands: Commands,
		asset_server: Res<TAssetServer>,
		entities: Query<(Entity, &Self), Changed<Self>>,
	) where
		Handle<TAsset>: IntoComponent,
	{
		for (entity, UseAsset { path, .. }) in &entities {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let component = asset_server.load_asset::<TAsset>(path).into_component();
			entity.try_insert(component);
		}
	}
}

impl<TAsset> PartialEq for UseAsset<TAsset>
where
	TAsset: Asset,
{
	fn eq(&self, other: &Self) -> bool {
		self.path == other.path
	}
}

#[cfg(test)]
mod tests {
	use std::ops::DerefMut;

	use super::*;
	use crate::{new_handle, new_mock, test_tools::SingleThreaded, traits::load_asset::LoadAsset};
	use mockall::{automock, predicate::eq};

	#[derive(Asset, TypePath)]
	struct _Asset;

	impl IntoComponent for Handle<_Asset> {
		type TComponent = _Component;

		fn into_component(self) -> Self::TComponent {
			_Component(self)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Component(Handle<_Asset>);

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

	type _UseAsset = UseAsset<_Asset, _AssetServer>;

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
		app.add_systems(Update, _UseAsset::insert_system);

		app
	}

	#[test]
	fn load_asset() {
		let mut app = setup();
		app.world_mut().spawn(_UseAsset::new(Path::new("my/path")));

		app.world_mut().resource_mut::<_AssetServer>().mock = new_mock!(Mock_AssetServer, |mock| {
			mock.expect_load_asset::<_Asset>()
				.times(1)
				.with(eq(Path::new("my/path")))
				.return_const(new_handle!(_Asset));
		});

		app.update();
	}

	#[test]
	fn insert_component() {
		let mut app = setup();
		let handle = new_handle!(_Asset);
		let entity = app
			.world_mut()
			.spawn(_UseAsset::new(Path::new("my/path")))
			.id();
		app.world_mut().resource_mut::<_AssetServer>().mock = new_mock!(Mock_AssetServer, |mock| {
			mock.expect_load_asset::<_Asset>()
				.return_const(handle.clone());
		});

		app.update();

		assert_eq!(
			Some(&_Component(handle)),
			app.world().entity(entity).get::<_Component>(),
		);
	}

	#[test]
	fn load_only_once() {
		let mut app = setup();
		app.world_mut().spawn(_UseAsset::new(Path::new("my/path")));

		app.world_mut().resource_mut::<_AssetServer>().mock = new_mock!(Mock_AssetServer, |mock| {
			mock.expect_load_asset::<_Asset>()
				.times(1)
				.return_const(new_handle!(_Asset));
		});

		app.update();
		app.update();
	}

	#[test]
	fn load_again_after_mut_deref() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_UseAsset::new(Path::new("my/path")))
			.id();

		app.world_mut().resource_mut::<_AssetServer>().mock = new_mock!(Mock_AssetServer, |mock| {
			mock.expect_load_asset::<_Asset>()
				.times(2)
				.return_const(new_handle!(_Asset));
		});

		app.update();
		let mut use_asset = app.world_mut().entity_mut(entity);
		let mut use_asset = use_asset.get_mut::<_UseAsset>().unwrap();
		use_asset.deref_mut();
		app.update();
	}
}
