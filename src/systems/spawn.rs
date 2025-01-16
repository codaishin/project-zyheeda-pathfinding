use bevy::prelude::*;

impl<T> Spawn for T {}

pub trait Spawn {
	fn spawn(mut commands: Commands)
	where
		Self: Component + Default,
	{
		commands.spawn(Self::default());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assert_count;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Component;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn spawn_component() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(_Component::spawn)?;

		let [entity] = assert_count!(1, app.world().iter_entities());
		assert_eq!(Some(&_Component), entity.get::<_Component>());
		Ok(())
	}
}
