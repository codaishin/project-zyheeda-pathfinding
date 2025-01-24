use super::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq)]
#[require(Transform, Visibility)]
pub struct ComputedPath(pub Vec<Vec3>);

impl ComputedPath {
	pub fn draw(mut commands: Commands, paths: Query<(Entity, &Self), Changed<Self>>) {
		let mut previous = None;
		for (entity, Self(path)) in &paths {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_descendants();

			for translation in path.iter().cloned() {
				entity.with_children(|parent| {
					previous = Some(
						parent
							.spawn((
								PathNode { previous },
								Transform::from_translation(translation),
							))
							.id(),
					);
				});
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
pub struct PathNode {
	previous: Option<Entity>,
}

impl PathNode {
	fn asset<TAsset>() -> UseAsset<TAsset>
	where
		TAsset: Asset,
	{
		UseAsset::new(Path::new("path_node.json"))
	}
}

#[derive(Component, Debug, PartialEq)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Self::asset),
	UseAsset<ColorMaterial>(PathNode::asset)
)]
pub struct PathNodeConnection;

impl PathNodeConnection {
	fn asset<TAsset>() -> UseAsset<TAsset>
	where
		TAsset: Asset,
	{
		UseAsset::new(Path::new("path_node_connection.json"))
	}

	pub fn draw(
		mut commands: Commands,
		nodes: Query<(Entity, &PathNode), Added<PathNode>>,
		transforms: Query<&Transform>,
	) {
		for (entity, PathNode { previous }) in &nodes {
			let Some(previous) = previous else {
				continue;
			};
			let Ok(pos) = transforms.get(entity) else {
				continue;
			};
			let Ok(pos_previous) = transforms.get(*previous) else {
				continue;
			};
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let pos = (pos_previous.translation - pos.translation) / 2.;

			entity.with_child((PathNodeConnection, Transform::from_translation(pos)));
		}
	}
}

#[cfg(test)]
mod test_draw_path_nodes {
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
				(
					Some(&PathNode { previous: None }),
					Some(Vec3::new(1., 2., 3.))
				),
				(
					Some(&PathNode {
						previous: Some(nodes[0].id())
					}),
					Some(Vec3::new(3., 4., 5.))
				),
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
				(
					Some(&PathNode { previous: None }),
					Some(Vec3::new(15., 25., 35.))
				),
				(
					Some(&PathNode {
						previous: Some(nodes[0].id())
					}),
					Some(Vec3::new(35., 45., 55.))
				),
			],
			nodes.map(|e| (
				e.get::<PathNode>(),
				e.get::<Transform>().map(|t| t.translation)
			))
		);
	}
}

#[cfg(test)]
mod test_draw_node_connection {
	use super::*;
	use crate::{assert_count, test_tools::SingleThreaded};

	fn child_of(entity: Entity) -> impl Fn(&EntityRef) -> bool {
		move |child| {
			child
				.get::<Parent>()
				.map(|p| p.get() == entity)
				.unwrap_or(false)
		}
	}

	fn is<TComponent>(entity: &EntityRef) -> bool
	where
		TComponent: Component,
	{
		entity.contains::<TComponent>()
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, PathNodeConnection::draw);

		app
	}

	#[test]
	fn spawn_connection_as_child_of_node() {
		let mut app = setup();
		let node_a = app
			.world_mut()
			.spawn((PathNode { previous: None }, Transform::default()))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
				},
				Transform::default(),
			))
			.id();

		app.update();

		let entities = app.world().iter_entities();
		let connections = assert_count!(1, entities.filter(is::<PathNodeConnection>));
		assert_count!(1, connections.into_iter().filter(child_of(node_b)));
	}

	#[test]
	fn do_not_spawn_connection_as_child_of_non_node() {
		let mut app = setup();
		let node_b = app.world_mut().spawn(Transform::default()).id();

		app.update();

		assert_count!(0, app.world().iter_entities().filter(child_of(node_b)));
	}

	#[test]
	fn spawn_connection_as_child_of_node_only_once() {
		let mut app = setup();
		let node_a = app
			.world_mut()
			.spawn((PathNode { previous: None }, Transform::default()))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
				},
				Transform::default(),
			))
			.id();

		app.update();
		app.update();

		assert_count!(1, app.world().iter_entities().filter(child_of(node_b)));
	}

	#[test]
	fn spawn_connection_between_nodes() {
		let mut app = setup();
		let node_a = app
			.world_mut()
			.spawn((PathNode { previous: None }, Transform::from_xyz(1., 1., 1.)))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
				},
				Transform::from_xyz(5., 5., 5.),
			))
			.id();

		app.update();

		let [connection] = assert_count!(1, app.world().iter_entities().filter(child_of(node_b)));
		assert_eq!(
			Some(&Transform::from_xyz(2., 2., 2.)),
			connection.get::<Transform>(),
		);
	}
}
