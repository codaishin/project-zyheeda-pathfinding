use bevy::{color::palettes::css::GRAY, prelude::*};
use project_zyheeda_pathfinding::{
	components::{
		player_camera::PlayerCamera,
		tile::{Grid, Tile},
	},
	systems::insert_asset::InsertAssetSystem,
};

fn main() -> AppExit {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins)
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
				Added::<Tile>::insert_asset(ColorMaterial::from_color(GRAY)),
				Added::<Tile>::insert_asset(Mesh::from(Rectangle::new(45., 45.))),
			),
		);

	app.run()
}
