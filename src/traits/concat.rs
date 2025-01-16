pub trait Concat {
	type TResult<T>;

	fn concat<T>(self, value: T) -> Self::TResult<T>;
}
