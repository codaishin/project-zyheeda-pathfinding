use bevy::prelude::*;
use project_zyheeda_pathfinding::{
	asset_loader::CustomAssetLoader,
	components::{
		player_camera::PlayerCamera,
		tile::{Grid, Tile},
	},
	dtos::{tile_color::TileColor, tile_size::TileSize},
	systems::insert_asset::InsertAssetSystem,
};
use std::path::Path;

fn main() -> AppExit {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins)
		.register_asset_loader(CustomAssetLoader::<ColorMaterial, TileColor>::default())
		.register_asset_loader(CustomAssetLoader::<Mesh, TileSize>::default())
		.add_systems(
			Startup,
			(
				PlayerCamera::spawn,
				Tile::spawn_in(Grid {
					height: 10,
					width: 10,
					scale: 50.,
				}),
			),
		)
		.add_systems(
			Update,
			(
				Added::<Tile>::insert_asset::<ColorMaterial>(Path::new("tile.json")),
				Added::<Tile>::insert_asset::<Mesh>(Path::new("tile.json")),
			),
		);

	app.run()
}
