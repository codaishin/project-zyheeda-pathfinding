use super::{
	channels::{Alpha, Blue, Green, Red},
	prefix::Prefix,
	HexColorParseError,
};
use crate::{parsers::parsed::Parsed, traits::parse::Parse};
use std::str::Chars;

pub struct NoSuffix;

impl Parse for NoSuffix {
	type TRequired = Parsed<(Prefix, Red, Green, Blue, Alpha)>;
	type TSource<'a> = Chars<'a>;
	type TError = HexColorParseError;

	fn parse(mut value: Self::TSource<'_>) -> Result<(Self, Self::TSource<'_>), Self::TError> {
		let suffix = value.by_ref().collect::<String>();

		if !suffix.is_empty() {
			return Err(HexColorParseError::InvalidSuffix(suffix));
		}

		Ok((NoSuffix, value))
	}
}
