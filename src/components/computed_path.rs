use super::use_asset::UseAsset;
use bevy::prelude::*;
use std::path::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Transform, Visibility)]
pub struct ComputedPath {
	pub path: Vec<Vec3>,
	pub draw_connections: bool,
}

impl ComputedPath {
	pub fn draw(mut commands: Commands, paths: Query<(Entity, &Self), Changed<Self>>) {
		let mut previous = None;
		for (entity, computed) in &paths {
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			entity.despawn_descendants();

			for translation in computed.path.iter().cloned() {
				entity.with_children(|parent| {
					previous = Some(
						parent
							.spawn((
								PathNode {
									previous,
									draw_connection: computed.draw_connections,
								},
								Transform::from_translation(translation),
							))
							.id(),
					);
				});
			}
		}
	}
}

#[derive(Component, Debug, PartialEq, Default)]
#[require(
	Transform,
	Visibility,
	UseAsset<Mesh>(Self::asset),
	UseAsset<ColorMaterial>(Self::asset)
)]
pub struct PathNode {
	previous: Option<Entity>,
	draw_connection: bool,
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
		for (entity, node) in &nodes {
			if !node.draw_connection {
				continue;
			}

			let Some(previous) = node.previous else {
				continue;
			};
			let Ok(pos) = transforms.get(entity) else {
				continue;
			};
			let Ok(pos_previous) = transforms.get(previous) else {
				continue;
			};
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			let offset = pos_previous.translation - pos.translation;
			let length = offset.length();

			entity.with_child((
				PathNodeConnection,
				Transform::from_translation(offset / 2.)
					.looking_to(Vec3::Z, offset)
					.with_scale(Vec3::new(1., length, 1.)),
			));
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
		app.world_mut().spawn(ComputedPath {
			path: vec![Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)],
			..default()
		});

		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_eq!(
			[
				(
					Some(&PathNode {
						previous: None,
						..default()
					}),
					Some(Vec3::new(1., 2., 3.))
				),
				(
					Some(&PathNode {
						previous: Some(nodes[0].id()),
						..default()
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
			.spawn(ComputedPath {
				path: vec![Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)],
				..default()
			})
			.id();

		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_count!(2, nodes.into_iter().filter(child_of(entity)));
	}

	#[test]
	fn spawn_path_nodes_only_once() {
		let mut app = setup();
		app.world_mut().spawn(ComputedPath {
			path: vec![Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)],
			..default()
		});

		app.update();
		app.update();

		assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
	}

	#[test]
	fn replace_nodes_when_path_changed() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(ComputedPath {
				path: vec![Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)],
				..default()
			})
			.id();

		app.update();
		let mut path = app.world_mut().entity_mut(entity);
		let mut path = path.get_mut::<ComputedPath>().unwrap();
		*path = ComputedPath {
			path: vec![Vec3::new(15., 25., 35.), Vec3::new(35., 45., 55.)],
			..default()
		};
		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_eq!(
			[
				(
					Some(&PathNode {
						previous: None,
						..default()
					}),
					Some(Vec3::new(15., 25., 35.))
				),
				(
					Some(&PathNode {
						previous: Some(nodes[0].id()),
						..default()
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

	#[test]
	fn spawn_path_nodes_without_connections() {
		let mut app = setup();
		app.world_mut().spawn(ComputedPath {
			path: vec![Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.)],
			draw_connections: true,
		});

		app.update();

		let nodes = assert_count!(2, app.world().iter_entities().filter(is::<PathNode>));
		assert_eq!(
			[
				(
					Some(&PathNode {
						previous: None,
						draw_connection: true,
					}),
					Some(Vec3::new(1., 2., 3.))
				),
				(
					Some(&PathNode {
						previous: Some(nodes[0].id()),
						draw_connection: true,
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
			.spawn((
				PathNode {
					previous: None,
					draw_connection: true,
				},
				Transform::default(),
			))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
					draw_connection: true,
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
			.spawn((
				PathNode {
					previous: None,
					draw_connection: true,
				},
				Transform::default(),
			))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
					draw_connection: true,
				},
				Transform::default(),
			))
			.id();

		app.update();
		app.update();

		assert_count!(1, app.world().iter_entities().filter(child_of(node_b)));
	}

	#[test]
	fn spawn_connection_between_nodes_relative_and_rotated() {
		let mut app = setup();
		let node_a = app
			.world_mut()
			.spawn((
				PathNode {
					previous: None,
					draw_connection: true,
				},
				Transform::from_xyz(1., 1., 0.),
			))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
					draw_connection: true,
				},
				Transform::from_xyz(2., 1., 0.),
			))
			.id();

		app.update();

		let [connection] = assert_count!(1, app.world().iter_entities().filter(child_of(node_b)));
		assert_eq!(
			Some(&Transform::from_xyz(-0.5, 0., 0.).looking_to(Vec3::Z, Vec3::new(-1., 0., 0.))),
			connection.get::<Transform>(),
		);
	}

	#[test]
	fn spawn_connection_between_nodes_relative_rotated_and_stretched() {
		let mut app = setup();
		let node_a = app
			.world_mut()
			.spawn((
				PathNode {
					previous: None,
					draw_connection: true,
				},
				Transform::from_xyz(1., 1., 0.),
			))
			.id();
		let node_b = app
			.world_mut()
			.spawn((
				PathNode {
					previous: Some(node_a),
					draw_connection: true,
				},
				Transform::from_xyz(5., 1., 0.),
			))
			.id();

		app.update();

		let [connection] = assert_count!(1, app.world().iter_entities().filter(child_of(node_b)));
		assert_eq!(
			Some(
				&Transform::from_xyz(-2., 0., 0.)
					.looking_to(Vec3::Z, Vec3::new(-1., 0., 0.))
					.with_scale(Vec3::new(1., 4., 1.))
			),
			connection.get::<Transform>(),
		);
	}

	#[test]
	fn do_not_spawn_connection_when_draw_connection_false() {
		let mut app = setup();
		let node_a = app
			.world_mut()
			.spawn((
				PathNode {
					previous: None,
					draw_connection: false,
				},
				Transform::default(),
			))
			.id();
		app.world_mut().spawn((
			PathNode {
				previous: Some(node_a),
				draw_connection: false,
			},
			Transform::default(),
		));

		app.update();

		let entities = app.world().iter_entities();
		assert_count!(0, entities.filter(is::<PathNodeConnection>));
	}
}
