use bevy::math::Vec3;

pub trait Translations {
	type TIter<'a>: Iterator<Item = Vec3>
	where
		Self: 'a;

	fn translations(&self) -> Self::TIter<'_>;
}
