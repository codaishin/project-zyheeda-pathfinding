use bevy::prelude::*;
use project_zyheeda_pathfinding::{
	asset_loader::CustomAssetLoader,
	assets::{collider_definition::ColliderDefinition, grid::Grid},
	components::{
		clickable::{Clickable, MouseLeft, MouseRight},
		compute_path_method::{straight_line_wide::StraightLineWide, ComputePathMethod},
		computed_path::{ComputedPath, PathNodeConnection},
		despawn::Despawn,
		grid_context::GridContext,
		player_camera::PlayerCamera,
		tile_collider::TileCollider,
		tile_grid::TileGrid,
		tile_type::{TileType, TileTypeValue},
		use_asset::UseAsset,
	},
	dtos::{grid_layout::GridLayout, mesh_definition::MeshDefinition, tile_color::TileColor},
	resources::mouse_world_position::MouseWorldPosition,
	states::path_placement::PathPlacement,
	systems::spawn::Spawn,
};

fn main() -> AppExit {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins)
		.init_state::<PathPlacement>()
		.init_asset::<Grid>()
		.init_asset::<ColliderDefinition>()
		.init_resource::<MouseWorldPosition>()
		.register_asset_loader(CustomAssetLoader::<Grid, GridLayout>::default())
		.register_asset_loader(CustomAssetLoader::<ColliderDefinition, MeshDefinition>::default())
		.register_asset_loader(CustomAssetLoader::<ColorMaterial, TileColor>::default())
		.register_asset_loader(CustomAssetLoader::<Mesh, MeshDefinition>::default())
		.add_systems(Startup, (PlayerCamera::spawn, TileGrid::spawn))
		.add_systems(Update, MouseWorldPosition::update_using::<PlayerCamera>)
		.add_systems(
			Update,
			(
				UseAsset::<Mesh>::insert,
				UseAsset::<Grid>::insert,
				UseAsset::<ColliderDefinition>::insert,
				UseAsset::<ColorMaterial>::insert.after(TileType::update_color),
			),
		)
		.add_systems(Update, Despawn::system)
		.add_systems(
			Update,
			(
				GridContext::<Grid>::spawn_tiles,
				GridContext::<Grid>::track_obstacles,
				ComputePathMethod::<Grid, StraightLineWide>::instantiate,
				ComputePathMethod::<Grid, StraightLineWide>::compute_path,
				ComputedPath::draw,
				PathNodeConnection::draw,
			)
				.chain(),
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
				PathPlacement::drag_on_hold::<MouseLeft>,
				PathPlacement::reset_on_release::<MouseLeft>,
				Clickable::<MouseRight>::toggle::<TileType>(TileTypeValue::Obstacle),
				Clickable::<MouseLeft>::switch_on_single::<TileType>(TileTypeValue::Start).run_if(
					in_state(PathPlacement::Start)
						.or(in_state(PathPlacement::Drag(Some(TileTypeValue::Start)))),
				),
				Clickable::<MouseLeft>::switch_on_single::<TileType>(TileTypeValue::End).run_if(
					in_state(PathPlacement::End)
						.or(in_state(PathPlacement::Drag(Some(TileTypeValue::End)))),
				),
				TileType::update_color,
			)
				.chain(),
		);

	app.run()
}
