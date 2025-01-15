#[macro_export]
macro_rules! assert_count {
	($expected_count:literal, $iter:expr) => {{
		let items = $iter.collect::<Vec<_>>();
		let items_count = items.len();

		match <[_; $expected_count]>::try_from(items) {
			Ok(items) => items,
			Err(_) => panic!(
				"\x1b[31mCount assertion failed:\nexpected: {expected}\n     got: {got}\x1b[0m",
				expected = $expected_count,
				got = items_count
			),
		}
	}};
}
