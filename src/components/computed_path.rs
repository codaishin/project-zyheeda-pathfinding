use std::path::Path;

use bevy::prelude::*;

use super::use_asset::UseAsset;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub struct ComputedPath(pub Vec<Vec3>);

impl ComputedPath {
	pub fn draw(mut commands: Commands, paths: Query<(Entity, &Self), Changed<Self>>) {
		for (entity, Self(path)) in &paths {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_descendants();

			for (i, translation) in path.iter().cloned().enumerate() {
				entity.with_child((PathNode(i), Transform::from_translation(translation)));
			}
		}
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Self::asset),
	UseAsset<ColorMaterial>(Self::asset)
)]
pub struct PathNode(usize);

impl PathNode {
	fn asset<TAsset>() -> UseAsset<TAsset>
	where
		TAsset: Asset,
	{
		UseAsset::new(Path::new("path_node.json"))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_count, test_tools::SingleThreaded};

	fn is<TComponent>(entity: &EntityRef) -> bool
	where
		TComponent: Component,
	{
		entity.contains::<TComponent>()
	}

	fn child_of(entity: Entity) -> impl Fn(&EntityRef) -> bool {
		move |child| {
			child
				.get::<Parent>()
				.map(|p| p.get() == entity)
				.unwrap_or(false)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, ComputedPath::draw);

		app
	}

	#[test]
	fn spawn_path_nodes() {
		let mut app = setup();
		app.world_mut().spawn(ComputedPath(vec![
			Vec3::new(1., 2., 3.),
			Vec3::new(3., 4., 5.),
		]));

		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_eq!(
			[
				(Some(&PathNode(0)), Some(Vec3::new(1., 2., 3.))),
				(Some(&PathNode(1)), Some(Vec3::new(3., 4., 5.))),
			],
			nodes.map(|e| (
				e.get::<PathNode>(),
				e.get::<Transform>().map(|t| t.translation)
			))
		);
	}

	#[test]
	fn spawn_path_nodes_as_children() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(ComputedPath(vec![
				Vec3::new(1., 2., 3.),
				Vec3::new(3., 4., 5.),
			]))
			.id();

		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_count!(2, nodes.into_iter().filter(child_of(entity)));
	}

	#[test]
	fn spawn_path_nodes_only_once() {
		let mut app = setup();
		app.world_mut().spawn(ComputedPath(vec![
			Vec3::new(1., 2., 3.),
			Vec3::new(3., 4., 5.),
		]));

		app.update();
		app.update();

		assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
	}

	#[test]
	fn replace_nodes_when_path_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(ComputedPath(vec![
				Vec3::new(1., 2., 3.),
				Vec3::new(3., 4., 5.),
			]))
			.id();

		app.update();
		let mut path = app.world_mut().entity_mut(entity);
		let mut path = path.get_mut::<ComputedPath>().unwrap();
		*path = ComputedPath(vec![Vec3::new(15., 25., 35.), Vec3::new(35., 45., 55.)]);
		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_eq!(
			[
				(Some(&PathNode(0)), Some(Vec3::new(15., 25., 35.))),
				(Some(&PathNode(1)), Some(Vec3::new(35., 45., 55.))),
			],
			nodes.map(|e| (
				e.get::<PathNode>(),
				e.get::<Transform>().map(|t| t.translation)
			))
		);
	}
}
