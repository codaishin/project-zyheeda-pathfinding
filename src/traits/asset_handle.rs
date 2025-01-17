use bevy::prelude::*;

pub trait AssetHandle {
	type TAsset: Asset;

	fn get_handle(&self) -> &Handle<Self::TAsset>;
}
