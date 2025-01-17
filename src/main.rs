use bevy::prelude::*;
use project_zyheeda_pathfinding::{
	asset_loader::CustomAssetLoader,
	assets::{grid::Grid, tile_collider_definition::TileColliderDefinition},
	components::{
		player_camera::PlayerCamera,
		tile_builder::TileBuilder,
		tile_grid::TileGrid,
		use_asset::UseAsset,
	},
	dtos::{grid_layout::GridLayout, tile_color::TileColor, tile_size::TileSize},
	resources::mouse_world_position::MouseWorldPosition,
	systems::spawn::Spawn,
};

fn main() -> AppExit {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins)
		.init_asset::<Grid>()
		.init_asset::<TileColliderDefinition>()
		.init_resource::<MouseWorldPosition>()
		.register_asset_loader(CustomAssetLoader::<Grid, GridLayout>::default())
		.register_asset_loader(CustomAssetLoader::<TileColliderDefinition, TileSize>::default())
		.register_asset_loader(CustomAssetLoader::<ColorMaterial, TileColor>::default())
		.register_asset_loader(CustomAssetLoader::<Mesh, TileSize>::default())
		.add_systems(Startup, (PlayerCamera::spawn, TileGrid::spawn))
		.add_systems(Update, MouseWorldPosition::update_using::<PlayerCamera>)
		.add_systems(
			Update,
			(
				UseAsset::<ColorMaterial>::insert_system,
				UseAsset::<Mesh>::insert_system,
				UseAsset::<Grid>::insert_system,
				UseAsset::<TileColliderDefinition>::insert_system,
			),
		)
		.add_systems(Update, TileBuilder::<Grid>::spawn_tiles);

	app.run()
}
