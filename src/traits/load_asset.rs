mod asset_server;
use bevy::prelude::*;
use std::path::Path;

pub trait LoadAsset {
	fn load_asset<TAsset>(&self, path: &Path) -> Handle<TAsset>
	where
		TAsset: Asset;
}
