use bevy::prelude::*;
use project_zyheeda_pathfinding::{
	asset_loader::CustomAssetLoader,
	assets::{grid::Grid, tile_collider_definition::TileColliderDefinition},
	components::{
		clickable::{Clickable, MouseLeft, MouseRight},
		player_camera::PlayerCamera,
		tile_builder::TileBuilder,
		tile_collider::TileCollider,
		tile_grid::TileGrid,
		tile_type::TileType,
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
		.add_systems(Update, TileBuilder::<Grid>::spawn_tiles)
		.add_systems(
			Update,
			(
				UseAsset::<Mesh>::insert,
				UseAsset::<Grid>::insert,
				UseAsset::<TileColliderDefinition>::insert,
				UseAsset::<ColorMaterial>::insert.after(TileType::update_color),
			),
		)
		.add_systems(
			Update,
			(
				Clickable::<MouseLeft>::detect_click_on::<TileCollider>,
				Clickable::<MouseRight>::detect_click_on::<TileCollider>,
			),
		)
		.add_systems(
			Update,
			(
				Clickable::<MouseRight>::toggle(TileType::Obstacle),
				Clickable::<MouseLeft>::toggle(TileType::Start),
				TileType::update_color,
			)
				.chain(),
		);

	app.run()
}
