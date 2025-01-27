use bevy::prelude::*;
use std::ops::DerefMut;

#[derive(Component, Debug, PartialEq)]
pub enum Despawn {
	NextFrame,
	AfterFrames(usize),
}

impl Despawn {
	pub fn system(mut commands: Commands, mut entities: Query<(Entity, &mut Self)>) {
		for (entity, mut despawn) in &mut entities {
			let despawn = despawn.deref_mut();
			let despawn = match despawn {
				Despawn::NextFrame => true,
				Despawn::AfterFrames(frames) => {
					*frames -= 1;
					*frames == 0
				}
			};

			if !despawn {
				continue;
			}

			let Some(entity) = commands.get_entity(entity) else {
				continue;
			};

			entity.despawn_recursive();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::SingleThreaded;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, Despawn::system);

		app
	}

	#[test]
	fn despawn() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Despawn::NextFrame).id();

		app.update();

		assert!(app.world().get_entity(entity).is_err());
	}

	#[test]
	fn do_not_component_missing() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert!(app.world().get_entity(entity).is_ok());
	}

	#[test]
	fn despawn_after_2_frames() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Despawn::AfterFrames(2)).id();
		let mut alive = [false; 2];

		app.update();

		alive[0] = app.world().get_entity(entity).is_ok();

		app.update();

		alive[1] = app.world().get_entity(entity).is_ok();

		assert_eq!([true, false], alive);
	}

	#[test]
	fn despawn_recursively() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Despawn::NextFrame).id();
		let child = app.world_mut().spawn_empty().set_parent(entity).id();

		app.update();

		assert!(app.world().get_entity(child).is_err());
	}
}
