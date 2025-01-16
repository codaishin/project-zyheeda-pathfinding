use bevy::prelude::*;
mod color_material;
mod mesh_2d;

pub trait IntoComponent
where
	Self: Sized,
{
	type TComponent: Component;

	fn into_component(self) -> Self::TComponent;
}
