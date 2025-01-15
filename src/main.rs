use bevy::prelude::*;
use project_zyheeda_pathfinding::components::tile::{Grid, Tile};

fn main() -> AppExit {
	let mut app = App::new();

	app.add_plugins(DefaultPlugins).add_systems(
		Startup,
		Tile::spawn_in(Grid {
			height: 10,
			width: 10,
		}),
	);

	app.run()
}
