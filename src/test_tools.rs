use bevy::{
	ecs::schedule::{ExecutorKind, ScheduleLabel},
	prelude::*,
};

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

#[macro_export]
macro_rules! new_handle {
	($ty:ty) => {
		Handle::<$ty>::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	};
}

#[macro_export]
macro_rules! new_mock {
	($ty:ty, $setup:expr) => {{
		let mut mock = <$ty>::default();
		$setup(&mut mock);
		mock
	}};
}

pub trait SingleThreaded {
	fn single_threaded(self, label: impl ScheduleLabel) -> Self;
}

impl SingleThreaded for App {
	fn single_threaded(mut self, label: impl ScheduleLabel) -> Self {
		self.edit_schedule(label, |schedule| {
			schedule.set_executor_kind(ExecutorKind::SingleThreaded);
		});

		self
	}
}
