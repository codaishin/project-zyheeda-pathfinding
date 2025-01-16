use bevy::prelude::*;
use project_zyheeda_pathfinding::{
	asset_loader::CustomAssetLoader,
	assets::grid::Grid,
	components::{player_camera::PlayerCamera, tile::Tile},
	dtos::{grid_layout::GridLayout, tile_color::TileColor, tile_size::TileSize},
	systems::{insert_asset::InsertAssetSystem, load::Load, spawn_grid::SpawnComponents},
};
use std::path::Path;

fn main() -> AppExit {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins)
		.init_asset::<Grid>()
		.register_asset_loader(CustomAssetLoader::<Grid, GridLayout>::default())
		.register_asset_loader(CustomAssetLoader::<ColorMaterial, TileColor>::default())
		.register_asset_loader(CustomAssetLoader::<Mesh, TileSize>::default())
		.add_systems(
			Startup,
			(PlayerCamera::spawn, Grid::load_from(Path::new("grid.json"))),
		)
		.add_systems(
			Update,
			(
				Grid::spawn::<Tile>,
				Added::<Tile>::insert_asset::<ColorMaterial>(Path::new("tile.json")),
				Added::<Tile>::insert_asset::<Mesh>(Path::new("tile.json")),
			)
				.chain(),
		);

	app.run()
}
