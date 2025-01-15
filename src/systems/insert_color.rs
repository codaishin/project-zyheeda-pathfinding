use bevy::{ecs::query::QueryFilter, prelude::*};
use uuid::Uuid;

impl<T> InsertColorSystem for T {}

pub trait InsertColorSystem {
	#[allow(clippy::type_complexity)]
	fn insert_color<T>(
		color: T,
	) -> impl Fn(Commands, ResMut<Assets<ColorMaterial>>, Query<Entity, Self>)
	where
		Color: From<T>,
		Self: QueryFilter + Sized,
	{
		let material = ColorMaterial::from_color(color);
		let id = AssetId::Uuid {
			uuid: Uuid::new_v4(),
		};

		move |mut commands, mut assets, entities| {
			for entity in &entities {
				let Some(mut entity) = commands.get_entity(entity) else {
					continue;
				};
				let handle = get_or_insert_material(&mut assets, id, &material);
				entity.try_insert(MeshMaterial2d(handle));
			}
		}
	}
}

fn get_or_insert_material(
	assets: &mut ResMut<Assets<ColorMaterial>>,
	id: AssetId<ColorMaterial>,
	material: &ColorMaterial,
) -> Handle<ColorMaterial> {
	assets.get_or_insert_with(id, || material.clone());

	let Some(handle) = assets.get_strong_handle(id) else {
		unreachable!();
	};

	handle
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		color::palettes::css::GREEN,
		ecs::system::{RunSystemError, RunSystemOnce},
	};

	#[derive(Component)]
	struct _Component;

	macro_rules! assert_count {
		($count:literal, $iter:expr) => {{
			let materials = $iter.collect::<Vec<_>>();
			let material_count = materials.len();

			match <[_; $count]>::try_from(materials) {
				Ok(materials) => materials,
				_ => panic!("expected {} items, got {}", $count, material_count),
			}
		}};
	}

	fn setup() -> App {
		let mut app = App::new();
		app.init_resource::<Assets<ColorMaterial>>();

		app
	}

	#[test]
	fn insert_color_material_into_entity() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Component).id();

		app.world_mut()
			.run_system_once(With::<_Component>::insert_color(Color::from(GREEN)))?;

		let assets = app.world().resource::<Assets<ColorMaterial>>();
		let [(id, ..)] = assert_count!(1, assets.iter());
		assert_eq!(
			Some(id),
			app.world()
				.entity(entity)
				.get::<MeshMaterial2d<ColorMaterial>>()
				.map(|MeshMaterial2d(handle)| handle.id()),
		);
		Ok(())
	}

	#[test]
	fn insert_color_material_into_assets() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Component);

		app.world_mut()
			.run_system_once(With::<_Component>::insert_color(Color::from(GREEN)))?;

		let assets = app.world().resource::<Assets<ColorMaterial>>();
		let [(.., asset)] = assert_count!(1, assets.iter());
		let expected = ColorMaterial::from_color(GREEN);
		assert_eq!(
			(expected.color, expected.alpha_mode, &expected.texture),
			(asset.color, asset.alpha_mode, &asset.texture)
		);
		Ok(())
	}

	#[test]
	fn insert_color_material_into_assets_only_once() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Component);
		app.world_mut().spawn(_Component);

		app.world_mut()
			.run_system_once(With::<_Component>::insert_color(Color::from(GREEN)))?;

		let assets = app.world().resource::<Assets<ColorMaterial>>();
		assert_count!(1, assets.iter());
		Ok(())
	}

	#[test]
	fn insert_color_material_into_assets_only_once_over_multiple_frames(
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Component);

		app.add_systems(Update, With::<_Component>::insert_color(Color::from(GREEN)));
		app.update();
		app.update();

		let assets = app.world().resource::<Assets<ColorMaterial>>();
		assert_count!(1, assets.iter());
		Ok(())
	}

	#[test]
	fn do_not_run_on_filter_mismatch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(With::<_Component>::insert_color(Color::from(GREEN)))?;

		assert_eq!(
			(None, 0),
			(
				app.world()
					.entity(entity)
					.get::<MeshMaterial2d<ColorMaterial>>()
					.map(|MeshMaterial2d(handle)| handle.id()),
				app.world()
					.resource::<Assets<ColorMaterial>>()
					.iter()
					.count()
			)
		);
		Ok(())
	}
}
