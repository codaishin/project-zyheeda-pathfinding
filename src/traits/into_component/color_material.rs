use super::IntoComponent;
use bevy::prelude::*;

impl IntoComponent for Handle<ColorMaterial> {
	type TComponent = MeshMaterial2d<ColorMaterial>;

	fn into_component(self) -> Self::TComponent {
		MeshMaterial2d::<ColorMaterial>(self)
	}
}
