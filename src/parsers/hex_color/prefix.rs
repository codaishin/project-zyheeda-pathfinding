use super::HexColorParseError;
use crate::{parsers::parsed::ParsedNothing, traits::parse::Parse};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub struct Prefix;

impl Parse for Prefix {
	type TRequired = ParsedNothing;
	type TSource<'a> = Chars<'a>;
	type TError = HexColorParseError;

	fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
		let Some(prefix) = value.next() else {
			return Err(HexColorParseError::EmptyString);
		};
		if prefix != '#' {
			return Err(HexColorParseError::InvalidPrefix(prefix));
		}

		Ok((Prefix, value))
	}
}
