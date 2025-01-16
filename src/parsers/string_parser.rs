use super::parsed::{Parsed, ParsedNothing};
use crate::traits::{concat::Concat, parse::Parse};
use std::str::Chars;

#[derive(Debug)]
pub struct StringParser<'a, TParsed = ParsedNothing> {
	remaining: Chars<'a>,
	parsed: TParsed,
}

impl<'a> StringParser<'a, ParsedNothing> {
	pub fn new(chars: Chars<'a>) -> Self {
		Self {
			remaining: chars,
			parsed: ParsedNothing,
		}
	}

	pub fn parse<TFirst>(self) -> Result<StringParser<'a, Parsed<(TFirst,)>>, TFirst::TError>
	where
		TFirst: Parse<TRequiresParsed = ParsedNothing, TSource<'a> = Chars<'a>>,
	{
		let (first, remaining) = TFirst::parse(self.remaining)?;

		Ok(StringParser {
			remaining,
			parsed: self.parsed.concat(first),
		})
	}
}

type ConcatParsedResult<T, TNext> = <Parsed<T> as Concat>::TResult<TNext>;

impl<'a, T> StringParser<'a, Parsed<T>>
where
	Parsed<T>: Concat,
{
	pub fn parse<TNext>(
		self,
	) -> Result<StringParser<'a, ConcatParsedResult<T, TNext>>, TNext::TError>
	where
		TNext: Parse<TRequiresParsed = Parsed<T>, TSource<'a> = Chars<'a>>,
	{
		let (next, remaining) = TNext::parse(self.remaining)?;

		Ok(StringParser {
			remaining,
			parsed: self.parsed.concat(next),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::concat::Concat;

	impl<T> PartialEq for StringParser<'_, T>
	where
		T: PartialEq,
	{
		fn eq(&self, other: &Self) -> bool {
			if self.parsed != other.parsed {
				return false;
			}

			if self.remaining.clone().count() != other.remaining.clone().count() {
				return false;
			}

			let mut other_remaining = other.remaining.clone();
			for s in self.remaining.clone() {
				let o = other_remaining.next().unwrap();
				if s != o {
					return false;
				}
			}

			true
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Error;

	#[derive(Debug, PartialEq)]
	struct _First;

	impl Parse for _First {
		type TRequiresParsed = ParsedNothing;
		type TSource<'a> = Chars<'a>;
		type TError = _Error;

		fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
			let str = [(); 3].map(|_| value.next());
			match str.iter().filter_map(|c| *c).collect::<String>().as_str() {
				"fst" => Ok((_First, value)),
				_ => Err(_Error),
			}
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Second;

	impl Parse for _Second {
		type TRequiresParsed = Parsed<(_First,)>;
		type TSource<'a> = Chars<'a>;
		type TError = _Error;

		fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
			let str = [(); 3].map(|_| value.next());
			match str.iter().filter_map(|c| *c).collect::<String>().as_str() {
				"snd" => Ok((_Second, value)),
				_ => Err(_Error),
			}
		}
	}

	#[test]
	fn parse_first() {
		let source = "fst";
		let parser = StringParser::new(source.chars());

		let parser = parser.parse::<_First>();

		assert_eq!(
			Ok(StringParser {
				remaining: "".chars(),
				parsed: ParsedNothing.concat(_First)
			}),
			parser,
		)
	}

	#[test]
	fn parse_second() {
		let source = "snd";
		let parser = StringParser {
			remaining: source.chars(),
			parsed: ParsedNothing.concat(_First),
		};

		let parser = parser.parse::<_Second>();

		assert_eq!(
			Ok(StringParser {
				remaining: "".chars(),
				parsed: ParsedNothing.concat(_First).concat(_Second)
			}),
			parser,
		)
	}
}
