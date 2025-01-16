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
		TFirst: Parse<TRequired = ParsedNothing, TSource<'a> = Chars<'a>>,
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
	pub fn unpack(self) -> T {
		let StringParser { parsed, .. } = self;

		parsed.unpack()
	}

	pub fn parse<TNext>(
		self,
	) -> Result<StringParser<'a, ConcatParsedResult<T, TNext>>, TNext::TError>
	where
		TNext: Parse<TRequired = Parsed<T>, TSource<'a> = Chars<'a>>,
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

	#[derive(Debug, PartialEq)]
	struct _Error;

	#[derive(Debug, PartialEq)]
	struct _First;

	impl Parse for _First {
		type TRequired = ParsedNothing;
		type TSource<'a> = Chars<'a>;
		type TError = _Error;

		fn parse(value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
			Ok((_First, value))
		}
	}

	#[derive(Debug, PartialEq)]
	struct _Second;

	impl Parse for _Second {
		type TRequired = Parsed<(_First,)>;
		type TSource<'a> = Chars<'a>;
		type TError = _Error;

		fn parse(value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
			Ok((_Second, value))
		}
	}

	#[test]
	fn parse_first() {
		let source = "fst";
		let parser = StringParser::new(source.chars());

		let parser = parser.parse::<_First>();

		assert_eq!(Ok((_First,)), parser.map(StringParser::unpack))
	}

	#[test]
	fn parse_second() {
		let source = "snd";
		let parser = StringParser {
			remaining: source.chars(),
			parsed: ParsedNothing.concat(_First),
		};

		let parser = parser.parse::<_Second>();

		assert_eq!(Ok((_First, _Second,)), parser.map(StringParser::unpack))
	}
}
