use super::IntoComponent;
use bevy::prelude::*;

impl IntoComponent for Handle<Mesh> {
	type TComponent = Mesh2d;

	fn into_component(self) -> Self::TComponent {
		Mesh2d(self)
	}
}
