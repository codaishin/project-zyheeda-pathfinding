use bevy::asset::LoadContext;

pub trait LoadFrom<TFrom> {
	fn load_from(from: TFrom, asset_server: &mut LoadContext) -> Self;
}
