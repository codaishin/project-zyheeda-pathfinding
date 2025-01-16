use super::HexColorParseError;
use crate::{parsers::parsed::ParsedNothing, traits::parse::Parse};
use std::str::Chars;

#[derive(Debug, PartialEq)]
pub struct HexPrefix;

impl Parse for HexPrefix {
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

		Ok((HexPrefix, value))
	}
}
