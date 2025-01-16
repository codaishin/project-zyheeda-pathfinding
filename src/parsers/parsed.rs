use crate::traits::concat::Concat;

#[derive(Debug, PartialEq)]
pub struct Parsed<T>(T);

impl<T> Parsed<T> {
	pub fn unpack(self) -> T {
		let Parsed(parsed) = self;

		parsed
	}
}

#[derive(Debug, PartialEq)]
pub struct ParsedNothing;

impl Concat for ParsedNothing {
	type TResult<T> = Parsed<(T,)>;

	fn concat<T>(self, value: T) -> Self::TResult<T> {
		Parsed((value,))
	}
}

macro_rules! impl_concat_recursively {
	($name:ident, $($names:ident),+) => {
		impl_concat_recursively!($($names),*);

		impl<$name, $($names),*> Concat for Parsed<($name, $($names,)*)> {
			type TResult<T> = Parsed<($name, $($names),*, T)>;

			fn concat<T>(self, value: T) -> Self::TResult<T> {
				#[allow(non_snake_case)]  // type names are used as variable names here, thus clippy is mad
				let Parsed(($name, $($names,)*)) = self;
				Parsed(($name, $($names,)* value))
			}
		}
	};
	($name:ident) => {
		impl<$name> Concat for Parsed<($name,)> {
			type TResult<T> = Parsed<($name, T)>;

			fn concat<T>(self, value: T) -> Self::TResult<T> {
				#[allow(non_snake_case)]  // type names are used as variable names here, thus clippy is mad
				let Parsed(($name,)) = self;
				Parsed(($name, value))
			}
		}
	};
}

impl_concat_recursively!(T1, T2, T3, T4, T5, T6);
