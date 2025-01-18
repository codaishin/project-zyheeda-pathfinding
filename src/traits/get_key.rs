pub trait GetKey {
	type TKey;

	fn get_key() -> Self::TKey;
}
