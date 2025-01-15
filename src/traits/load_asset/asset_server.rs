use super::LoadAsset;
use bevy::prelude::*;
use std::path::Path;

impl LoadAsset for AssetServer {
	fn load_asset<TAsset>(&self, path: &Path) -> Handle<TAsset>
	where
		TAsset: Asset,
	{
		self.load(path)
	}
}
