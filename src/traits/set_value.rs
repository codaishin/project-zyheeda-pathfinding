pub trait SetValue {
	type TValue;

	fn set_value(&mut self, value: Self::TValue);
}
