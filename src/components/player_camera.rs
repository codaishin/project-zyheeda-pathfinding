use bevy::prelude::*;

#[derive(Component)]
#[require(Camera2d)]
pub struct PlayerCamera;

impl PlayerCamera {
	pub fn spawn(mut commands: Commands) {
		commands.spawn(PlayerCamera);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assert_count;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_camera() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(PlayerCamera::spawn)?;

		assert_count!(
			1,
			app.world()
				.iter_entities()
				.filter_map(|e| e.get::<PlayerCamera>())
		);
		Ok(())
	}
}
