use bevy::prelude::*;

#[derive(Resource, Debug, PartialEq)]
pub struct LoadedAsset<T>(pub Handle<T>)
where
	T: Asset;
