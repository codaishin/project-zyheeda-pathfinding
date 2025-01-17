use crate::{
	components::tile_collider::TileCollider,
	traits::{
		into_component::IntoComponent,
		is_point_hit::{IsPointHit, Relative},
	},
};
use bevy::prelude::*;

#[derive(Asset, TypePath, Debug, PartialEq)]
pub struct TileColliderDefinition {
	pub half_height: f32,
	pub half_width: f32,
}

impl IntoComponent for Handle<TileColliderDefinition> {
	type TComponent = TileCollider;

	fn into_component(self) -> Self::TComponent {
		TileCollider(self)
	}
}

impl IsPointHit for TileColliderDefinition {
	fn is_point_hit(&self, position: Relative) -> bool {
		let Vec2 { x, y } = *position;
		x.abs() <= self.half_width && y.abs() <= self.half_height
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_hit_true() {
		let tile = TileColliderDefinition {
			half_width: 5.,
			half_height: 3.,
		};

		assert!(tile.is_point_hit(Relative::new(Vec2::new(4., 2.))));
	}

	#[test]
	fn is_not_hit_false_when_x_greater_half_width() {
		let tile = TileColliderDefinition {
			half_width: 5.,
			half_height: 3.,
		};

		assert!(!tile.is_point_hit(Relative::new(Vec2::new(6., 2.))));
	}

	#[test]
	fn is_not_hit_false_when_abs_x_greater_half_width() {
		let tile = TileColliderDefinition {
			half_width: 5.,
			half_height: 3.,
		};

		assert!(!tile.is_point_hit(Relative::new(Vec2::new(-6., 2.))));
	}

	#[test]
	fn is_not_hit_false_when_y_greater_half_height() {
		let tile = TileColliderDefinition {
			half_width: 5.,
			half_height: 3.,
		};

		assert!(!tile.is_point_hit(Relative::new(Vec2::new(1., 4.))));
	}

	#[test]
	fn is_not_hit_false_when_abs_y_greater_half_height() {
		let tile = TileColliderDefinition {
			half_width: 5.,
			half_height: 3.,
		};

		assert!(!tile.is_point_hit(Relative::new(Vec2::new(1., -4.))));
	}

	#[test]
	fn is_hit_when_rel_position_on_tile_border() {
		let tile = TileColliderDefinition {
			half_width: 5.,
			half_height: 3.,
		};

		assert!(tile.is_point_hit(Relative::new(Vec2::new(5., 3.))));
	}
}
